// Copyright 2024 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

#include "clawser/browser/browser_controller.h"

#include "base/json/json_writer.h"
#include "base/strings/string_number_conversions.h"
#include "base/strings/stringprintf.h"
#include "base/strings/utf_string_conversions.h"
#include "clawser/browser/cdp_client.h"
#include "clawser/browser/net_websocket.h"
#include "base/command_line.h"
#include "content/public/browser/browser_context.h"
#include "content/public/browser/render_frame_host.h"
#include "content/public/browser/storage_partition.h"
#include "content/public/browser/web_contents.h"
#include "content/public/common/isolated_world_ids.h"
#include "headless/lib/browser/headless_web_contents_impl.h"
#include "headless/public/headless_browser.h"
#include "headless/public/headless_browser_context.h"
#include "headless/public/headless_web_contents.h"
#include "net/http/http_request_headers.h"
#include "net/traffic_annotation/network_traffic_annotation.h"
#include "services/network/public/cpp/resource_request.h"
#include "services/network/public/cpp/shared_url_loader_factory.h"
#include "services/network/public/mojom/cookie_manager.mojom.h"
#include "services/network/public/mojom/network_context.mojom.h"
#include "url/gurl.h"

namespace clawser::browser {

BrowserController::BrowserController(
    headless::HeadlessBrowser* browser,
    headless::HeadlessBrowserContext* context)
    : browser_(browser), context_(context) {}

BrowserController::~BrowserController() {
  // CdpClient must be destroyed before pages are torn down.
  cdp_client_.reset();
}

std::string BrowserController::Navigate(const std::string& url) {
  GURL gurl(url);
  if (!gurl.is_valid())
    return "";

  headless::HeadlessWebContents* wc =
      context_->CreateWebContentsBuilder().SetInitialURL(gurl).Build();
  if (!wc)
    return "";

  std::string page_id = base::StringPrintf("p%d", next_page_id_++);
  pages_[page_id] = wc;

  // Attach CDP client for capture/replay if watches are registered.
  if (!watch_endpoints_.empty())
    EnsureCdpAttached(page_id);

  return page_id;
}

void BrowserController::ExecuteJS(const std::string& page_id,
                                  const std::string& code,
                                  JSCallback callback) {
  auto it = pages_.find(page_id);
  if (it == pages_.end()) {
    std::move(callback).Run(base::Value("error: page not found"));
    return;
  }

  auto* wc_impl = headless::HeadlessWebContentsImpl::From(it->second);
  content::WebContents* web_contents = wc_impl->web_contents();
  content::RenderFrameHost* rfh = web_contents->GetPrimaryMainFrame();

  rfh->ExecuteJavaScriptForTests(
      base::UTF8ToUTF16(code), std::move(callback),
      content::ISOLATED_WORLD_ID_GLOBAL);
}

std::string BrowserController::AddWatch(const std::string& endpoint) {
  watch_endpoints_.push_back(endpoint);
  // Update the command line so the renderer's BuildHookScript() picks up
  // watches. CommandLine::AppendSwitchASCII uses map semantics (overwrites),
  // so this is safe to call repeatedly. Cross-DLL safe in component builds.
  std::string joined;
  for (size_t i = 0; i < watch_endpoints_.size(); ++i) {
    if (i > 0)
      joined += ",";
    joined += watch_endpoints_[i];
  }
  base::CommandLine::ForCurrentProcess()->AppendSwitchASCII(
      "clawser-watch", joined);

  std::string watch_id = base::StringPrintf("w%d", next_watch_id_++);
  watch_id_to_endpoint_[watch_id] = endpoint;
  endpoint_to_watch_id_[endpoint] = watch_id;
  return watch_id;
}

const std::vector<std::string>& BrowserController::GetWatchEndpoints() const {
  return watch_endpoints_;
}

content::WebContents* BrowserController::GetWebContents(
    const std::string& page_id) {
  auto it = pages_.find(page_id);
  if (it == pages_.end())
    return nullptr;
  auto* wc_impl = headless::HeadlessWebContentsImpl::From(it->second);
  return wc_impl->web_contents();
}

void BrowserController::WaitForCapture(const std::string& watch_id,
                                       uint32_t timeout_ms,
                                       WaitCallback cb) {
  auto ep_it = watch_id_to_endpoint_.find(watch_id);
  if (ep_it == watch_id_to_endpoint_.end()) {
    std::move(cb).Run(/*timed_out=*/false, base::Value::Dict());
    return;
  }

  // Check if we already have a capture for this endpoint.
  if (cdp_client_) {
    auto* existing = cdp_client_->GetLastCapture(ep_it->second);
    if (existing) {
      std::move(cb).Run(/*timed_out=*/false, existing->Clone());
      return;
    }
  }

  // No capture yet — register a pending wait with timeout.
  auto waiter = std::make_unique<PendingWait>();
  waiter->watch_id = watch_id;
  waiter->callback = std::move(cb);
  PendingWait* raw = waiter.get();
  waiter->timeout.Start(
      FROM_HERE, base::Milliseconds(timeout_ms),
      base::BindOnce(&BrowserController::OnWaitTimeout,
                     base::Unretained(this), raw));
  pending_waits_.push_back(std::move(waiter));
}

void BrowserController::ReplayCapture(
    const std::string& watch_id,
    base::OnceCallback<void(base::Value::Dict)> cb) {
  auto ep_it = watch_id_to_endpoint_.find(watch_id);
  if (ep_it == watch_id_to_endpoint_.end() || !cdp_client_) {
    std::move(cb).Run(base::Value::Dict());
    return;
  }
  cdp_client_->Replay(ep_it->second, std::move(cb));
}

base::Value::Dict* BrowserController::GetLastCapture(
    const std::string& watch_id) {
  auto ep_it = watch_id_to_endpoint_.find(watch_id);
  if (ep_it == watch_id_to_endpoint_.end() || !cdp_client_)
    return nullptr;
  return cdp_client_->GetLastCapture(ep_it->second);
}

void BrowserController::OnCaptureReceived(std::string endpoint,
                                          base::Value::Dict data) {
  auto wid_it = endpoint_to_watch_id_.find(endpoint);
  if (wid_it == endpoint_to_watch_id_.end())
    return;

  // Find the first pending wait that matches.
  for (auto& waiter : pending_waits_) {
    if (waiter->watch_id != wid_it->second)
      continue;

    const std::string* type = data.FindString("type");
    bool is_websocket = type && *type == "websocket";

    if (is_websocket) {
      // WebSocket captures resolve immediately (no response to wait for).
      auto owned = std::move(waiter);
      std::erase_if(pending_waits_, [](const auto& w) { return !w; });
      owned->timeout.Stop();
      std::move(owned->callback).Run(/*timed_out=*/false, std::move(data));
    } else {
      // HTTP captures: store data, wait for __clawser_response__.
      waiter->capture_received = true;
      waiter->capture_data = std::move(data);
    }
    return;
  }
}

void BrowserController::OnWaitTimeout(PendingWait* waiter) {
  for (auto it = pending_waits_.begin(); it != pending_waits_.end(); ++it) {
    if (it->get() == waiter) {
      auto owned = std::move(*it);
      pending_waits_.erase(it);
      if (owned->capture_received) {
        // Got capture but response never arrived — return capture-only.
        std::move(owned->callback)
            .Run(/*timed_out=*/false, std::move(owned->capture_data));
      } else {
        std::move(owned->callback)
            .Run(/*timed_out=*/true, base::Value::Dict());
      }
      return;
    }
  }
}

void BrowserController::EnsureCdpAttached(const std::string& page_id) {
  if (!cdp_client_) {
    cdp_client_ = std::make_unique<CdpClient>();
    cdp_client_->SetCaptureCallback(base::BindRepeating(
        &BrowserController::OnCaptureReceived, weak_factory_.GetWeakPtr()));
    cdp_client_->SetResponseCallback(base::BindRepeating(
        &BrowserController::OnResponseReceived,
        weak_factory_.GetWeakPtr()));
  }
  auto* wc = GetWebContents(page_id);
  if (wc)
    cdp_client_->AttachToPage(wc);
}

content::StoragePartition* BrowserController::GetStoragePartition() {
  // Get from any active page's WebContents.
  for (auto& [pid, hwc] : pages_) {
    auto* wc_impl = headless::HeadlessWebContentsImpl::From(hwc);
    auto* wc = wc_impl->web_contents();
    return wc->GetBrowserContext()->GetDefaultStoragePartition();
  }
  return nullptr;
}

void BrowserController::OnResponseReceived(std::string endpoint,
                                           base::Value::Dict response) {
  auto wid_it = endpoint_to_watch_id_.find(endpoint);
  if (wid_it == endpoint_to_watch_id_.end())
    return;

  // Find a pending wait that already has capture data for this endpoint.
  for (auto it = pending_waits_.begin(); it != pending_waits_.end(); ++it) {
    if ((*it)->watch_id == wid_it->second && (*it)->capture_received) {
      auto waiter = std::move(*it);
      pending_waits_.erase(it);
      waiter->timeout.Stop();
      // Merge response into capture data.
      waiter->capture_data.Set("response", std::move(response));
      std::move(waiter->callback)
          .Run(/*timed_out=*/false, std::move(waiter->capture_data));
      return;
    }
  }
}

// --- WebSocket (C++ Mojo network stack) ---

void BrowserController::OpenWebSocket(
    const std::string& page_id,
    const std::string& url,
    base::OnceCallback<void(std::string)> cb) {
  auto* sp = GetStoragePartition();
  if (!sp) {
    std::move(cb).Run("");
    return;
  }

  GURL gurl(url);
  if (!gurl.is_valid()) {
    std::move(cb).Run("");
    return;
  }

  std::string ws_id = base::StringPrintf("ws%d", next_ws_id_++);
  auto ws = std::make_unique<NetWebSocket>();
  NetWebSocket* raw = ws.get();
  websockets_[ws_id] = std::move(ws);

  url::Origin origin = url::Origin::Create(gurl);
  raw->Connect(
      gurl, sp->GetNetworkContext(), origin,
      base::BindOnce(
          [](BrowserController* self, std::string ws_id,
             base::OnceCallback<void(std::string)> cb, bool ok) {
            if (!ok) {
              self->websockets_.erase(ws_id);
              std::move(cb).Run("");
              return;
            }
            std::move(cb).Run(ws_id);
          },
          base::Unretained(this), ws_id, std::move(cb)));
}

void BrowserController::SendWebSocket(
    const std::string& ws_id,
    const std::string& data,
    base::OnceCallback<void(bool)> cb) {
  auto it = websockets_.find(ws_id);
  if (it == websockets_.end()) {
    std::move(cb).Run(false);
    return;
  }
  it->second->Send(data, std::move(cb));
}

void BrowserController::RecvWebSocket(
    const std::string& ws_id,
    uint32_t timeout_ms,
    base::OnceCallback<void(std::string)> cb) {
  auto it = websockets_.find(ws_id);
  if (it == websockets_.end()) {
    std::move(cb).Run("");
    return;
  }
  it->second->Recv(timeout_ms, std::move(cb));
}

void BrowserController::CloseWebSocket(
    const std::string& ws_id,
    base::OnceCallback<void(bool)> cb) {
  auto it = websockets_.find(ws_id);
  if (it == websockets_.end()) {
    std::move(cb).Run(false);
    return;
  }
  it->second->Close();
  websockets_.erase(it);
  std::move(cb).Run(true);
}

// --- HTTP Fetch (C++ SimpleURLLoader) ---

void BrowserController::FetchRequest(
    const std::string& method,
    const std::string& url,
    const base::Value::Dict* headers,
    const std::string* body,
    uint32_t timeout_ms,
    FetchCallback cb) {
  auto* sp = GetStoragePartition();
  if (!sp) {
    std::move(cb).Run(0, base::Value::Dict(), "", "");
    return;
  }

  auto request = std::make_unique<network::ResourceRequest>();
  request->url = GURL(url);
  request->method = method;
  request->credentials_mode = network::mojom::CredentialsMode::kInclude;

  if (headers) {
    for (auto [key, val] : *headers) {
      if (val.is_string())
        request->headers.SetHeader(key, val.GetString());
    }
  }

  net::NetworkTrafficAnnotationTag annotation =
      net::DefineNetworkTrafficAnnotation("clawser_fetch", R"(
        semantics {
          sender: "Clawser Browser"
          description: "HTTP fetch for antidetect browser automation."
          trigger: "User-initiated via Rust API."
          data: "HTTP request data."
          destination: OTHER
        }
        policy { cookies_allowed: YES cookies_store: "user profile" })");

  auto loader = network::SimpleURLLoader::Create(std::move(request),
                                                  annotation);
  loader->SetAllowHttpErrorResults(true);
  if (timeout_ms > 0)
    loader->SetTimeoutDuration(base::Milliseconds(timeout_ms));

  if (body && !body->empty()) {
    // Use Content-Type from request headers if provided.
    std::string content_type = "application/octet-stream";
    if (headers) {
      const std::string* ct = headers->FindString("Content-Type");
      if (!ct) ct = headers->FindString("content-type");
      if (ct) content_type = *ct;
    }
    loader->AttachStringForUpload(*body, content_type);
  }

  auto* loader_ptr = loader.get();
  pending_loaders_.push_back(std::move(loader));

  loader_ptr->DownloadToString(
      sp->GetURLLoaderFactoryForBrowserProcess().get(),
      base::BindOnce(
          [](BrowserController* self,
             network::SimpleURLLoader* loader_ptr,
             FetchCallback cb,
             std::optional<std::string> response_body) {
            int status = 0;
            base::Value::Dict resp_headers;
            std::string final_url;

            if (auto* info = loader_ptr->ResponseInfo()) {
              if (info->headers)
                status = info->headers->response_code();
              final_url = loader_ptr->GetFinalURL().spec();
              // Extract response headers.
              if (info->headers) {
                size_t iter = 0;
                std::string name, value;
                while (info->headers->EnumerateHeaderLines(&iter, &name,
                                                           &value)) {
                  resp_headers.Set(name, value);
                }
              }
            }

            std::string body = response_body.value_or("");

            // Remove the loader from pending list.
            auto& loaders = self->pending_loaders_;
            std::erase_if(loaders, [loader_ptr](const auto& l) {
              return l.get() == loader_ptr;
            });

            std::move(cb).Run(status, std::move(resp_headers),
                              std::move(body), std::move(final_url));
          },
          base::Unretained(this), loader_ptr, std::move(cb)),
      /*max_body_size=*/5 * 1024 * 1024);
}

// --- Cookies (C++ CookieManager) ---

void BrowserController::GetCookies(
    const std::string& url,
    base::OnceCallback<void(base::Value::List)> cb) {
  auto* sp = GetStoragePartition();
  if (!sp) {
    std::move(cb).Run(base::Value::List());
    return;
  }

  auto* cm = sp->GetCookieManagerForBrowserProcess();
  if (url.empty()) {
    cm->GetAllCookies(base::BindOnce(
        [](base::OnceCallback<void(base::Value::List)> cb,
           const net::CookieList& cookies) {
          base::Value::List list;
          for (const auto& c : cookies) {
            base::Value::Dict d;
            d.Set("name", c.Name());
            d.Set("value", c.Value());
            d.Set("domain", c.Domain());
            d.Set("path", c.Path());
            d.Set("secure", c.IsSecure());
            d.Set("httponly", c.IsHttpOnly());
            list.Append(std::move(d));
          }
          std::move(cb).Run(std::move(list));
        },
        std::move(cb)));
  } else {
    // Get cookies for specific URL.
    net::CookieOptions options;
    options.set_include_httponly();
    options.set_same_site_cookie_context(
        net::CookieOptions::SameSiteCookieContext::MakeInclusive());
    cm->GetCookieList(
        GURL(url), options,
        net::CookiePartitionKeyCollection(),
        base::BindOnce(
            [](base::OnceCallback<void(base::Value::List)> cb,
               const net::CookieAccessResultList& included,
               const net::CookieAccessResultList& excluded) {
              base::Value::List list;
              for (const auto& cwar : included) {
                const auto& c = cwar.cookie;
                base::Value::Dict d;
                d.Set("name", c.Name());
                d.Set("value", c.Value());
                d.Set("domain", c.Domain());
                d.Set("path", c.Path());
                d.Set("secure", c.IsSecure());
                d.Set("httponly", c.IsHttpOnly());
                list.Append(std::move(d));
              }
              std::move(cb).Run(std::move(list));
            },
            std::move(cb)));
  }
}

void BrowserController::Shutdown() {
  websockets_.clear();
  pending_loaders_.clear();
  cdp_client_.reset();
  browser_->Shutdown();
}

}  // namespace clawser::browser

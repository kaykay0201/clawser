// Copyright 2024 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

#ifndef CLAWSER_BROWSER_BROWSER_CONTROLLER_H_
#define CLAWSER_BROWSER_BROWSER_CONTROLLER_H_

#include <map>
#include <memory>
#include <string>
#include <vector>

#include "base/functional/callback.h"
#include "base/memory/raw_ptr.h"
#include "base/memory/weak_ptr.h"
#include "base/timer/timer.h"
#include "base/values.h"
#include "services/network/public/cpp/simple_url_loader.h"

namespace headless {
class HeadlessBrowser;
class HeadlessBrowserContext;
class HeadlessWebContents;
}  // namespace headless

namespace content {
class WebContents;
}

namespace clawser::browser {

namespace network::mojom {
class CookieManager;
class NetworkContext;
}  // namespace network::mojom

class CdpClient;
class NetWebSocket;

// Controls the headless browser instance. Manages pages, JS execution,
// watch registration, and lifecycle. All methods must be called on the
// browser UI thread.
class BrowserController {
 public:
  BrowserController(headless::HeadlessBrowser* browser,
                    headless::HeadlessBrowserContext* context);
  ~BrowserController();

  // Navigate to URL, returns page_id.
  std::string Navigate(const std::string& url);

  // Execute JS in the given page's main world.
  using JSCallback = base::OnceCallback<void(base::Value)>;
  void ExecuteJS(const std::string& page_id,
                 const std::string& code,
                 JSCallback callback);

  // Register an endpoint to watch. Returns watch_id.
  std::string AddWatch(const std::string& endpoint);

  // Get all registered watch endpoints.
  const std::vector<std::string>& GetWatchEndpoints() const;

  // Get WebContents for a page_id (or nullptr).
  content::WebContents* GetWebContents(const std::string& page_id);

  // Wait for a watched endpoint to fire. Calls |cb| with the captured
  // data, or times out after |timeout_ms| milliseconds.
  using WaitCallback =
      base::OnceCallback<void(bool timed_out, base::Value::Dict data)>;
  void WaitForCapture(const std::string& watch_id,
                      uint32_t timeout_ms,
                      WaitCallback cb);

  // Re-invoke the captured caller for a fresh payload.
  void ReplayCapture(const std::string& watch_id,
                     base::OnceCallback<void(base::Value::Dict)> cb);

  // Get the last captured data for a watch_id (non-blocking).
  base::Value::Dict* GetLastCapture(const std::string& watch_id);

  // Open a WebSocket via the C++ network stack (no JS, full antidetect).
  void OpenWebSocket(const std::string& page_id,
                     const std::string& url,
                     base::OnceCallback<void(std::string ws_id)> cb);

  // Send data on a WebSocket.
  void SendWebSocket(const std::string& ws_id,
                     const std::string& data,
                     base::OnceCallback<void(bool ok)> cb);

  // Receive data from a WebSocket (blocks with timeout).
  void RecvWebSocket(const std::string& ws_id,
                     uint32_t timeout_ms,
                     base::OnceCallback<void(std::string data)> cb);

  // Close a WebSocket connection.
  void CloseWebSocket(const std::string& ws_id,
                      base::OnceCallback<void(bool ok)> cb);

  // HTTP fetch via C++ network stack (SimpleURLLoader).
  using FetchCallback =
      base::OnceCallback<void(int status,
                              base::Value::Dict headers,
                              std::string body,
                              std::string final_url)>;
  void FetchRequest(const std::string& method,
                    const std::string& url,
                    const base::Value::Dict* headers,
                    const std::string* body,
                    uint32_t timeout_ms,
                    FetchCallback cb);

  // Get cookies from the browser's cookie store.
  void GetCookies(const std::string& url,
                  base::OnceCallback<void(base::Value::List)> cb);

  // Shutdown the browser.
  void Shutdown();

 private:
  // Called by CdpClient when a capture arrives.
  void OnCaptureReceived(std::string endpoint, base::Value::Dict data);

  // Called when a pending wait times out.
  struct PendingWait;
  void OnWaitTimeout(PendingWait* waiter);
  raw_ptr<headless::HeadlessBrowser> browser_;
  raw_ptr<headless::HeadlessBrowserContext> context_;

  // page_id -> HeadlessWebContents mapping
  std::map<std::string, headless::HeadlessWebContents*> pages_;
  int next_page_id_ = 0;

  // Watch state
  std::vector<std::string> watch_endpoints_;
  int next_watch_id_ = 0;
  std::map<std::string, std::string> watch_id_to_endpoint_;
  std::map<std::string, std::string> endpoint_to_watch_id_;

  // WebSocket connections (C++ Mojo-based, no JS).
  std::map<std::string, std::unique_ptr<NetWebSocket>> websockets_;
  int next_ws_id_ = 0;

  // Pending HTTP fetches (SimpleURLLoader must stay alive until callback).
  std::vector<std::unique_ptr<network::SimpleURLLoader>> pending_loaders_;

  // Get the StoragePartition for network access.
  content::StoragePartition* GetStoragePartition();

  // Ensure CdpClient is attached for the given page.
  void EnsureCdpAttached(const std::string& page_id);

  // Called when a response arrives for a watched endpoint.
  void OnResponseReceived(std::string endpoint, base::Value::Dict response);

  // CDP client for capture + replay.
  std::unique_ptr<CdpClient> cdp_client_;

  // Pending async waits. For HTTP watches, waits for both capture + response.
  struct PendingWait {
    std::string watch_id;
    base::OneShotTimer timeout;
    WaitCallback callback;
    bool capture_received = false;
    base::Value::Dict capture_data;
  };
  std::vector<std::unique_ptr<PendingWait>> pending_waits_;

  // Must be last member.
  base::WeakPtrFactory<BrowserController> weak_factory_{this};
};

}  // namespace clawser::browser

#endif  // CLAWSER_BROWSER_BROWSER_CONTROLLER_H_

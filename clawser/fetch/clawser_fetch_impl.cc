#include "clawser/fetch/clawser_fetch_impl.h"

#include <string>

#include "base/at_exit.h"
#include "base/command_line.h"
#include "base/files/file_util.h"
#include "base/functional/bind.h"
#include "base/logging.h"
#include "base/message_loop/message_pump_type.h"
#include "base/rand_util.h"
#include "base/strings/string_number_conversions.h"
#include "base/strings/string_util.h"
#include "base/strings/stringprintf.h"
#include "base/task/single_thread_task_runner.h"
#include "base/task/thread_pool/thread_pool_instance.h"
#include "clawser/clawser_config.h"
#include "clawser/fetch/clawser_fetch.h"
#include "clawser/hardware_profiles.h"
#include "net/base/elements_upload_data_stream.h"
#include "net/base/io_buffer.h"
#include "net/base/upload_bytes_element_reader.h"
#include "net/base/upload_data_stream.h"
#include "net/base/upload_element_reader.h"
#include "net/cookies/canonical_cookie.h"
#include "net/cookies/cookie_monster.h"
#include "net/cookies/cookie_util.h"
#include "net/traffic_annotation/network_traffic_annotation.h"
#include "net/url_request/url_request_context_builder.h"
#include "url/gurl.h"

namespace clawser::fetch {

namespace {

thread_local std::string g_last_error;

const net::NetworkTrafficAnnotationTag kTrafficAnnotation =
    net::DefineNetworkTrafficAnnotation("clawser_fetch", R"(
      semantics {
        sender: "ClawserFetch"
        description: "HTTP fetch via Chromium net stack"
        trigger: "Programmatic API call"
        data: "User-specified URL and headers"
        destination: OTHER
      }
      policy {
        cookies_allowed: YES
        cookies_store: "user"
      }
    )");

class BaseInitializer {
 public:
  static BaseInitializer& Get() {
    static BaseInitializer instance;
    return instance;
  }

 private:
  BaseInitializer() {
    if (!base::CommandLine::InitializedForCurrentProcess()) {
      base::CommandLine::Init(0, nullptr);
    }
    if (!base::ThreadPoolInstance::Get()) {
      base::ThreadPoolInstance::CreateAndStartWithDefaultParams("ClawserFetch");
    }
  }
  ~BaseInitializer() = default;
  base::AtExitManager at_exit_;
};

struct ChromeVersionInfo {
  const char* version;
  const char* grease_brand;
  const char* grease_version;
};

// Chrome version → matching GREASE brand for sec-ch-ua.
// The GREASE brand rotates with each major version.
constexpr ChromeVersionInfo kChromeVersions[] = {
    {"135", "Not-A.Brand", "8"},
    {"134", "Not:A-Brand", "24"},
    {"133", "Not:A-Brand", "24"},
    {"132", "Not A(Brand", "99"},
    {"131", "Not/A)Brand", "8"},
    {"130", "Not?A_Brand", "99"},
};
constexpr size_t kNumChromeVersions = std::size(kChromeVersions);

// Chrome's canonical header order for navigation requests.
// WAFs fingerprint header ordering — sending them out of order is a bot signal.
constexpr const char* kCanonicalHeaderOrder[] = {
    "Host",
    "Connection",
    "sec-ch-ua",
    "sec-ch-ua-mobile",
    "sec-ch-ua-platform",
    "Upgrade-Insecure-Requests",
    "User-Agent",
    "Accept",
    "Sec-Fetch-Site",
    "Sec-Fetch-Mode",
    "Sec-Fetch-User",
    "Sec-Fetch-Dest",
    "Accept-Encoding",
    "Accept-Language",
    "Priority",
};
constexpr size_t kCanonicalHeaderCount = std::size(kCanonicalHeaderOrder);

}  // namespace

void SetLastError(const std::string& error) {
  g_last_error = error;
}

FetchResponse::FetchResponse() = default;
FetchResponse::~FetchResponse() = default;
FetchRequest::FetchRequest() = default;
FetchRequest::~FetchRequest() = default;
RequestWithSession::RequestWithSession() = default;
RequestWithSession::~RequestWithSession() = default;

// --- FetchSession ---

FetchSession::FetchSession() : io_thread_("ClawserFetchIO") {}

FetchSession::~FetchSession() {
  if (io_thread_.IsRunning()) {
    io_thread_.task_runner()->PostTask(
        FROM_HERE,
        base::BindOnce(
            [](std::unique_ptr<net::URLRequestContext>* ctx) { ctx->reset(); },
            base::Unretained(&context_)));
    io_thread_.Stop();
  }
}

void FetchSession::ApplySeedToConfig(uint64_t hw_seed,
                                     uint64_t canvas_seed,
                                     uint64_t webgl_seed,
                                     uint64_t audio_seed,
                                     uint64_t client_rects_seed) {
  const auto& profiles = GetHardwareProfiles();
  size_t hw_idx = hw_seed % profiles.size();
  size_t ver_idx = (hw_seed >> 32) % kNumChromeVersions;
  const HardwareProfile& hw = profiles[hw_idx];
  const auto& ver = kChromeVersions[ver_idx];

  std::string json = base::StringPrintf(
      R"({
  "version": 1,
  "profile_id": "seed-%)" PRIu64 R"(",
  "navigator": {
    "user_agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/%s.0.0.0 Safari/537.36",
    "platform": "Win32",
    "vendor": "Google Inc.",
    "app_version": "5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/%s.0.0.0 Safari/537.36",
    "language": "en-US",
    "languages": ["en-US", "en"],
    "hardware_concurrency": %d,
    "device_memory": %d,
    "max_touch_points": 0,
    "user_agent_data": {
      "brands": [
        {"brand": "Google Chrome", "version": "%s"},
        {"brand": "Chromium", "version": "%s"},
        {"brand": "%s", "version": "%s"}
      ],
      "mobile": false,
      "platform": "Windows",
      "platform_version": "15.0.0",
      "architecture": "x86",
      "model": "",
      "bitness": "64"
    }
  },
  "screen": {
    "width": %d, "height": %d,
    "avail_width": %d, "avail_height": %d,
    "color_depth": 24, "pixel_depth": 24,
    "device_pixel_ratio": 1.0
  },
  "gpu": {
    "vendor": "%s",
    "renderer": "%s"
  },
  "noise_seeds": {
    "canvas": %)" PRIu64 R"(,
    "webgl": %)" PRIu64 R"(,
    "audio": %)" PRIu64 R"(,
    "client_rects": %)" PRIu64 R"(
  },
  "timezone": "America/New_York",
  "locale": "en-US",
  "media_devices": { "audio_inputs": 1, "audio_outputs": 1, "video_inputs": 1 },
  "webrtc": { "policy": "disabled" },
  "battery": { "enabled": false },
  "bluetooth": { "enabled": false },
  "usb": { "enabled": false }
})",
      hw_seed, ver.version, ver.version, hw.hardware_concurrency,
      hw.device_memory, ver.version, ver.version,
      ver.grease_brand, ver.grease_version, hw.screen_width,
      hw.screen_height, hw.screen_width, hw.screen_height - 40,
      hw.gl_vendor.c_str(), hw.gl_renderer.c_str(), canvas_seed, webgl_seed,
      audio_seed, client_rects_seed);

  ClawserConfigManager::GetInstance().ParseJson(json);
}

void FetchSession::SetupDefaultHeaders() {
  const auto& config_mgr = ClawserConfigManager::GetInstance();
  if (!config_mgr.IsLoaded())
    return;

  const auto& config = config_mgr.GetConfig();
  const auto& nav = config.navigator;

  // Header order matters for fingerprinting. Chrome sends headers in this
  // specific order for navigation requests. We set them in order so that
  // iteration produces the correct sequence.

  // 1. sec-ch-ua headers
  if (!nav.user_agent_data.brands.empty()) {
    std::string sec_ch_ua;
    for (size_t i = 0; i < nav.user_agent_data.brands.size(); ++i) {
      if (i > 0)
        sec_ch_ua += ", ";
      sec_ch_ua += "\"" + nav.user_agent_data.brands[i].brand + "\";v=\"" +
                   nav.user_agent_data.brands[i].version + "\"";
    }
    default_headers_.SetHeader("sec-ch-ua", sec_ch_ua);
  }

  default_headers_.SetHeader("sec-ch-ua-mobile",
                             nav.user_agent_data.mobile ? "?1" : "?0");
  if (!nav.user_agent_data.platform.empty()) {
    default_headers_.SetHeader(
        "sec-ch-ua-platform", "\"" + nav.user_agent_data.platform + "\"");
  }

  // 2. Upgrade-Insecure-Requests (before User-Agent in Chrome's order)
  default_headers_.SetHeader("Upgrade-Insecure-Requests", "1");

  // 3. User-Agent is set by URLRequestContext, but we add Accept right after

  // 4. Accept — Chrome always sends this for navigation requests
  default_headers_.SetHeader(
      "Accept",
      "text/html,application/xhtml+xml,application/xml;q=0.9,"
      "image/avif,image/webp,image/apng,*/*;q=0.8,"
      "application/signed-exchange;v=b3;q=0.7");

  // 5. Sec-Fetch-* headers
  default_headers_.SetHeader("Sec-Fetch-Site", "none");
  default_headers_.SetHeader("Sec-Fetch-Mode", "navigate");
  default_headers_.SetHeader("Sec-Fetch-User", "?1");
  default_headers_.SetHeader("Sec-Fetch-Dest", "document");

  // 6. Accept-Encoding — must match what the net stack actually supports
  default_headers_.SetHeader("Accept-Encoding", "gzip, deflate, br, zstd");

  // 7. Accept-Language — derived from the config's language settings
  if (!nav.languages.empty()) {
    std::string accept_lang;
    for (size_t i = 0; i < nav.languages.size(); ++i) {
      if (i > 0) {
        accept_lang += ",";
        // Chrome uses decreasing quality values: 0.9, 0.8, 0.7, ...
        double q = 1.0 - (i * 0.1);
        if (q < 0.1)
          q = 0.1;
        accept_lang += nav.languages[i] + ";q=" +
                       base::StringPrintf("%.1f", q);
      } else {
        accept_lang += nav.languages[i];
      }
    }
    default_headers_.SetHeader("Accept-Language", accept_lang);
  } else if (!nav.language.empty()) {
    default_headers_.SetHeader("Accept-Language", nav.language);
  } else {
    default_headers_.SetHeader("Accept-Language", "en-US,en;q=0.9");
  }

  // 8. Priority — Chrome 131+ sends this for navigation requests
  default_headers_.SetHeader("Priority", "u=0, i");
}

bool FetchSession::BuildContext() {
  bool success = false;
  base::WaitableEvent done;

  io_thread_.task_runner()->PostTask(
      FROM_HERE, base::BindOnce(
                     [](FetchSession* self, bool* success,
                        base::WaitableEvent* done) {
                       net::URLRequestContextBuilder builder;

                       if (ClawserConfigManager::GetInstance().IsLoaded()) {
                         const auto& config =
                             ClawserConfigManager::GetInstance().GetConfig();
                         if (!config.navigator.user_agent.empty()) {
                           builder.set_user_agent(config.navigator.user_agent);
                         }
                       }

                       builder.DisableHttpCache();
                       builder.set_enable_brotli(true);
                       builder.set_enable_zstd(true);

                       self->context_ = builder.Build();
                       *success = (self->context_ != nullptr);
                       done->Signal();
                     },
                     base::Unretained(this), &success, &done));

  done.Wait();
  if (!success)
    SetLastError("Failed to build URLRequestContext");
  return success;
}

bool FetchSession::InitFromSeed(uint64_t hw_seed,
                                uint64_t canvas_seed,
                                uint64_t webgl_seed,
                                uint64_t audio_seed,
                                uint64_t client_rects_seed) {
  BaseInitializer::Get();

  base::Thread::Options options(base::MessagePumpType::IO, 0);
  if (!io_thread_.StartWithOptions(std::move(options))) {
    SetLastError("Failed to start IO thread");
    return false;
  }

  ApplySeedToConfig(hw_seed, canvas_seed, webgl_seed, audio_seed,
                    client_rects_seed);

  if (!BuildContext())
    return false;
  SetupDefaultHeaders();
  return true;
}

bool FetchSession::Init(const std::string& config_json_path) {
  BaseInitializer::Get();

  base::Thread::Options options(base::MessagePumpType::IO, 0);
  if (!io_thread_.StartWithOptions(std::move(options))) {
    SetLastError("Failed to start IO thread");
    return false;
  }

  if (!config_json_path.empty()) {
    if (!ClawserConfigManager::GetInstance().IsLoaded()) {
      ClawserConfigManager::GetInstance().LoadFromFile(config_json_path);
    }
  }

  if (!BuildContext())
    return false;
  SetupDefaultHeaders();
  return true;
}

std::unique_ptr<FetchResponse> FetchSession::Send(
    std::unique_ptr<FetchRequest> request) {
  // --- Step 1: Merge all header sources into a flat list (caller wins). ---
  // We use a vector of pairs so we can track both the canonical name and value.
  std::vector<std::pair<std::string, std::string>> merged;

  auto find_merged = [&](const std::string& name)
      -> std::pair<std::string, std::string>* {
    for (auto& entry : merged) {
      if (base::EqualsCaseInsensitiveASCII(entry.first, name))
        return &entry;
    }
    return nullptr;
  };

  // Seed with session defaults.
  {
    net::HttpRequestHeaders::Iterator it(default_headers_);
    while (it.GetNext())
      merged.emplace_back(it.name(), it.value());
  }

  // Caller headers override defaults.
  {
    net::HttpRequestHeaders::Iterator it(request->headers);
    while (it.GetNext()) {
      auto* existing = find_merged(it.name());
      if (existing) {
        existing->second = it.value();
      } else {
        merged.emplace_back(it.name(), it.value());
      }
    }
  }

  // Auto-calculate Host from URL if not explicitly set.
  if (!find_merged("Host")) {
    GURL request_url(request->url);
    if (request_url.is_valid()) {
      std::string host = request_url.host();
      // Only include port if non-default for the scheme.
      if (request_url.has_port()) {
        int port = request_url.IntPort();
        if (!((request_url.SchemeIs("https") && port == 443) ||
              (request_url.SchemeIs("http") && port == 80))) {
          host += ":" + request_url.port();
        }
      }
      merged.emplace_back("Host", host);
    }
  }

  // --- Step 2: Rebuild headers in Chrome's canonical order. ---
  request->headers.Clear();

  // Emit canonical headers first, in the correct order.
  for (size_t i = 0; i < kCanonicalHeaderCount; ++i) {
    auto* entry = find_merged(kCanonicalHeaderOrder[i]);
    if (entry) {
      // Use the canonical casing from the table, not whatever the caller used.
      request->headers.SetHeader(kCanonicalHeaderOrder[i], entry->second);
    }
  }

  // Append any non-canonical headers at the end (e.g. custom headers like
  // X-Requested-With, Origin, Referer, Cookie, Content-Type, etc.).
  for (const auto& [name, value] : merged) {
    bool is_canonical = false;
    for (size_t i = 0; i < kCanonicalHeaderCount; ++i) {
      if (base::EqualsCaseInsensitiveASCII(name, kCanonicalHeaderOrder[i])) {
        is_canonical = true;
        break;
      }
    }
    if (!is_canonical)
      request->headers.SetHeaderIfMissing(name, value);
  }

  auto delegate = std::make_unique<BlockingFetchDelegate>();
  delegate->set_max_redirects(request->max_redirects);
  BlockingFetchDelegate* delegate_ptr = delegate.get();

  io_thread_.task_runner()->PostTask(
      FROM_HERE,
      base::BindOnce(
          [](net::URLRequestContext* context, FetchRequest* req,
             BlockingFetchDelegate* delegate) {
            GURL url(req->url);
            if (!url.is_valid()) {
              SetLastError("Invalid URL: " + req->url);
              delegate->WaitForCompletion();
              return;
            }

            std::unique_ptr<net::URLRequest> url_request =
                context->CreateRequest(
                    url, net::DEFAULT_PRIORITY,
                    static_cast<net::URLRequest::Delegate*>(delegate),
                    kTrafficAnnotation);
            url_request->set_method(req->method);
            url_request->SetExtraRequestHeaders(req->headers);

            if (!req->body.empty()) {
              auto reader = std::make_unique<net::UploadBytesElementReader>(
                  base::span<const uint8_t>(req->body));
              std::vector<std::unique_ptr<net::UploadElementReader>> readers;
              readers.push_back(std::move(reader));
              url_request->set_upload(
                  std::make_unique<net::ElementsUploadDataStream>(
                      std::move(readers), 0));
            }

            url_request->Start();
            url_request.release();
          },
          base::Unretained(context_.get()), request.get(), delegate_ptr));

  delegate->WaitForCompletion();

  if (delegate->net_error() != 0 && !delegate->TakeResponse()) {
    SetLastError("Network error: " +
                 base::NumberToString(delegate->net_error()));
    return nullptr;
  }

  return delegate->TakeResponse();
}

std::string FetchSession::GetAllCookiesJson() {
  std::string result;
  base::WaitableEvent done;

  io_thread_.task_runner()->PostTask(
      FROM_HERE,
      base::BindOnce(
          [](net::URLRequestContext* context, std::string* result,
             base::WaitableEvent* done) {
            context->cookie_store()->GetAllCookiesAsync(
                base::BindOnce(
                    [](std::string* result, base::WaitableEvent* done,
                       const net::CookieList& cookies) {
                      std::string json = "[";
                      for (size_t i = 0; i < cookies.size(); ++i) {
                        const auto& c = cookies[i];
                        if (i > 0)
                          json += ",";
                        json += "{\"name\":\"" + c.Name() +
                                "\",\"value\":\"" + c.Value() +
                                "\",\"domain\":\"" + c.Domain() +
                                "\",\"path\":\"" + c.Path() + "\"" +
                                ",\"secure\":" +
                                (c.IsSecure() ? "true" : "false") +
                                ",\"httponly\":" +
                                (c.IsHttpOnly() ? "true" : "false") + "}";
                      }
                      json += "]";
                      *result = std::move(json);
                      done->Signal();
                    },
                    result, done));
          },
          base::Unretained(context_.get()), &result, &done));

  done.Wait();
  return result;
}

// --- BlockingFetchDelegate ---

BlockingFetchDelegate::BlockingFetchDelegate()
    : completion_event_(base::WaitableEvent::ResetPolicy::MANUAL,
                        base::WaitableEvent::InitialState::NOT_SIGNALED),
      read_buf_(base::MakeRefCounted<net::IOBufferWithSize>(16384)),
      response_(std::make_unique<FetchResponse>()) {}

BlockingFetchDelegate::~BlockingFetchDelegate() = default;

void BlockingFetchDelegate::OnReceivedRedirect(
    net::URLRequest* request,
    const net::RedirectInfo& redirect_info,
    bool* defer_redirect) {
  redirect_count_++;
  if (max_redirects_ == 0 ||
      (max_redirects_ > 0 && redirect_count_ > max_redirects_)) {
    request->Cancel();
    return;
  }
}

void BlockingFetchDelegate::OnResponseStarted(net::URLRequest* request,
                                              int net_error) {
  net_error_ = net_error;
  if (net_error != net::OK) {
    delete request;
    completion_event_.Signal();
    return;
  }

  response_->status_code = request->GetResponseCode();
  response_->final_url = request->url().spec();

  if (request->response_headers()) {
    size_t iter = 0;
    std::string name, value;
    while (request->response_headers()->EnumerateHeaderLines(&iter, &name,
                                                             &value)) {
      response_->headers.emplace_back(name, value);
    }
  }

  ReadBody(request);
}

void BlockingFetchDelegate::OnReadCompleted(net::URLRequest* request,
                                            int bytes_read) {
  if (bytes_read > 0) {
    response_->body.insert(response_->body.end(), read_buf_->data(),
                           read_buf_->data() + bytes_read);
    ReadBody(request);
    return;
  }
  if (bytes_read < 0)
    net_error_ = bytes_read;
  delete request;
  completion_event_.Signal();
}

void BlockingFetchDelegate::ReadBody(net::URLRequest* request) {
  while (true) {
    int bytes_read = request->Read(read_buf_.get(), read_buf_->size());
    if (bytes_read == net::ERR_IO_PENDING)
      return;
    if (bytes_read <= 0) {
      if (bytes_read < 0)
        net_error_ = bytes_read;
      delete request;
      completion_event_.Signal();
      return;
    }
    response_->body.insert(response_->body.end(), read_buf_->data(),
                           read_buf_->data() + bytes_read);
  }
}

void BlockingFetchDelegate::WaitForCompletion() {
  completion_event_.Wait();
}

std::unique_ptr<FetchResponse> BlockingFetchDelegate::TakeResponse() {
  return std::move(response_);
}

}  // namespace clawser::fetch

// --- C API ---

extern "C" {

ClawserSession* clawser_session_create_random(ClawserSeed* out_seed) {
  uint64_t hw = base::RandUint64();
  uint64_t canvas = base::RandUint64();
  uint64_t webgl = base::RandUint64();
  uint64_t audio = base::RandUint64();
  uint64_t rects = base::RandUint64();

  auto session = std::make_unique<clawser::fetch::FetchSession>();
  if (!session->InitFromSeed(hw, canvas, webgl, audio, rects))
    return nullptr;

  if (out_seed) {
    out_seed->hw_seed = hw;
    out_seed->canvas_seed = canvas;
    out_seed->webgl_seed = webgl;
    out_seed->audio_seed = audio;
    out_seed->client_rects_seed = rects;
  }

  return reinterpret_cast<ClawserSession*>(session.release());
}

ClawserSession* clawser_session_from_seed(const ClawserSeed* seed) {
  if (!seed) {
    clawser::fetch::SetLastError("Invalid argument: null seed");
    return nullptr;
  }
  auto session = std::make_unique<clawser::fetch::FetchSession>();
  if (!session->InitFromSeed(seed->hw_seed, seed->canvas_seed,
                             seed->webgl_seed, seed->audio_seed,
                             seed->client_rects_seed))
    return nullptr;
  return reinterpret_cast<ClawserSession*>(session.release());
}

ClawserSession* clawser_session_create(const char* config_json_path) {
  auto session = std::make_unique<clawser::fetch::FetchSession>();
  std::string path = config_json_path ? config_json_path : "";
  if (!session->Init(path))
    return nullptr;
  return reinterpret_cast<ClawserSession*>(session.release());
}

const char* clawser_session_get_cookies(ClawserSession* session) {
  if (!session)
    return "[]";
  thread_local std::string cookies_json;
  cookies_json =
      reinterpret_cast<clawser::fetch::FetchSession*>(session)
          ->GetAllCookiesJson();
  return cookies_json.c_str();
}

void clawser_session_destroy(ClawserSession* session) {
  delete reinterpret_cast<clawser::fetch::FetchSession*>(session);
}

const char* clawser_last_error(void) {
  return clawser::fetch::g_last_error.c_str();
}

ClawserRequest* clawser_request_new(ClawserSession* session,
                                    const char* method,
                                    const char* url) {
  if (!session || !method || !url) {
    clawser::fetch::SetLastError("Invalid argument: null pointer");
    return nullptr;
  }
  auto* rws = new clawser::fetch::RequestWithSession();
  rws->session = reinterpret_cast<clawser::fetch::FetchSession*>(session);
  rws->request = std::make_unique<clawser::fetch::FetchRequest>();
  rws->request->method = method;
  rws->request->url = url;
  return reinterpret_cast<ClawserRequest*>(rws);
}

void clawser_request_set_header(ClawserRequest* req, const char* name,
                                const char* value) {
  if (!req || !name || !value) return;
  auto* rws = reinterpret_cast<clawser::fetch::RequestWithSession*>(req);
  rws->request->headers.SetHeader(name, value);
}

void clawser_request_set_body(ClawserRequest* req, const uint8_t* data,
                              size_t len) {
  if (!req || !data || len == 0) return;
  auto* rws = reinterpret_cast<clawser::fetch::RequestWithSession*>(req);
  rws->request->body.assign(data, data + len);
}

void clawser_request_set_max_redirects(ClawserRequest* req, int max_redirects) {
  if (!req) return;
  auto* rws = reinterpret_cast<clawser::fetch::RequestWithSession*>(req);
  rws->request->max_redirects = max_redirects;
}

void clawser_request_set_timeout_ms(ClawserRequest* req, uint32_t timeout_ms) {
  if (!req) return;
  auto* rws = reinterpret_cast<clawser::fetch::RequestWithSession*>(req);
  rws->request->timeout_ms = timeout_ms;
}

ClawserResponse* clawser_request_send(ClawserRequest* req) {
  if (!req) {
    clawser::fetch::SetLastError("Invalid argument: null request");
    return nullptr;
  }
  auto* rws = reinterpret_cast<clawser::fetch::RequestWithSession*>(req);
  auto response = rws->session->Send(std::move(rws->request));
  delete rws;
  if (!response) return nullptr;
  return reinterpret_cast<ClawserResponse*>(response.release());
}

void clawser_request_destroy(ClawserRequest* req) {
  if (!req) return;
  delete reinterpret_cast<clawser::fetch::RequestWithSession*>(req);
}

int clawser_response_status_code(const ClawserResponse* resp) {
  if (!resp) return 0;
  return reinterpret_cast<const clawser::fetch::FetchResponse*>(resp)->status_code;
}

const char* clawser_response_header(const ClawserResponse* resp, const char* name) {
  if (!resp || !name) return nullptr;
  const auto* r = reinterpret_cast<const clawser::fetch::FetchResponse*>(resp);
  for (const auto& [k, v] : r->headers) {
    if (k == name) return v.c_str();
  }
  return nullptr;
}

size_t clawser_response_header_count(const ClawserResponse* resp) {
  if (!resp) return 0;
  return reinterpret_cast<const clawser::fetch::FetchResponse*>(resp)->headers.size();
}

const char* clawser_response_header_name_at(const ClawserResponse* resp, size_t index) {
  if (!resp) return nullptr;
  const auto* r = reinterpret_cast<const clawser::fetch::FetchResponse*>(resp);
  if (index >= r->headers.size()) return nullptr;
  return r->headers[index].first.c_str();
}

const char* clawser_response_header_value_at(const ClawserResponse* resp, size_t index) {
  if (!resp) return nullptr;
  const auto* r = reinterpret_cast<const clawser::fetch::FetchResponse*>(resp);
  if (index >= r->headers.size()) return nullptr;
  return r->headers[index].second.c_str();
}

const uint8_t* clawser_response_body(const ClawserResponse* resp) {
  if (!resp) return nullptr;
  return reinterpret_cast<const clawser::fetch::FetchResponse*>(resp)->body.data();
}

size_t clawser_response_body_len(const ClawserResponse* resp) {
  if (!resp) return 0;
  return reinterpret_cast<const clawser::fetch::FetchResponse*>(resp)->body.size();
}

const char* clawser_response_url(const ClawserResponse* resp) {
  if (!resp) return nullptr;
  return reinterpret_cast<const clawser::fetch::FetchResponse*>(resp)->final_url.c_str();
}

void clawser_response_destroy(ClawserResponse* resp) {
  delete reinterpret_cast<clawser::fetch::FetchResponse*>(resp);
}

const char* clawser_version(void) { return "0.1.0"; }

}  // extern "C"

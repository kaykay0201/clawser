#ifndef CLAWSER_FETCH_CLAWSER_FETCH_IMPL_H_
#define CLAWSER_FETCH_CLAWSER_FETCH_IMPL_H_

#include <memory>
#include <string>
#include <utility>
#include <vector>

#include "base/memory/scoped_refptr.h"
#include "base/synchronization/waitable_event.h"
#include "net/base/io_buffer.h"
#include "base/threading/thread.h"
#include "net/http/http_request_headers.h"
#include "net/url_request/url_request.h"
#include "net/url_request/url_request_context.h"

namespace clawser::fetch {

struct FetchResponse {
  FetchResponse();
  ~FetchResponse();

  int status_code = 0;
  std::string final_url;
  std::vector<std::pair<std::string, std::string>> headers;
  std::vector<uint8_t> body;
};

struct FetchRequest {
  FetchRequest();
  ~FetchRequest();

  std::string method;
  std::string url;
  net::HttpRequestHeaders headers;
  std::vector<uint8_t> body;
  int max_redirects = -1;
  uint32_t timeout_ms = 0;
};

class FetchSession {
 public:
  FetchSession();
  ~FetchSession();

  // Init from seed (deterministic).
  bool InitFromSeed(uint64_t hw_seed, uint64_t canvas_seed,
                    uint64_t webgl_seed, uint64_t audio_seed,
                    uint64_t client_rects_seed);
  // Init from a config JSON file path (advanced).
  bool Init(const std::string& config_json_path);

  std::unique_ptr<FetchResponse> Send(std::unique_ptr<FetchRequest> request);
  // Merge session defaults + request headers in Chrome canonical order.
  // Does not consume the request — safe to call before Send().
  net::HttpRequestHeaders PrepareHeaders(const FetchRequest& request);
  std::string GetAllCookiesJson();

  const net::HttpRequestHeaders& default_headers() const {
    return default_headers_;
  }

 private:
  bool BuildContext();
  void ApplySeedToConfig(uint64_t hw_seed, uint64_t canvas_seed,
                         uint64_t webgl_seed, uint64_t audio_seed,
                         uint64_t client_rects_seed);
  void SetupDefaultHeaders();

  base::Thread io_thread_;
  std::unique_ptr<net::URLRequestContext> context_;
  net::HttpRequestHeaders default_headers_;
};

class BlockingFetchDelegate : public net::URLRequest::Delegate {
 public:
  BlockingFetchDelegate();
  ~BlockingFetchDelegate() override;

  void set_max_redirects(int n) { max_redirects_ = n; }

  // URLRequest::Delegate:
  void OnReceivedRedirect(net::URLRequest* request,
                          const net::RedirectInfo& redirect_info,
                          bool* defer_redirect) override;
  void OnResponseStarted(net::URLRequest* request, int net_error) override;
  void OnReadCompleted(net::URLRequest* request, int bytes_read) override;

  void WaitForCompletion();
  std::unique_ptr<FetchResponse> TakeResponse();
  int net_error() const { return net_error_; }

 private:
  void ReadBody(net::URLRequest* request);

  base::WaitableEvent completion_event_;
  int max_redirects_ = 20;
  int redirect_count_ = 0;
  int net_error_ = 0;
  scoped_refptr<net::IOBufferWithSize> read_buf_;
  std::unique_ptr<FetchResponse> response_;
};

struct RequestWithSession {
  RequestWithSession();
  ~RequestWithSession();

  raw_ptr<FetchSession> session = nullptr;
  std::unique_ptr<FetchRequest> request;
  // Cached result from preview_headers.
  std::vector<std::pair<std::string, std::string>> preview_cache;
};

void SetLastError(const std::string& error);

}  // namespace clawser::fetch

#endif  // CLAWSER_FETCH_CLAWSER_FETCH_IMPL_H_

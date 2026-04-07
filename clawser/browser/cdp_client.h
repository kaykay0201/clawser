// Copyright 2024 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

#ifndef CLAWSER_BROWSER_CDP_CLIENT_H_
#define CLAWSER_BROWSER_CDP_CLIENT_H_

#include <map>
#include <string>

#include "base/functional/callback.h"
#include "base/memory/scoped_refptr.h"
#include "base/values.h"
#include "content/public/browser/devtools_agent_host.h"
#include "content/public/browser/devtools_agent_host_client.h"

namespace content {
class WebContents;
}

namespace clawser::browser {

// CDP (Chrome DevTools Protocol) client that attaches to a page's V8
// inspector to capture API calls and replay them. Uses Runtime domain
// for console.debug signals from hook scripts, and Debugger domain
// for call frame inspection when debugger; statements hit.
//
// Lifecycle: created by BrowserController, attached per-page via
// AttachToPage(). Must be destroyed on the UI thread.
class CdpClient : public content::DevToolsAgentHostClient {
 public:
  // Fired when a watched endpoint captures data.
  using CaptureCallback =
      base::RepeatingCallback<void(std::string endpoint,
                                   base::Value::Dict data)>;

  // Fired when a response arrives for a captured endpoint.
  using ResponseCallback =
      base::RepeatingCallback<void(std::string endpoint,
                                   base::Value::Dict response)>;

  CdpClient();
  ~CdpClient() override;

  // Attach to a page's DevTools agent. Enables Runtime + Debugger.
  void AttachToPage(content::WebContents* wc);

  // Detach from the current page (if attached).
  void Detach();

  // Set callback for capture events.
  void SetCaptureCallback(CaptureCallback cb);

  // Set callback for response events (fired after capture).
  void SetResponseCallback(ResponseCallback cb);

  // Get the last captured data for an endpoint, or nullptr.
  base::Value::Dict* GetLastCapture(const std::string& endpoint);

  // Re-invoke the captured caller function for a fresh payload.
  // Fires callback with the new capture data, or empty dict on failure.
  void Replay(const std::string& endpoint,
              base::OnceCallback<void(base::Value::Dict)> cb);

  // Evaluate JS in the page's main context via CDP Runtime.evaluate.
  void EvaluateJS(const std::string& expression,
                  bool await_promise,
                  base::OnceCallback<void(base::Value)> cb);

  // Evaluate JS in an isolated world (invisible to page JS).
  // Creates the world on first call. Uses the same network stack.
  void EvaluateInIsolatedWorld(const std::string& expression,
                               bool await_promise,
                               base::OnceCallback<void(base::Value)> cb);

  // Get all cookies via CDP Network.getCookies.
  void GetCookies(base::OnceCallback<void(base::Value::List)> cb);

  // content::DevToolsAgentHostClient overrides
  void DispatchProtocolMessage(
      content::DevToolsAgentHost* agent_host,
      base::span<const uint8_t> message) override;
  void AgentHostClosed(content::DevToolsAgentHost* agent_host) override;
  bool IsTrusted() override;

 private:
  // Send a CDP command, returns the command id.
  int SendCommand(const std::string& method, base::Value::Dict params);

  // Convenience overload with no params.
  int SendCommand(const std::string& method);

  // CDP message routing
  void OnCdpResponse(int id, base::Value::Dict result);
  void OnCdpEvent(const std::string& method, base::Value::Dict params);

  // Event handlers
  void OnConsoleAPICalled(base::Value::Dict params);
  void OnDebuggerPaused(base::Value::Dict params);

  // During a pause: evaluate on the caller frame to get a persistent
  // function reference, then resume.
  void CaptureCallerAndResume(const std::string& endpoint,
                              base::Value::Dict capture_data,
                              const base::Value::List& call_frames);

  // Create isolated world if not yet created.
  void EnsureIsolatedWorld(base::OnceClosure then);
  void OnIsolatedWorldCreated(base::OnceClosure then,
                              base::Value::Dict result);

  scoped_refptr<content::DevToolsAgentHost> agent_host_;
  int next_cdp_id_ = 1;
  std::map<int, base::OnceCallback<void(base::Value::Dict)>>
      pending_commands_;

  // Per-endpoint capture state.
  struct CaptureState {
    CaptureState();
    ~CaptureState();
    CaptureState(CaptureState&&);
    CaptureState& operator=(CaptureState&&);

    base::Value::Dict last_capture;
    base::Value::Dict last_response;  // from __clawser_response__
    bool has_response = false;
    std::string caller_object_id;
    std::string caller_function_name;
  };
  std::map<std::string, CaptureState> captures_;

  ResponseCallback response_callback_;

  // Isolated world context ID (0 = not yet created).
  int isolated_context_id_ = 0;
  std::string main_frame_id_;

  CaptureCallback capture_callback_;

  // Debugger pause state machine.
  bool expecting_pause_ = false;
  std::string pause_endpoint_;
  base::Value::Dict pause_capture_data_;

  // Replay state.
  bool replay_in_progress_ = false;
  std::string replay_endpoint_;
  base::OnceCallback<void(base::Value::Dict)> replay_callback_;
};

}  // namespace clawser::browser

#endif  // CLAWSER_BROWSER_CDP_CLIENT_H_

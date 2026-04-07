// Copyright 2024 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

#include "clawser/browser/watcher_engine.h"

#include "base/command_line.h"
#include "base/json/json_writer.h"
#include "base/strings/string_split.h"
#include "base/strings/string_util.h"
#include "base/strings/utf_string_conversions.h"
#include "clawser/browser/chrome_object_setup.h"
#include "clawser/clawser_config.h"
#include "v8/include/v8-context.h"
#include "v8/include/v8-script.h"

namespace clawser::browser {

ClawserWatcherObserver::ClawserWatcherObserver(content::RenderFrame* frame)
    : content::RenderFrameObserver(frame) {}

ClawserWatcherObserver::~ClawserWatcherObserver() = default;

void ClawserWatcherObserver::DidCreateScriptContext(
    v8::Local<v8::Context> context,
    int32_t world_id) {
  // Only inject into the main world (world_id 0).
  if (world_id != 0)
    return;

  // 1. Inject window.chrome object (C++ V8 API, undetectable)
  SetupWindowChromeObject(context);

  // 2. Inject API watcher hooks
  InjectWatcherHooks(context);
}

void ClawserWatcherObserver::OnDestruct() {
  delete this;
}

void ClawserWatcherObserver::InjectWatcherHooks(
    v8::Local<v8::Context> context) {
  if (!ClawserConfigManager::GetInstance().IsLoaded())
    return;

  std::string script = BuildHookScript();
  if (script.empty())
    return;

  v8::Isolate* isolate = context->GetIsolate();
  v8::HandleScope handle_scope(isolate);
  v8::Context::Scope context_scope(context);
  v8::TryCatch try_catch(isolate);

  v8::Local<v8::String> source =
      v8::String::NewFromUtf8(isolate, script.c_str()).ToLocalChecked();
  v8::Local<v8::Script> compiled;
  if (!v8::Script::Compile(context, source).ToLocal(&compiled))
    return;
  compiled->Run(context).FromMaybe(v8::Local<v8::Value>());
}

std::string ClawserWatcherObserver::BuildHookScript() {
  // Read watch endpoints from --clawser-watch command line switch.
  // Set by BrowserController::AddWatch (which uses AppendSwitchASCII,
  // map semantics = overwrites), or by chrome --clawser-config mode.
  // Cross-DLL safe: CommandLine::ForCurrentProcess() is process-global.
  const base::CommandLine& cmd =
      *base::CommandLine::ForCurrentProcess();
  std::string watch_str = cmd.GetSwitchValueASCII("clawser-watch");
  if (watch_str.empty())
    return "";

  std::vector<std::string> endpoints = base::SplitString(
      watch_str, ",", base::TRIM_WHITESPACE, base::SPLIT_WANT_NONEMPTY);

  std::string endpoints_json = "[";
  for (size_t i = 0; i < endpoints.size(); ++i) {
    if (i > 0)
      endpoints_json += ",";
    // Escape for JS string
    std::string escaped;
    base::JSONWriter::Write(base::Value(endpoints[i]), &escaped);
    endpoints_json += escaped;
  }
  endpoints_json += "]";

  // The hook script:
  // - Overrides fetch, XHR.open/send, WebSocket constructor/send
  // - Matches watched endpoints
  // - Signals capture via console.debug('__clawser_captured__', data)
  // - Freezes overrides with Object.defineProperty
  return R"((function() {
  var WATCH = )" + endpoints_json +
         R"(;
  if (!WATCH.length) return;

  const _fetch = window.fetch;
  const _XHROpen = XMLHttpRequest.prototype.open;
  const _XHRSend = XMLHttpRequest.prototype.send;
  const _WS = window.WebSocket;
  const _WSSend = WebSocket.prototype.send;

  function matchEndpoint(url) {
    var s = String(url);
    for (var i = 0; i < WATCH.length; i++) {
      if (s.indexOf(WATCH[i]) !== -1) return WATCH[i];
    }
    return null;
  }

  // --- fetch() hook ---
  var hookedFetch = function(input, init) {
    var url = typeof input === 'string' ? input : (input && input.url) || '';
    var ep = matchEndpoint(url);
    if (ep) {
      var data = {
        type: 'http',
        endpoint: ep,
        method: (init && init.method) || 'GET',
        url: url,
        headers: {},
        body: null
      };
      if (init && init.headers) {
        try {
          var h = new Headers(init.headers);
          h.forEach(function(v, k) { data.headers[k] = v; });
        } catch(e) {}
      }
      if (init && init.body != null) {
        try { data.body = String(init.body); } catch(e) {}
      }
      console.debug('__clawser_captured__', JSON.stringify(data));
      debugger;
      return _fetch.apply(this, arguments).then(function(resp) {
        var clone = resp.clone();
        clone.text().then(function(body) {
          var rh = {};
          resp.headers.forEach(function(v, k) { rh[k] = v; });
          console.debug('__clawser_response__', JSON.stringify({
            endpoint: ep, status: resp.status, headers: rh,
            body: body, url: resp.url
          }));
        });
        return resp;
      });
    }
    return _fetch.apply(this, arguments);
  };
  // Preserve toString to look native
  hookedFetch.toString = function() { return 'function fetch() { [native code] }'; };
  Object.defineProperty(window, 'fetch', {
    value: hookedFetch, writable: false, configurable: false
  });

  // --- XHR hook ---
  var xhrUrls = new WeakMap();
  XMLHttpRequest.prototype.open = function(method, url) {
    xhrUrls.set(this, {method: method, url: String(url)});
    return _XHROpen.apply(this, arguments);
  };
  XMLHttpRequest.prototype.send = function(body) {
    var info = xhrUrls.get(this);
    if (info) {
      var ep = matchEndpoint(info.url);
      if (ep) {
        var data = {
          type: 'http',
          endpoint: ep,
          method: info.method,
          url: info.url,
          headers: {},
          body: body != null ? String(body) : null
        };
        console.debug('__clawser_captured__', JSON.stringify(data));
        debugger;
      }
    }
    return _XHRSend.apply(this, arguments);
  };

  // --- WebSocket constructor hook ---
  window.WebSocket = function(url, protocols) {
    var ep = matchEndpoint(url);
    if (ep) {
      var data = {
        type: 'websocket',
        endpoint: ep,
        url: String(url)
      };
      console.debug('__clawser_ws_created__', JSON.stringify(data));
      debugger;
    }
    var ws = new _WS(url, protocols);
    // Hook send on this instance
    if (ep) {
      var origSend = ws.send.bind(ws);
      ws.send = function(msg) {
        console.debug('__clawser_ws_send__', JSON.stringify({
          endpoint: ep, data: String(msg)
        }));
        return origSend(msg);
      };
      ws.addEventListener('message', function(e) {
        console.debug('__clawser_ws_recv__', JSON.stringify({
          endpoint: ep, data: String(e.data)
        }));
      });
    }
    return ws;
  };
  window.WebSocket.toString = function() { return 'function WebSocket() { [native code] }'; };
  window.WebSocket.CONNECTING = _WS.CONNECTING;
  window.WebSocket.OPEN = _WS.OPEN;
  window.WebSocket.CLOSING = _WS.CLOSING;
  window.WebSocket.CLOSED = _WS.CLOSED;
  window.WebSocket.prototype = _WS.prototype;
})();)";
}

}  // namespace clawser::browser

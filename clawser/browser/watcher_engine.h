// Copyright 2024 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

#ifndef CLAWSER_BROWSER_WATCHER_ENGINE_H_
#define CLAWSER_BROWSER_WATCHER_ENGINE_H_

#include <string>
#include <vector>

#include "content/public/renderer/render_frame_observer.h"
#include "v8/include/v8.h"

namespace clawser::browser {

// RenderFrameObserver that injects API hooks (fetch/XHR/WebSocket)
// and window.chrome object into every new V8 context, BEFORE any
// page JS executes. Instantiated per-RenderFrame from
// HeadlessContentRendererClient::RenderFrameCreated.
class ClawserWatcherObserver : public content::RenderFrameObserver {
 public:
  explicit ClawserWatcherObserver(content::RenderFrame* frame);
  ~ClawserWatcherObserver() override;

  // RenderFrameObserver overrides
  void DidCreateScriptContext(v8::Local<v8::Context> context,
                              int32_t world_id) override;
  void OnDestruct() override;

 private:
  // Injects fetch/XHR/WebSocket interception hooks into the context.
  void InjectWatcherHooks(v8::Local<v8::Context> context);

  // Builds the JS hook script from the watch endpoint list.
  std::string BuildHookScript();
};

}  // namespace clawser::browser

#endif  // CLAWSER_BROWSER_WATCHER_ENGINE_H_

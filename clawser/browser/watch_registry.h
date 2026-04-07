// Copyright 2024 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

#ifndef CLAWSER_BROWSER_WATCH_REGISTRY_H_
#define CLAWSER_BROWSER_WATCH_REGISTRY_H_

#include <string>
#include <vector>

#include "base/no_destructor.h"

namespace clawser::browser {

// Process-global watch endpoint registry. In --single-process mode the
// browser thread's AddWatch() writes here and the renderer thread's
// BuildHookScript() reads it. Access is safe because in single-process
// mode, hooks are injected synchronously during DidCreateScriptContext
// which only fires after navigation — by that point all AddWatch() calls
// from the init command have already completed on the UI thread.
inline std::vector<std::string>& GetWatchRegistry() {
  static base::NoDestructor<std::vector<std::string>> endpoints;
  return *endpoints;
}

}  // namespace clawser::browser

#endif  // CLAWSER_BROWSER_WATCH_REGISTRY_H_

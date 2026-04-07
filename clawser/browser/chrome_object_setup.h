// Copyright 2024 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

#ifndef CLAWSER_BROWSER_CHROME_OBJECT_SETUP_H_
#define CLAWSER_BROWSER_CHROME_OBJECT_SETUP_H_

#include "v8/include/v8.h"

namespace clawser::browser {

// Injects window.chrome object with chrome.app and chrome.runtime stubs
// into the given V8 context. Must be called during DidCreateScriptContext
// before any page JS runs. Only injects if window.chrome doesn't already
// exist (extensions create their own).
//
// Guarded by ClawserConfigManager::IsLoaded() internally.
void SetupWindowChromeObject(v8::Local<v8::Context> context);

}  // namespace clawser::browser

#endif  // CLAWSER_BROWSER_CHROME_OBJECT_SETUP_H_

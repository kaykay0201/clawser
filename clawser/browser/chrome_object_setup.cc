// Copyright 2024 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

#include "clawser/browser/chrome_object_setup.h"

#include "clawser/clawser_config.h"
#include "v8/include/v8-context.h"
#include "v8/include/v8-function.h"
#include "v8/include/v8-object.h"
#include "v8/include/v8-primitive.h"
#include "v8/include/v8-template.h"

namespace clawser::browser {

namespace {

v8::Local<v8::String> V8Str(v8::Isolate* isolate, const char* str) {
  return v8::String::NewFromUtf8(isolate, str).ToLocalChecked();
}

void EmptyCallback(const v8::FunctionCallbackInfo<v8::Value>& info) {
  info.GetReturnValue().SetUndefined();
}

// Builds an object with string key-value pairs, used for enum-like objects
// e.g. { INSTALL: "install", UPDATE: "update" }
v8::Local<v8::Object> BuildEnumObject(
    v8::Isolate* isolate,
    v8::Local<v8::Context> context,
    std::initializer_list<std::pair<const char*, const char*>> entries) {
  v8::Local<v8::Object> obj = v8::Object::New(isolate);
  for (const auto& [key, value] : entries) {
    obj->Set(context, V8Str(isolate, key), V8Str(isolate, value)).Check();
  }
  return obj;
}

}  // namespace

void SetupWindowChromeObject(v8::Local<v8::Context> context) {
  if (!ClawserConfigManager::GetInstance().IsLoaded())
    return;

  v8::Isolate* isolate = context->GetIsolate();
  v8::HandleScope handle_scope(isolate);
  v8::Context::Scope context_scope(context);

  v8::Local<v8::Object> global = context->Global();
  v8::Local<v8::String> chrome_key = V8Str(isolate, "chrome");

  // Don't overwrite if extensions already created window.chrome
  if (global->Has(context, chrome_key).FromMaybe(false))
    return;

  v8::Local<v8::Object> chrome = v8::Object::New(isolate);

  // --- chrome.app ---
  v8::Local<v8::Object> app = v8::Object::New(isolate);
  app->Set(context, V8Str(isolate, "isInstalled"),
           v8::Boolean::New(isolate, false))
      .Check();
  app->Set(context, V8Str(isolate, "getIsInstalled"),
           v8::Function::New(context, EmptyCallback).ToLocalChecked())
      .Check();
  app->Set(context, V8Str(isolate, "getDetails"),
           v8::Function::New(context, EmptyCallback).ToLocalChecked())
      .Check();
  app->Set(context, V8Str(isolate, "runningState"),
           v8::Function::New(context, EmptyCallback).ToLocalChecked())
      .Check();
  app->Set(context, V8Str(isolate, "InstallState"),
           BuildEnumObject(isolate, context,
                           {{"DISABLED", "disabled"},
                            {"INSTALLED", "installed"},
                            {"NOT_INSTALLED", "not_installed"}}))
      .Check();
  app->Set(context, V8Str(isolate, "RunningState"),
           BuildEnumObject(isolate, context,
                           {{"CANNOT_RUN", "cannot_run"},
                            {"READY_TO_RUN", "ready_to_run"},
                            {"RUNNING", "running"}}))
      .Check();

  // --- chrome.runtime ---
  v8::Local<v8::Object> runtime = v8::Object::New(isolate);
  runtime
      ->Set(context, V8Str(isolate, "connect"),
            v8::Function::New(context, EmptyCallback).ToLocalChecked())
      .Check();
  runtime
      ->Set(context, V8Str(isolate, "sendMessage"),
            v8::Function::New(context, EmptyCallback).ToLocalChecked())
      .Check();
  runtime
      ->Set(context, V8Str(isolate, "OnInstalledReason"),
            BuildEnumObject(
                isolate, context,
                {{"CHROME_UPDATE", "chrome_update"},
                 {"INSTALL", "install"},
                 {"SHARED_MODULE_UPDATE", "shared_module_update"},
                 {"UPDATE", "update"}}))
      .Check();
  runtime
      ->Set(context, V8Str(isolate, "OnRestartRequiredReason"),
            BuildEnumObject(isolate, context,
                            {{"APP_UPDATE", "app_update"},
                             {"OS_UPDATE", "os_update"},
                             {"PERIODIC", "periodic"}}))
      .Check();
  runtime
      ->Set(context, V8Str(isolate, "PlatformArch"),
            BuildEnumObject(isolate, context,
                            {{"ARM", "arm"},
                             {"ARM64", "arm64"},
                             {"MACH_64", "x86-64"},
                             {"MACH_32", "x86-32"},
                             {"X86_64", "x86-64"},
                             {"X86_32", "x86-32"}}))
      .Check();
  runtime
      ->Set(context, V8Str(isolate, "PlatformNaclArch"),
            BuildEnumObject(isolate, context,
                            {{"ARM", "arm"},
                             {"X86_32", "x86-32"},
                             {"X86_64", "x86-64"}}))
      .Check();
  runtime
      ->Set(context, V8Str(isolate, "PlatformOs"),
            BuildEnumObject(isolate, context,
                            {{"ANDROID", "android"},
                             {"CROS", "cros"},
                             {"LINUX", "linux"},
                             {"MAC", "mac"},
                             {"OPENBSD", "openbsd"},
                             {"WIN", "win"}}))
      .Check();
  runtime
      ->Set(
          context, V8Str(isolate, "RequestUpdateCheckStatus"),
          BuildEnumObject(isolate, context,
                          {{"NO_UPDATE", "no_update"},
                           {"THROTTLED", "throttled"},
                           {"UPDATE_AVAILABLE", "update_available"}}))
      .Check();

  // --- Assemble chrome object ---
  chrome->Set(context, V8Str(isolate, "app"), app).Check();
  chrome->Set(context, V8Str(isolate, "runtime"), runtime).Check();

  // Freeze: writable=false, configurable=false
  global
      ->DefineOwnProperty(
          context, chrome_key, chrome,
          static_cast<v8::PropertyAttribute>(v8::ReadOnly | v8::DontDelete))
      .Check();
}

}  // namespace clawser::browser

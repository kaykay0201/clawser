// Copyright 2024 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

// Entry point for clawser_browser.exe — a headless Chromium browser with
// antidetect and API payload capture, controlled via stdin/stdout JSON.

#ifdef UNSAFE_BUFFERS_BUILD
#pragma allow_unsafe_libc_calls
#endif

#include <iostream>
#include <memory>
#include <string>

#include "base/at_exit.h"
#include "base/command_line.h"
#include "base/functional/bind.h"
#include "base/json/json_reader.h"
#include "base/json/json_writer.h"
#include "base/logging.h"
#include "base/process/process.h"
#include "base/rand_util.h"
#include "base/strings/string_number_conversions.h"
#include "base/threading/thread.h"
#include "build/build_config.h"
#include "clawser/browser/browser_controller.h"
#include "clawser/browser/message_handler.h"
#include "clawser/clawser_config.h"
#include "clawser/hardware_profiles.h"
#include "content/public/app/content_main.h"
#include "content/public/common/content_switches.h"
#include "headless/lib/browser/headless_browser_impl.h"
#include "headless/lib/headless_content_main_delegate.h"
#include "headless/public/headless_browser.h"
#include "headless/public/headless_browser_context.h"

#if BUILDFLAG(IS_WIN)
#include <io.h>
#include <windows.h>
#include "content/public/app/sandbox_helper_win.h"
#include "sandbox/win/src/sandbox_types.h"
#endif

namespace clawser::browser {

// Raw stdout handle saved before ContentMain can modify CRT's stdout.
// On Windows, ContentMain calls RouteStdioToConsole which can break
// CRT's fputs/stdout when the process was spawned with piped handles.
// We bypass CRT entirely and use WriteFile on this saved handle.
#if BUILDFLAG(IS_WIN)
static HANDLE g_raw_stdout = INVALID_HANDLE_VALUE;
#endif

void WriteJsonLine(const std::string& json_with_newline) {
#if BUILDFLAG(IS_WIN)
  if (g_raw_stdout != INVALID_HANDLE_VALUE) {
    DWORD written;
    WriteFile(g_raw_stdout, json_with_newline.c_str(),
              static_cast<DWORD>(json_with_newline.size()), &written, nullptr);
    return;
  }
#endif
  fputs(json_with_newline.c_str(), stdout);
  fflush(stdout);
}

namespace {

// Chrome version table — reused from clawser_fetch_impl.cc.
struct ChromeVersionInfo {
  const char* version;
  const char* grease_brand;
  const char* grease_version;
};

// Only claim Chrome 135 — must match our Chromium branch (6998 = Chrome 135).
// Claiming older versions creates a UA ↔ JA3/JA4 mismatch that Akamai and
// other TLS fingerprinters detect (the TLS ClientHello is always Chrome 135
// regardless of what UA string we set).
constexpr ChromeVersionInfo kChromeVersions[] = {
    {"135", "Not-A.Brand", "8"},
};
constexpr size_t kNumChromeVersions = std::size(kChromeVersions);

// Seed struct matching the C API / Rust crate.
struct Seed {
  uint64_t hw_seed;
  uint64_t canvas_seed;
  uint64_t webgl_seed;
  uint64_t audio_seed;
  uint64_t client_rects_seed;
};

Seed GenerateRandomSeed() {
  return {base::RandUint64(), base::RandUint64(), base::RandUint64(),
          base::RandUint64(), base::RandUint64()};
}

uint64_t ParseSeedField(const base::Value::Dict& dict, const char* key) {
  // Rust sends seeds as strings ("12345") for full uint64 precision.
  // Also handle doubles for backwards compatibility.
  if (const std::string* s = dict.FindString(key)) {
    uint64_t val = 0;
    base::StringToUint64(*s, &val);
    return val;
  }
  return static_cast<uint64_t>(dict.FindDouble(key).value_or(0));
}

Seed ParseSeedFromJson(const base::Value::Dict& seed_dict) {
  return {ParseSeedField(seed_dict, "hw_seed"),
          ParseSeedField(seed_dict, "canvas_seed"),
          ParseSeedField(seed_dict, "webgl_seed"),
          ParseSeedField(seed_dict, "audio_seed"),
          ParseSeedField(seed_dict, "client_rects_seed")};
}

// Builds a full ClawserConfig JSON from seed values and applies it
// to the global ClawserConfigManager singleton.
// Reused logic from clawser/fetch/clawser_fetch_impl.cc ApplySeedToConfig.
void ApplySeedToConfig(const Seed& seed) {
  const auto& profiles = GetHardwareProfiles();
  size_t hw_idx = seed.hw_seed % profiles.size();
  size_t ver_idx = (seed.hw_seed >> 32) % kNumChromeVersions;
  const HardwareProfile& hw = profiles[hw_idx];
  const auto& ver = kChromeVersions[ver_idx];

  std::string v = ver.version;
  std::string ua =
      "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 "
      "(KHTML, like Gecko) Chrome/" + v + ".0.0.0 Safari/537.36";

  // Build JSON via base::Value to avoid StringPrintf + PRIu64 issues.
  base::Value::Dict nav_ua_data;
  base::Value::List brands;
  {
    base::Value::Dict b1;
    b1.Set("brand", "Google Chrome");
    b1.Set("version", v);
    brands.Append(std::move(b1));
    base::Value::Dict b2;
    b2.Set("brand", "Chromium");
    b2.Set("version", v);
    brands.Append(std::move(b2));
    base::Value::Dict b3;
    b3.Set("brand", ver.grease_brand);
    b3.Set("version", ver.grease_version);
    brands.Append(std::move(b3));
  }
  nav_ua_data.Set("brands", std::move(brands));
  nav_ua_data.Set("mobile", false);
  nav_ua_data.Set("platform", "Windows");
  nav_ua_data.Set("platform_version", "15.0.0");
  nav_ua_data.Set("architecture", "x86");
  nav_ua_data.Set("model", "");
  nav_ua_data.Set("bitness", "64");

  base::Value::Dict nav;
  nav.Set("user_agent", ua);
  nav.Set("platform", "Win32");
  nav.Set("vendor", "Google Inc.");
  nav.Set("app_version",
           "5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 "
           "(KHTML, like Gecko) Chrome/" + v + ".0.0.0 Safari/537.36");
  nav.Set("language", "en-US");
  base::Value::List langs;
  langs.Append("en-US");
  langs.Append("en");
  nav.Set("languages", std::move(langs));
  nav.Set("hardware_concurrency", hw.hardware_concurrency);
  nav.Set("device_memory", hw.device_memory);
  nav.Set("max_touch_points", 0);
  nav.Set("user_agent_data", std::move(nav_ua_data));

  base::Value::Dict screen;
  screen.Set("width", hw.screen_width);
  screen.Set("height", hw.screen_height);
  screen.Set("avail_width", hw.screen_width);
  screen.Set("avail_height", hw.screen_height - 40);
  screen.Set("color_depth", 24);
  screen.Set("pixel_depth", 24);
  screen.Set("device_pixel_ratio", 1.0);

  base::Value::Dict gpu;
  gpu.Set("vendor", hw.gl_vendor);
  gpu.Set("renderer", hw.gl_renderer);

  base::Value::Dict noise;
  noise.Set("canvas", base::NumberToString(seed.canvas_seed));
  noise.Set("webgl", base::NumberToString(seed.webgl_seed));
  noise.Set("audio", base::NumberToString(seed.audio_seed));
  noise.Set("client_rects", base::NumberToString(seed.client_rects_seed));

  base::Value::Dict media;
  media.Set("audio_inputs", 1);
  media.Set("audio_outputs", 1);
  media.Set("video_inputs", 1);

  base::Value::Dict webrtc;
  webrtc.Set("policy", "disabled");

  base::Value::Dict battery;
  battery.Set("enabled", false);

  base::Value::Dict root;
  root.Set("version", 1);
  root.Set("profile_id",
           "seed-" + base::NumberToString(seed.hw_seed));
  root.Set("navigator", std::move(nav));
  root.Set("screen", std::move(screen));
  root.Set("gpu", std::move(gpu));
  root.Set("noise_seeds", std::move(noise));
  root.Set("timezone", "America/New_York");
  root.Set("locale", "en-US");
  root.Set("media_devices", std::move(media));
  root.Set("webrtc", std::move(webrtc));
  root.Set("battery", std::move(battery));
  root.Set("bluetooth", false);
  root.Set("usb", false);

  std::string json;
  base::JSONWriter::Write(root, &json);
  ClawserConfigManager::GetInstance().ParseJson(json);
}

// Reads the first stdin line synchronously (the init command).
// Must be called BEFORE content::ContentMain blocks.
base::Value::Dict ReadInitCommand() {
  std::string line;
  if (!std::getline(std::cin, line) || line.empty()) {
    LOG(ERROR) << "Failed to read init command from stdin";
    return base::Value::Dict();
  }

  auto parsed = base::JSONReader::ReadAndReturnValueWithError(line);
  if (!parsed.has_value() || !parsed->is_dict()) {
    LOG(ERROR) << "Invalid init JSON: " << line;
    return base::Value::Dict();
  }

  return std::move(parsed->GetDict());
}

// Global state passed from main() to the browser start callback.
struct StartupState {
  Seed seed;
  std::vector<std::string> watch_endpoints;
  int init_id = 0;
};

StartupState g_startup_state;

// The application class — receives the browser start callback.
class ClawserBrowserApp {
 public:
  void OnBrowserStart(headless::HeadlessBrowser* browser) {
    LOG(INFO) << "[clawser] Browser started, creating context...";
    browser_ = browser;

    // Create incognito browser context
    auto* context =
        browser_->CreateBrowserContextBuilder().SetIncognitoMode(true).Build();
    browser_->SetDefaultBrowserContext(context);

    // Initialize browser controller and message handler
    controller_ = std::make_unique<BrowserController>(browser_, context);
    handler_ = std::make_unique<MessageHandler>(controller_.get());

    // Register pre-configured watches
    for (const auto& endpoint : g_startup_state.watch_endpoints) {
      controller_->AddWatch(endpoint);
    }

    // Send init response using base::Value for clean JSON serialization
    const auto& s = g_startup_state.seed;
    base::Value::Dict seed_dict;
    seed_dict.Set("hw_seed", base::NumberToString(s.hw_seed));
    seed_dict.Set("canvas_seed", base::NumberToString(s.canvas_seed));
    seed_dict.Set("webgl_seed", base::NumberToString(s.webgl_seed));
    seed_dict.Set("audio_seed", base::NumberToString(s.audio_seed));
    seed_dict.Set("client_rects_seed",
                  base::NumberToString(s.client_rects_seed));

    base::Value::Dict response;
    response.Set("id", g_startup_state.init_id);
    response.Set("ok", true);
    response.Set("seed", std::move(seed_dict));

    std::string json;
    base::JSONWriter::Write(response, &json);
    json += "\n";
    WriteJsonLine(json);

    // Start stdin read loop on a dedicated thread
    handler_->StartStdinLoop();
  }

 private:
  raw_ptr<headless::HeadlessBrowser> browser_ = nullptr;
  std::unique_ptr<BrowserController> controller_;
  std::unique_ptr<MessageHandler> handler_;
};

// Child process entry — renderer, GPU, utility processes route here.
// Pattern from headless/app/headless_shell.cc:201-211.
void ChildProcessMain(content::ContentMainParams params) {
  headless::HeadlessContentMainDelegate delegate(nullptr);
  params.delegate = &delegate;
  int rc = content::ContentMain(std::move(params));
  base::Process::TerminateCurrentProcessImmediately(rc);
}

}  // namespace

}  // namespace clawser::browser

int main(int argc, const char** argv) {
#if BUILDFLAG(IS_WIN)
  // Save a duplicate of the stdout pipe handle BEFORE ContentMain modifies it.
  // RouteStdioToConsole (headless mode) does freopen("CONOUT$", stdout) which
  // CloseHandle()s the original pipe. DuplicateHandle survives that.
  //
  // We also try _get_osfhandle(1) as fallback — on some Windows configs with
  // STARTF_USESTDHANDLES, GetStdHandle may return the console handle instead
  // of the piped handle set by the parent process.
  {
    HANDLE original = INVALID_HANDLE_VALUE;
    // Try CRT fd 1 first (most reliable for piped handles).
    intptr_t fd1 = _get_osfhandle(1);
    if (fd1 != -1 && fd1 != (intptr_t)INVALID_HANDLE_VALUE) {
      original = (HANDLE)fd1;
    }
    // Fallback to GetStdHandle.
    if (original == INVALID_HANDLE_VALUE) {
      original = GetStdHandle(STD_OUTPUT_HANDLE);
    }
    if (original != INVALID_HANDLE_VALUE) {
      DuplicateHandle(GetCurrentProcess(), original, GetCurrentProcess(),
                      &clawser::browser::g_raw_stdout, 0, FALSE,
                      DUPLICATE_SAME_ACCESS);
    }
  }
#endif

  // Disable stdout buffering for JSON line protocol
  setvbuf(stdout, nullptr, _IONBF, 0);

  content::ContentMainParams params(nullptr);

#if BUILDFLAG(IS_WIN)
  sandbox::SandboxInterfaceInfo sandbox_info = {nullptr};
  content::InitializeSandboxInfo(&sandbox_info);
  params.sandbox_info = &sandbox_info;
  base::CommandLine::Init(0, nullptr);
#else
  params.argc = argc;
  params.argv = argv;
  base::CommandLine::Init(argc, argv);
#endif

  base::CommandLine& command_line =
      *base::CommandLine::ForCurrentProcess();

  // Child process dispatch — if --type is set, this is a renderer/GPU/etc.
  std::string process_type =
      command_line.GetSwitchValueASCII(switches::kProcessType);
  if (!process_type.empty()) {
    clawser::browser::ChildProcessMain(std::move(params));
    return 0;  // Not reached
  }

  // Browser process — read init command from stdin BEFORE blocking
  LOG(INFO) << "[clawser] Browser process starting, reading init...";
  auto init_cmd = clawser::browser::ReadInitCommand();
  int init_id = init_cmd.FindInt("id").value_or(0);

  // Parse or generate seed
  clawser::browser::Seed seed;
  const base::Value::Dict* seed_dict = init_cmd.FindDict("seed");
  if (seed_dict) {
    seed = clawser::browser::ParseSeedFromJson(*seed_dict);
    LOG(INFO) << "[clawser] Using provided seed, hw=" << seed.hw_seed;
  } else {
    seed = clawser::browser::GenerateRandomSeed();
    LOG(INFO) << "[clawser] Generated random seed, hw=" << seed.hw_seed;
  }

  // Parse watch list
  std::vector<std::string> watches;
  const base::Value::List* watch_list = init_cmd.FindList("watch");
  if (watch_list) {
    for (const auto& val : *watch_list) {
      if (val.is_string())
        watches.push_back(val.GetString());
    }
  }

  // Store in global state for the callback
  clawser::browser::g_startup_state.seed = seed;
  clawser::browser::g_startup_state.watch_endpoints = std::move(watches);
  clawser::browser::g_startup_state.init_id = init_id;

  // Apply antidetect profile
  clawser::browser::ApplySeedToConfig(seed);


  // Append headless switches
  command_line.AppendSwitch("headless");
  command_line.AppendSwitch("single-process");
  command_line.AppendSwitch("no-sandbox");
  command_line.AppendSwitch("disable-gpu");
  command_line.AppendSwitch("clawser-browser");

  // Create headless browser with our start callback
  clawser::browser::ClawserBrowserApp app;
  auto browser = std::make_unique<headless::HeadlessBrowserImpl>(
      base::BindOnce(&clawser::browser::ClawserBrowserApp::OnBrowserStart,
                     base::Unretained(&app)));
  headless::HeadlessContentMainDelegate delegate(std::move(browser));
  params.delegate = &delegate;
  return content::ContentMain(std::move(params));
}

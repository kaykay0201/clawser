// Copyright 2024 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

// Entry point for clawser_browser.exe — a headless/headful Chromium browser
// with antidetect and API payload capture, controlled via TCP JSON protocol.
//
// Communication: Rust passes --clawser-port=PORT. The browser binds a TCP
// listener on 127.0.0.1:PORT, accepts one connection, and runs the JSON
// line protocol over that socket. No stdin/stdout pipes — avoids Windows
// CRT heap corruption from piped handles + Chromium's RouteStdioToConsole.

#ifdef UNSAFE_BUFFERS_BUILD
#pragma allow_unsafe_libc_calls
#endif

#include <memory>
#include <string>

#include "base/command_line.h"
#include "base/functional/bind.h"
#include "base/json/json_writer.h"
#include "base/memory/scoped_refptr.h"
#include "base/logging.h"
#include "base/process/process.h"
#include "base/rand_util.h"
#include "base/strings/string_number_conversions.h"
#include "base/strings/string_split.h"
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
#include "net/base/io_buffer.h"
#include "net/base/ip_address.h"
#include "net/base/ip_endpoint.h"
#include "net/base/net_errors.h"
#include "net/log/net_log_source.h"
#include "net/socket/tcp_server_socket.h"
#include "net/socket/stream_socket.h"
#include "net/traffic_annotation/network_traffic_annotation.h"

#if BUILDFLAG(IS_WIN)
#include <windows.h>
#include "content/public/app/sandbox_helper_win.h"
#include "sandbox/win/src/sandbox_types.h"
#endif

namespace clawser::browser {

namespace {

// Chrome 134 — must match our Chromium branch (134.0.6998.0).
constexpr char kChromeVersion[] = "134";
constexpr char kGreaseBrand[] = "Not-A.Brand";
constexpr char kGreaseVersion[] = "8";

struct Seed {
  uint64_t hw_seed = 0;
  uint64_t canvas_seed = 0;
  uint64_t webgl_seed = 0;
  uint64_t audio_seed = 0;
  uint64_t client_rects_seed = 0;
};

Seed GenerateRandomSeed() {
  return {base::RandUint64(), base::RandUint64(), base::RandUint64(),
          base::RandUint64(), base::RandUint64()};
}

void ApplySeedToConfig(const Seed& seed) {
  const auto& profiles = GetHardwareProfiles();
  size_t hw_idx = seed.hw_seed % profiles.size();
  const HardwareProfile& hw = profiles[hw_idx];

  std::string v = kChromeVersion;
  std::string ua =
      "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 "
      "(KHTML, like Gecko) Chrome/" + v + ".0.0.0 Safari/537.36";

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
    b3.Set("brand", kGreaseBrand);
    b3.Set("version", kGreaseVersion);
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
  root.Set("profile_id", "seed-" + base::NumberToString(seed.hw_seed));
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

// Global state passed from main() to the browser start callback.
struct StartupState {
  Seed seed;
  std::vector<std::string> watch_endpoints;
  int port = 0;
};

StartupState g_startup_state;

// The application class — receives the browser start callback.
class ClawserBrowserApp {
 public:
  void OnBrowserStart(headless::HeadlessBrowser* browser) {
    LOG(INFO) << "[clawser] Browser started, creating context...";
    browser_ = browser;

    auto* context =
        browser_->CreateBrowserContextBuilder().SetIncognitoMode(true).Build();
    browser_->SetDefaultBrowserContext(context);

    controller_ = std::make_unique<BrowserController>(browser_, context);
    handler_ = std::make_unique<MessageHandler>(controller_.get());

    for (const auto& endpoint : g_startup_state.watch_endpoints) {
      controller_->AddWatch(endpoint);
    }

    // Start TCP listener on the port Rust told us to use.
    StartTcpListener();
  }

 private:
  void StartTcpListener() {
    server_socket_ = std::make_unique<net::TCPServerSocket>(
        nullptr, net::NetLogSource());
    int result = server_socket_->ListenWithAddressAndPort(
        "127.0.0.1", g_startup_state.port, /*backlog=*/1);
    if (result != net::OK) {
      LOG(ERROR) << "[clawser] Failed to listen on port "
                 << g_startup_state.port << ": " << result;
      browser_->Shutdown();
      return;
    }
    LOG(INFO) << "[clawser] TCP listening on 127.0.0.1:"
              << g_startup_state.port;

    // Accept one connection.
    result = server_socket_->Accept(
        &client_socket_,
        base::BindOnce(&ClawserBrowserApp::OnAccepted,
                       base::Unretained(this)));
    if (result == net::OK) {
      OnAccepted(net::OK);
    }
    // ERR_IO_PENDING = will call OnAccepted later
  }

  void OnAccepted(int result) {
    if (result != net::OK || !client_socket_) {
      LOG(ERROR) << "[clawser] Accept failed: " << result;
      browser_->Shutdown();
      return;
    }
    LOG(INFO) << "[clawser] Client connected";

    // Close server socket — only one connection allowed.
    server_socket_.reset();

    // Send ready signal with seed.
    const auto& s = g_startup_state.seed;
    base::Value::Dict seed_dict;
    seed_dict.Set("hw_seed", base::NumberToString(s.hw_seed));
    seed_dict.Set("canvas_seed", base::NumberToString(s.canvas_seed));
    seed_dict.Set("webgl_seed", base::NumberToString(s.webgl_seed));
    seed_dict.Set("audio_seed", base::NumberToString(s.audio_seed));
    seed_dict.Set("client_rects_seed",
                  base::NumberToString(s.client_rects_seed));

    base::Value::Dict ready;
    ready.Set("ready", true);
    ready.Set("seed", std::move(seed_dict));

    std::string json;
    base::JSONWriter::Write(ready, &json);
    json += "\n";
    TcpWrite(json);

    // Give the TCP socket to MessageHandler for the JSON command loop.
    handler_->StartTcpLoop(std::move(client_socket_));
  }

  void TcpWrite(const std::string& data) {
    if (!client_socket_)
      return;
    auto buf = base::MakeRefCounted<net::StringIOBuffer>(data);
    net::NetworkTrafficAnnotationTag annotation =
        net::DefineNetworkTrafficAnnotation("clawser_tcp", R"(
          semantics { sender: "Clawser Browser" description: "TCP IPC"
            trigger: "Internal" data: "JSON commands" destination: LOCAL }
          policy { cookies_allowed: NO })");
    client_socket_->Write(
        buf.get(), buf->size(),
        base::BindOnce([](int result) {
          if (result < 0)
            LOG(ERROR) << "[clawser] TCP ready write failed: " << result;
        }),
        annotation);
  }

  raw_ptr<headless::HeadlessBrowser> browser_ = nullptr;
  std::unique_ptr<BrowserController> controller_;
  std::unique_ptr<MessageHandler> handler_;
  std::unique_ptr<net::TCPServerSocket> server_socket_;
  std::unique_ptr<net::StreamSocket> client_socket_;
};

void ChildProcessMain(content::ContentMainParams params) {
  headless::HeadlessContentMainDelegate delegate(nullptr);
  params.delegate = &delegate;
  int rc = content::ContentMain(std::move(params));
  base::Process::TerminateCurrentProcessImmediately(rc);
}

}  // namespace

}  // namespace clawser::browser

int main(int argc, const char** argv) {
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

  // Child process dispatch.
  std::string process_type =
      command_line.GetSwitchValueASCII(switches::kProcessType);
  if (!process_type.empty()) {
    clawser::browser::ChildProcessMain(std::move(params));
    return 0;
  }

  // Browser process.
  LOG(INFO) << "[clawser] Browser process starting...";

  // --clawser-port=PORT (required from Rust, TCP communication)
  int port = 0;
  base::StringToInt(
      command_line.GetSwitchValueASCII("clawser-port"), &port);
  if (port <= 0) {
    LOG(ERROR) << "[clawser] --clawser-port is required";
    return 1;
  }
  clawser::browser::g_startup_state.port = port;

  // --clawser-seed=hw,canvas,webgl,audio,rects (optional)
  clawser::browser::Seed seed;
  std::string seed_str =
      command_line.GetSwitchValueASCII("clawser-seed");
  if (!seed_str.empty()) {
    std::vector<std::string> parts = base::SplitString(
        seed_str, ",", base::TRIM_WHITESPACE, base::SPLIT_WANT_NONEMPTY);
    if (parts.size() == 5) {
      base::StringToUint64(parts[0], &seed.hw_seed);
      base::StringToUint64(parts[1], &seed.canvas_seed);
      base::StringToUint64(parts[2], &seed.webgl_seed);
      base::StringToUint64(parts[3], &seed.audio_seed);
      base::StringToUint64(parts[4], &seed.client_rects_seed);
      LOG(INFO) << "[clawser] Using provided seed, hw=" << seed.hw_seed;
    } else {
      seed = clawser::browser::GenerateRandomSeed();
      LOG(WARNING) << "[clawser] Bad --clawser-seed format, using random";
    }
  } else {
    seed = clawser::browser::GenerateRandomSeed();
    LOG(INFO) << "[clawser] Generated random seed, hw=" << seed.hw_seed;
  }
  clawser::browser::g_startup_state.seed = seed;

  // --clawser-watch=endpoint1,endpoint2 (optional)
  std::string watch_str =
      command_line.GetSwitchValueASCII("clawser-watch");
  if (!watch_str.empty()) {
    clawser::browser::g_startup_state.watch_endpoints = base::SplitString(
        watch_str, ",", base::TRIM_WHITESPACE, base::SPLIT_WANT_NONEMPTY);
  }

  // Apply antidetect profile.
  clawser::browser::ApplySeedToConfig(seed);

  // Core switches.
  command_line.AppendSwitch("single-process");
  command_line.AppendSwitch("no-sandbox");
  command_line.AppendSwitch("disable-gpu");
  command_line.AppendSwitch("clawser-browser");

  if (!command_line.HasSwitch("headless")) {
    LOG(INFO) << "[clawser] Running in headful mode";
  }

  clawser::browser::ClawserBrowserApp app;
  auto browser = std::make_unique<headless::HeadlessBrowserImpl>(
      base::BindOnce(&clawser::browser::ClawserBrowserApp::OnBrowserStart,
                     base::Unretained(&app)));
  headless::HeadlessContentMainDelegate delegate(std::move(browser));
  params.delegate = &delegate;
  return content::ContentMain(std::move(params));
}

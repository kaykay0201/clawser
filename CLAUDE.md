# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Repository Overview

This is a **Chromium fork with an integrated antidetect browser module called "clawser"**. The base is the Chromium open-source browser engine (~30M+ lines of C++), with custom fingerprint spoofing and privacy modifications layered on top via the `clawser/` top-level directory. The build system is GN + Ninja and source management uses `depot_tools`.

The fork branch is `master` (remote: `origin/clawser-patched`). The remote is `https://github.com/kaykay0201/clawser.git`. There is no upstream `main` branch tracked locally.

## Build Commands

**IMPORTANT: On this machine, VS BuildTools is installed but not detected by `vswhere`. You MUST set these env vars before any GN or ninja command:**

```bash
export DEPOT_TOOLS_WIN_TOOLCHAIN=0
export GYP_MSVS_OVERRIDE_PATH="C:/Program Files (x86)/Microsoft Visual Studio/2022/BuildTools"
export GYP_MSVS_VERSION=2022
export vs2022_install="C:/Program Files (x86)/Microsoft Visual Studio/2022/BuildTools"
```

**Paths on this machine:**
- `depot_tools`: `C:/depot_tools` (add `/c/depot_tools` to PATH)
- `ninja`: Use `C:/depot_tools/ninja.exe` directly (not autoninja — it goes through a wrapper that fails)
- `gn`: `buildtools/win/gn.exe`
- `clang-cl`: `third_party/llvm-build/Release+Asserts/bin/clang-cl.exe`

```bash
# === Environment setup (run once per shell session) ===
export DEPOT_TOOLS_WIN_TOOLCHAIN=0
export GYP_MSVS_OVERRIDE_PATH="C:/Program Files (x86)/Microsoft Visual Studio/2022/BuildTools"
export GYP_MSVS_VERSION=2022
export vs2022_install="C:/Program Files (x86)/Microsoft Visual Studio/2022/BuildTools"

# === Generate build files (after BUILD.gn changes) ===
buildtools/win/gn.exe gen out/Default   # Dev (component build, fast incremental)
buildtools/win/gn.exe gen out/Release   # Release (standalone DLL for distribution)

# === Build targets (dev) ===
C:/depot_tools/ninja.exe -C out/Default chrome           # Full Chrome (headful + headless + antidetect)
C:/depot_tools/ninja.exe -C out/Default clawser_fetch    # Fetch engine DLL (dev only)
C:/depot_tools/ninja.exe -C out/Default unit_tests       # Unit tests

# === Build standalone DLL for distribution ===
# IMPORTANT: ALWAYS use out/Release for shipping. out/Default produces DLLs
# that depend on base.dll, net.dll, etc. which don't exist outside the build dir.
C:/depot_tools/ninja.exe -C out/Release clawser_fetch    # Standalone ~15MB DLL

# === Build a single file (note the trailing ^) ===
C:/depot_tools/ninja.exe -C out/Default ../../base/logging.cc^

# === Other GN commands ===
buildtools/win/gn.exe refs out/Default path/to/file.cc
buildtools/win/gn.exe desc out/Default //clawser deps
buildtools/win/gn.exe args out/Default

# Sync third-party dependencies (after pulling or switching branches)
gclient sync
```

**Convenience build script** (handles env vars, Ctrl+C safety, corruption detection):
```bash
./build.sh                        # Build chrome, 70% cores
./build.sh chrome                 # Build chrome (explicit)
./build.sh -j32 chrome            # Full cores
./build.sh --clean                # Force gn gen + build
./build.sh --kill                 # Gracefully stop a running build
./build.sh --status               # Check if build is running
```

**NEVER kill ninja mid-build** (taskkill, Ctrl+C without the build script, closing terminal). Interrupted ninja corrupts `.ninja_deps`/`.ninja_log`, forcing a full 16k+ step rebuild. Use `./build.sh --kill` or let it finish. If already corrupted (ninja says "premature end of file; recovering"): re-run `gn gen out/Default` to regenerate. Ninja will do a full rebuild but dependency tracking will be restored.

**NEVER start two ninja builds simultaneously** on the same `out/` directory. This corrupts the dependency database and build artifacts, causing heap corruption crashes (`0xc0000374`) from DLL/exe version mismatch.

Current build config (`out/Default/args.gn`): `is_debug=false`, `is_component_build=true`, `enable_nacl=false`, `symbol_level=1`, `blink_symbol_level=0`.

Common args: `is_debug=true` (for debug builds), `is_component_build=true` (faster incremental builds).

## Running with Clawser

```bash
# Launch headful Chrome with antidetect
out/Default/chrome --clawser-config=/path/to/profile.json

# Launch headless Chrome with antidetect + CDP
out/Default/chrome --headless=new --clawser-config=/path/to/profile.json \
  --remote-debugging-port=9222 --remote-allow-origins=*

# Config path MUST be an absolute Windows path (not MSYS /e/... format)
# Use: cygpath -w "$(pwd)/out/Default/test_profile.json"
```

The `--clawser-config` flag is defined in `chrome/common/chrome_switches.h`. The config is a JSON file parsed by `ClawserConfigManager::LoadFromFile()`. If no config is provided, clawser is inactive and Chrome behaves normally. Both headful and headless modes use identical antidetect code paths — `--headless=new` only skips GPU compositing.

## Testing

```bash
# Build and run unit tests
autoninja -C out/Default unit_tests
out/Default/unit_tests --gtest_filter="ClassName.TestName"

# Auto-find and run tests for a source file
python3 tools/autotest.py -C out/Default path/to/foo_unittest.cc

# Find which test target builds a file
gn refs out/Default --testonly=true --type=executable --all path/to/file.cc

# Browser tests (heavier, launches real browser)
autoninja -C out/Default browser_tests
out/Default/browser_tests --gtest_filter="BrowserTestName.*"
```

Test file conventions: `_unittest.cc` for unit tests, `_browsertest.cc` for browser tests.

## Code Formatting and Style

```bash
# Format changed lines (C++, Java, Python, etc.)
git cl format

# Format a single C++ file
clang-format -i path/to/file.cc
```

- C++ follows the [Google C++ Style Guide](https://google.github.io/styleguide/cppguide.html) with Chromium-specific exceptions in `styleguide/c++/c++.md`.
- Targets C++20. See `styleguide/c++/c++-features.md` for allowed/disallowed modern features.
- Blink code (`third_party/blink/`) uses [Blink style](styleguide/c++/blink-c++.md).
- Test-only functions must use the `ForTesting` suffix (enforced by presubmit).

## Clawser Antidetect Module — Architecture

### Overview

The `clawser/` directory is a self-contained `static_library` (14 header/implementation pairs) that all spoofing logic flows through. It depends only on `//base` and `//third_party/icu` (plus `//third_party/khronos:khronos_headers` as a public config for WebGL types). The rest of the codebase conditionally calls into clawser via the singleton `ClawserConfigManager::GetInstance()`, checking `.IsLoaded()` before applying overrides. When no config is loaded, all code paths fall through to stock Chromium behavior.

The `//clawser` target is referenced in 27+ `BUILD.gn` files across `base/`, `cc/`, `chrome/`, `content/`, `gpu/`, `net/`, `third_party/blink/`, and `v8/`.

### Config Loading Flow

1. **Browser process**: `chrome/browser/chrome_browser_main.cc` reads `--clawser-config`, calls `ClawserConfigManager::LoadFromFile(path)`, then re-applies ICU timezone via `icu::TimeZone::adoptDefault()` (ICU is initialized before config loads).
2. **Switch propagation**: `chrome/browser/chrome_content_browser_client.cc` propagates the `--clawser-config` flag to child (renderer) processes.
3. **Renderer process**: `content/renderer/render_thread_impl.cc` loads the same config file in each renderer, re-applies ICU timezone, and disables `WebRuntimeFeatures::AutomationControlled` in `content/child/runtime_features.cc`.
4. **Language fix**: `third_party/blink/renderer/core/frame/navigator_language.cc` skips Chrome's `ReduceAcceptLanguage` feature when clawser is active, preserving the full language list from config.
5. **Fallback**: If GPU/screen/navigator fields are empty in the JSON, `hardware_profiles.cc` selects a random plausible hardware profile.

### Config JSON Structure (`ClawserConfig`)

Defined in `clawser/clawser_config.h`. Top-level keys:

| Key | Purpose |
|-----|---------|
| `profile_id` | Unique identifier for this fingerprint profile |
| `navigator` | User-agent, platform, vendor, languages, hardware_concurrency, device_memory, max_touch_points, user_agent_data (Client Hints brands) |
| `screen` | width, height, avail_width, avail_height, color_depth, pixel_depth, device_pixel_ratio |
| `gpu` | vendor, renderer, webgl_params (max_texture_size, extensions, viewport dims, etc.) |
| `noise_seeds` | Deterministic seeds for canvas, webgl, audio, client_rects noise (auto-randomized if 0) |
| `timezone` / `locale` | ICU timezone and locale overrides |
| `fonts` | Whitelist of allowed font names |
| `media_devices` | Fake counts for audio_inputs, audio_outputs, video_inputs |
| `speech_voices` | Fake speech synthesis voice list |
| `webrtc` | policy (`disabled`/`proxy_only`/`spoofed`), fake_local_ip |
| `battery` / `bluetooth` / `usb` | Enable/disable flags for these APIs |

### Integration Points by Layer

**Network layer (`net/`)** — 10+ files:
- TLS cipher + signature algorithm ordering fixed to Chrome 135 (JA3/JA4 match) (`net/socket/ssl_client_socket_impl.cc`)
- HTTP/2 fingerprint randomization: pseudo-header ordering, SETTINGS frame values, WINDOW_UPDATE deltas, GREASE (`net/spdy/spdy_http_utils.cc`, `net/spdy/spdy_session.cc`)
- Socket pool tuning: 32 max sockets/pool, 6 max sockets/group (`net/socket/client_socket_pool_manager.cc`)
- DNS control: prevent direct DNS, force DNS-over-proxy (`net/dns/host_resolver_manager.cc`, `net/dns/dns_client.cc`)
- Accept-Language header spoofing (`net/url_request/url_request_http_job.cc`)

**Content layer (`content/`)** — 10+ files:
- User-Agent string override (`content/common/user_agent.cc`)
- Client Hints spoofing: `Sec-CH-UA`, `Sec-CH-UA-Platform`, architecture, bitness (`content/browser/client_hints/client_hints.cc`)
- Font list filtering on Windows (`content/common/font_list_win.cc`)
- Back/forward cache limited to 1 entry (`content/browser/renderer_host/back_forward_cache_impl.cc`)
- Max renderer processes capped to 4 (`content/browser/renderer_host/render_process_host_impl.cc`)

**Blink/JS APIs (`third_party/blink/`)** — 40+ files:
- `navigator.*` properties: userAgent, platform, vendor, appVersion, languages, hardwareConcurrency, deviceMemory, maxTouchPoints
- `screen.*` properties: width, height, availWidth, availHeight, colorDepth, pixelDepth
- `window.devicePixelRatio`, `window.innerWidth/Height`
- Canvas fingerprint noise on `getImageData()` (`modules/canvas/canvas2d/base_rendering_context_2d.cc`)
- WebGL parameter spoofing and `readPixels()` noise (`modules/webgl/webgl_rendering_context_base.cc`)
- Audio fingerprint noise on `AudioBuffer` and `OfflineAudioContext` (`modules/webaudio/`)
- `Element.getClientRects()` / `getBoundingClientRect()` noise (`core/dom/element.cc`)
- Media device enumeration spoofing (`modules/mediastream/media_devices.cc`)
- WebRTC blocking/spoofing (`modules/peerconnection/rtc_peer_connection.cc`)
- Font cache control (`platform/fonts/font_cache.cc`)
- Timezone spoofing via Date objects (`platform/wtf/date_math.cc`)
- CSS media query values match spoofed screen dimensions (`core/css/media_values.cc`)
- Battery, Bluetooth, USB API gating (`modules/battery/`, `modules/bluetooth/`, `modules/webusb/`)
- Speech synthesis voice list spoofing (`modules/speech/speech_synthesis.cc`)

**GPU layer (`gpu/`)** — 4 files:
- `gpu/config/gpu_info.cc` — `ApplyClawserOverrides()` spoofs vendor/device IDs, driver info, GL strings
- `gpu/command_buffer/service/feature_info.cc` — GL renderer string replacement

**Graphics (`cc/`)** — 2 files:
- Tile size reduced to 128x128 and memory policy to 16MB when clawser active (`cc/trees/layer_tree_settings.cc`, `cc/tiles/image_decode_cache_utils.cc`)

**Base layer** — 2 files:
- ICU timezone/locale override (`base/i18n/icu_util.cc`, `base/i18n/rtl.cc`)

### Clawser File Organization

Files follow a consistent naming convention:

| Pattern | Files | Purpose |
|---------|-------|---------|
| `*_spoof.h/.cc` | navigator, screen, webgl, webrtc, gpu_info, font, dns, timezone, media_devices | Getter functions that return spoofed values from config |
| `*_noise.h/.cc` | canvas, audio, client_rects | Apply deterministic noise to raw data buffers |
| `clawser_config.h/.cc` | Config struct + singleton manager | JSON parsing, config storage |
| `hardware_profiles.h/.cc` | 97 realistic GPU/screen profiles | Fallback when config fields are empty — covers NVIDIA RTX 30/40/50, AMD RX 6000/7000/9000, Intel UHD/Iris/Arc across common resolutions. The Rust crate (`clawser-browser/src/profiles.rs`) has 100 profiles with full device configs from Steam Hardware Survey data |

### Key Design Patterns in Clawser

- **Singleton config**: All integration points call `ClawserConfigManager::GetInstance().IsLoaded()` as a guard — no #ifdefs, no build flags. The module is always compiled in.
- **Deterministic noise**: Canvas, WebGL, audio, and client rects use seeded noise (from `noise_seeds` in config) via a `Xorshift128Plus` PRNG (defined inline in `canvas_noise.h`). Same seed always produces the same fingerprint, but different profiles differ.
- **Per-process-once randomization**: HTTP/2 and TLS parameters use `static const` variables initialized with `base::RandGenerator()` so they're random but consistent within a process lifetime. This covers pseudo-header ordering (3 variants), SETTINGS frame profiles (4 variants), and WINDOW_UPDATE deltas (3 variants).
- **Graceful fallback**: When config fields are empty/zero, either stock Chromium behavior is used or `hardware_profiles.cc` selects a random plausible profile via `SelectRandomProfile()`.

### Adding New Spoofing Hooks

1. Add any new config fields to `ClawserConfig` struct in `clawser/clawser_config.h` and parse them in `clawser_config.cc`.
2. Create a new spoof helper (e.g., `clawser/new_spoof.h/.cc`) if the logic is non-trivial. Follow the naming convention: `*_spoof` for value overrides, `*_noise` for data buffer perturbation. Add it to `clawser/BUILD.gn`.
3. At the integration site, include the relevant clawser header, guard with `ClawserConfigManager::GetInstance().IsLoaded()`, and apply the override.
4. Add the consuming target to `clawser/BUILD.gn`'s `visibility` list.
5. For noise functions: use `Xorshift128Plus` with the appropriate seed from `config.noise_seeds` so fingerprints are deterministic per-profile.

## Clawser Browser — Rust Crate (`clawser-browser/`)

An async Rust crate that spawns `chrome.exe` with `--clawser-config` and controls it via CDP (Chrome DevTools Protocol) over WebSocket. Supports both headful and headless modes with full antidetect.

### Build & Run

```bash
# Build chrome.exe first (the browser engine)
./build.sh chrome

# Build and run Rust examples
CLAWSER_CHROME_PATH=out/Default/chrome.exe cargo run --manifest-path clawser-browser/Cargo.toml --example smoke_test
CLAWSER_CHROME_PATH=out/Default/chrome.exe cargo run --manifest-path clawser-browser/Cargo.toml --example youtube_test
CLAWSER_CHROME_PATH=out/Default/chrome.exe cargo run --manifest-path clawser-browser/Cargo.toml --example rotate_test
CLAWSER_CHROME_PATH=out/Default/chrome.exe cargo run --manifest-path clawser-browser/Cargo.toml --example input_test
```

### Architecture

```
Rust user code → clawser-browser crate → CDP WebSocket → chrome.exe
                   (tokio async)           (tungstenite)    (multi-process)
                                                              ↓
                                                     Full Chromium browser
                                                       + V8/Blink + net::
                                                       + all antidetect patches
                                                       + --clawser-config=profile.json
```

### Rust Crate Files (`clawser-browser/src/`)

| File | Purpose |
|------|---------|
| `lib.rs` | Public API: `Browser`, `Page`, `BrowserBuilder`. Async CDP methods for navigate, js, click, type, scroll, screenshot, cookies |
| `process.rs` | Spawns `chrome.exe` with CDP port, waits for readiness via `/json/version`, gets page WebSocket URL |
| `protocol.rs` | CDP WebSocket send/recv via `tokio-tungstenite`. Matches response IDs, handles events |
| `profiles.rs` | 100 static device profiles (Steam Hardware Survey data) + deterministic seed generation + config JSON serialization |
| `types.rs` | Data types: `Cookie`, `Response`, `HwProfile` |

### Rust Crate API

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Random profile from 100 built-in devices × unlimited seeds
    let browser = Browser::builder()
        .headful()          // or .headless()
        .random()           // or .profile(42, 12345) for deterministic
        .build().await?;    // or .config("path/to/profile.json")

    let page = browser.navigate("https://target.com").await?;

    // Human behavior simulation (Akamai Layer 3)
    page.human_idle(2000).await?;     // Random mouse bezier curves
    page.scroll(300).await?;          // Momentum scroll
    page.click(600.0, 300.0).await?;  // Move + click with hold
    page.type_text("query").await?;   // Variable dwell + gap timing

    // JS evaluation
    let title = page.js("document.title").await?;

    // Cookies persist across sessions (same profile = same user-data-dir)
    let cookies = browser.cookies("https://target.com").await?;

    // Screenshot
    let png = browser.screenshot().await?;

    browser.shutdown().await?;
    Ok(())
}
```

### Key Design Decisions

- **chrome.exe, not custom exe**: Uses the same `chrome.exe` binary for both headful and headless. All antidetect patches are in chrome.exe via `--clawser-config`. No separate `clawser_browser.exe` needed.
- **CDP over WebSocket**: Standard Chrome DevTools Protocol — same as Chrome DevTools uses. `tokio-tungstenite` for async WebSocket, `reqwest` for CDP endpoint polling.
- **Profile rotation**: 100 realistic hardware profiles baked into the binary as static data. `profiles::generate_config_json(index, seed)` creates a complete config deterministically. Same (index, seed) = same fingerprint always.
- **Cookie persistence**: `user-data-dir` is tied to profile ID, not CDP port. Same profile across sessions = Chrome auto-persists cookies including `_abck`.
- **Async native**: All methods are `async`. Uses `tokio::sync::Mutex`, `tokio::time::sleep`, `tokio::process::Command`. Compatible with `tokio::spawn`, `tokio::join!`, `tokio::select!`.

## Debugging

```bash
# Run Chrome with verbose logging
out/Default/chrome --enable-logging=stderr --v=1

# Run in single-process mode (easier to debug, not representative of production)
out/Default/chrome --single-process

# Attach debugger to a renderer (find PID from chrome://system or task manager)
# Then attach your debugger to that PID

# Check clawser config loading (look for "Clawser config loaded" in log output)
out/Default/chrome --clawser-config=profile.json --enable-logging=stderr 2>&1 | grep -i clawser
```

## Upstream Chromium Architecture

### Multi-Process Model

Chromium runs as multiple cooperating processes: browser (main), renderer (per-site), GPU, utility, and network service. Processes are sandboxed for security.

### Directory Layering (critical to understand)

Dependencies flow downward — lower layers cannot depend on higher layers:

| Layer | Directory | Purpose |
|-------|-----------|---------|
| Foundation | `base/` | Cross-platform primitives (threading, memory, strings, logging) |
| Platform | `net/`, `gpu/`, `media/`, `ui/`, `mojo/` | Networking, graphics, media, UI toolkit, IPC |
| Web Platform | `content/` | Multi-process browser core, Blink integration, web APIs. No Chrome-specific features |
| Components | `components/` | Shared subsystems (autofill, sync, passwords) reusable across products |
| Product | `chrome/` | Chrome browser application (UI, extensions, settings). Top of the dependency graph |
| Embedders | `android_webview/`, `ios/` | Platform-specific product embeddings |

**Key rule**: `content/` contains pure web platform code. Chrome-specific features live in `chrome/` or `components/`. The `clawser/` module intentionally breaks this layering — it is referenced from `base/`, `content/`, `net/`, `gpu/`, `cc/`, and `third_party/blink/` because fingerprint spoofing must hook into every layer.

### IPC: Mojo

Inter-process communication uses Mojo. Interfaces defined in `.mojom` files generate C++ bindings:
- `mojo::Remote<Interface>` — sends messages (client side)
- `mojo::Receiver<Interface>` — receives messages (implementation side)
- Service definitions live in `services/`

### Build System: GN + Ninja

- `BUILD.gn` files define targets throughout the tree
- `.gn` (root) configures the GN build
- `.gni` files are imported for shared build configuration
- `gn gen` generates Ninja files; `autoninja` executes the build

## Important Conventions

- `DEPS` (root) manages third-party dependencies via `gclient sync`
- `include_rules` in `DEPS` files enforce layering at build time
- `DIR_METADATA` files track component ownership for bug routing
- `WATCHLISTS` auto-CCs reviewers based on file patterns
- Test support code goes in `test/` subdirectories, not alongside production code

## Gotchas & Warnings

- **ICU timezone must be re-applied after config load**: `InitializeICU()` runs before `LoadFromFile()`. Both `chrome_browser_main.cc` and `render_thread_impl.cc` call `icu::TimeZone::adoptDefault()` after loading config. If this is missing, `Intl.DateTimeFormat().resolvedOptions().timeZone` returns the real timezone instead of the spoofed one.
- **ReduceAcceptLanguage truncates languages**: Chrome's `kReduceAcceptLanguage` feature (on by default) truncates `navigator.languages` to a single entry. `navigator_language.cc` skips this when clawser is active. If languages show only one entry, check this guard.
- **Renderer config loading fails silently**: `content/renderer/render_thread_impl.cc` ignores the return value of `LoadFromFile()`. If the config file can't be read from the renderer process (sandboxing, wrong path), all Blink-level spoofing silently doesn't apply. The browser process logs success/failure but the renderer does not.
- **Hardcoded switch string in renderer**: The renderer uses `"clawser-config"` as a hardcoded string instead of `switches::kClawserConfig`. If the switch name changes, the renderer will silently stop loading configs.
- **Windows paths**: Config path goes through `GetSwitchValueASCII()` → `FilePath::FromUTF8Unsafe()`. Non-ASCII paths or UNC paths may fail silently. Use simple ASCII paths.
- **Must rebuild after BUILD.gn changes**: Chromium's GN build doesn't auto-regenerate. After editing any `BUILD.gn`, run `gn gen out/Default` then `autoninja -C out/Default chrome`. Just running autoninja alone won't pick up new targets or dependency changes.
- **Component build produces DLLs**: With `is_component_build=true`, each GN target becomes a separate DLL on Windows. This means faster incremental builds but different runtime behavior than the monolithic release build. If a spoofing hook appears to not work, verify it's linked into the correct DLL.

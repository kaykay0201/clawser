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

# === Generate build files (after BUILD.gn changes) ===
buildtools/win/gn.exe gen out/Default

# === Build targets ===
C:/depot_tools/ninja.exe -C out/Default chrome           # Full Chrome
C:/depot_tools/ninja.exe -C out/Default clawser_fetch    # Fetch engine DLL only
C:/depot_tools/ninja.exe -C out/Default unit_tests       # Unit tests

# === Build a single file (note the trailing ^) ===
C:/depot_tools/ninja.exe -C out/Default ../../base/logging.cc^

# === Other GN commands ===
buildtools/win/gn.exe refs out/Default path/to/file.cc
buildtools/win/gn.exe desc out/Default //clawser deps
buildtools/win/gn.exe args out/Default

# Sync third-party dependencies (after pulling or switching branches)
gclient sync
```

**If build.ninja gets corrupted** (ninja says "premature end of file; recovering"): re-run `gn gen out/Default` with the env vars above to regenerate it.

Current build config (`out/Default/args.gn`): `is_debug=false`, `is_component_build=true`, `enable_nacl=false`, `symbol_level=1`, `blink_symbol_level=0`.

Common args: `is_debug=true` (for debug builds), `is_component_build=true` (faster incremental builds).

## Running with Clawser

```bash
# Launch Chrome with a clawser fingerprint profile
out/Default/chrome --clawser-config=/path/to/profile.json
```

The `--clawser-config` flag is defined in `chrome/common/chrome_switches.h`. The config is a JSON file parsed by `ClawserConfigManager::LoadFromFile()`. If no config is provided, clawser is inactive and Chrome behaves normally.

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

1. **Browser process**: `chrome/browser/chrome_browser_main.cc` reads `--clawser-config`, calls `ClawserConfigManager::LoadFromFile(path)`.
2. **Switch propagation**: `chrome/browser/chrome_content_browser_client.cc` propagates the `--clawser-config` flag to child (renderer) processes.
3. **Renderer process**: `content/renderer/render_thread_impl.cc` loads the same config file in each renderer. Also disables `WebRuntimeFeatures::AutomationControlled` in `content/child/runtime_features.cc`.
4. **Fallback**: If GPU/screen/navigator fields are empty in the JSON, `hardware_profiles.cc` selects a random plausible hardware profile.

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
- HTTP/2 fingerprint randomization: pseudo-header ordering, SETTINGS frame values, WINDOW_UPDATE deltas, GREASE (`net/spdy/spdy_http_utils.cc`, `net/spdy/spdy_session.cc`)
- Socket pool tuning: 32 max sockets/pool, 6 max sockets/group (`net/socket/client_socket_pool_manager.cc`)
- DNS control: prevent direct DNS, force DNS-over-proxy (`net/dns/host_resolver_manager.cc`, `net/dns/dns_client.cc`)
- Accept-Language header spoofing (`net/url_request/url_request_http_job.cc`)

**Content layer (`content/`)** — 10+ files:
- User-Agent string override (`content/common/user_agent.cc`)
- Client Hints spoofing (`content/browser/client_hints/client_hints.cc`)
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
| `hardware_profiles.h/.cc` | 80+ realistic GPU/screen profiles | Fallback when config fields are empty — covers NVIDIA RTX 30/40/50, AMD RX 6000/7000/9000, Intel UHD/Iris/Arc across common resolutions |

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

- **Renderer config loading fails silently**: `content/renderer/render_thread_impl.cc` ignores the return value of `LoadFromFile()`. If the config file can't be read from the renderer process (sandboxing, wrong path), all Blink-level spoofing silently doesn't apply. The browser process logs success/failure but the renderer does not.
- **Hardcoded switch string in renderer**: The renderer uses `"clawser-config"` as a hardcoded string instead of `switches::kClawserConfig`. If the switch name changes, the renderer will silently stop loading configs.
- **Windows paths**: Config path goes through `GetSwitchValueASCII()` → `FilePath::FromUTF8Unsafe()`. Non-ASCII paths or UNC paths may fail silently. Use simple ASCII paths.
- **Must rebuild after BUILD.gn changes**: Chromium's GN build doesn't auto-regenerate. After editing any `BUILD.gn`, run `gn gen out/Default` then `autoninja -C out/Default chrome`. Just running autoninja alone won't pick up new targets or dependency changes.
- **Component build produces DLLs**: With `is_component_build=true`, each GN target becomes a separate DLL on Windows. This means faster incremental builds but different runtime behavior than the monolithic release build. If a spoofing hook appears to not work, verify it's linked into the correct DLL.

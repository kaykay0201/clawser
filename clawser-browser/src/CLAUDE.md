# clawser-browser/src/

## Purpose

Implementation of the `clawser-browser` crate: browser control (`lib.rs`), profile/fingerprint generation (`profile.rs`), and browser-less HTTP impersonation (`client.rs`). Three files, all `pub`-exported from `lib.rs`.

## Structure

lib.rs      — Public API: `Browser`, `Page`, `BrowserBuilder`, `Result` alias; built-in `HumanSimulator` auto-started per `Page`; re-exports `wreq`, `wreq::header`, profile helpers, `HttpClient`
profile.rs  — 100 static `HwProfile` entries (Steam Hardware Survey data); `generate_config_json(index, seed)` produces the JSON consumed by `chrome.exe --clawser-config`; `random_profile()`, `write_config_file()`; Xorshift64 PRNG for deterministic fingerprint derivation; Windows path canonicalization for temp profile files
client.rs   — `HttpClient` + `HttpClientBuilder` — `wreq::Client` preconfigured with `wreq_util::Emulation` targeting Chrome 134 on Windows for JA3/JA4 + HTTP/2 + default-header parity without launching a browser

## Conventions

- **Three modules, no submodules.** Keep additions flat — if a new concern doesn't fit in one of these files, consider whether it belongs in the crate at all before adding a fourth module.
- **Deterministic where it matters.** Anything that affects the observable fingerprint (profile selection, noise seeds, user-data-dir name) must be a pure function of `(index, seed)`. Random helpers are fine for the *choice* of seed, but reuse must reproduce identically.
- **Static profile data.** `PROFILES` is a `const` array of `HwProfile`. Adding/removing entries changes the `(index, seed)` → fingerprint mapping for everyone — treat as a breaking change, bump the crate minor version.
- **`CHROME_MAJOR` / `CHROME_FULL` / `GREASE_BRAND` live at the top of `profile.rs`.** These must match the C++ side (`clawser/fetch/clawser_fetch_impl.cc`, `clawser/browser/clawser_browser_main.cc`) and `chrome/VERSION`. Whenever you bump one, bump all four in the same commit.
- **`Browser::new_page()` auto-starts the `HumanSimulator`** (background tokio task, Bezier-curve mouse moves + scrolls, 125 Hz polling, 60/25/15 move/scroll/idle mix). It stops via `tokio::sync::watch` on `Page` drop. Don't add a second simulator — if you want different behavior, parameterize the existing one.

## Key Patterns

- JSON config shape: `profile.rs::generate_config_json()` — reference for every field the C++ `ClawserConfig` parser expects. If you add a field on the C++ side, add it here.
- Antidetect flag set: `lib.rs::BrowserBuilder::build()` — reference for which chromiumoxide defaults are stripped (`disable_default_args`) and which must stay (sandbox, features).
- Chrome 134 emulation: `client.rs::HttpClient::new()` — reference for `wreq_util::Emulation` configuration.

## Dependencies

- Imports from: `chromiumoxide` (browser, CDP commands, `Page`), `tokio` (runtime, spawn, watch), `futures_util::StreamExt`, `wreq` + `wreq_util` (HTTP impersonation), `serde`/`serde_json` (config JSON)
- Exported to: `clawser-browser` crate public API (via `pub use` in `lib.rs`)

## Gotchas

- **Windows path canonicalization in `profile.rs`.** The config path written to disk must be an absolute Windows path (not MSYS `/e/...`). `write_config_file()` canonicalizes; callers who write their own config files must do the same or `--clawser-config` will silently fail to load in the renderer.
- **`AtomicU64` counters are process-scoped.** Anything that uses them for uniqueness resets across process restarts — don't rely on them for persistence keys.
- **`HttpClient` and `Browser` don't share cookies.** They are separate stacks (wreq vs. Chromium net::). If a flow needs both, you have to move cookies manually.

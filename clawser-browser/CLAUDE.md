# clawser-browser/

## Purpose

Published Rust crate (`clawser-browser`, v0.2.0, MIT) — async wrapper around `chromiumoxide` 0.9 that launches the forked `chrome.exe` with `--clawser-config` and drives it over CDP. Also ships `HttpClient`, a browser-less fingerprint impersonator built on `wreq` (BoringSSL). Consumers include internal tooling and external users via crates.io.

## Structure

Cargo.toml       — Package metadata (v0.2.0, MIT); deps: `chromiumoxide` 0.9 (default-features off), `tokio` full, `serde`, `serde_json`, `futures-util`, `reqwest` (rustls+socks), `wreq` 6.0.0-rc.28 (cookies+json+gzip+brotli+deflate+socks), `wreq-util` 3.0.0-rc.10 (emulation)
Cargo.lock       — Lockfile (committed — this is a binary-behavior-sensitive crate)
README.md        — Crate landing page for docs.rs + GitHub
LICENSE          — MIT text
.env             — Local-only test credentials (proxy key, etc.) — gitignored
.gitignore       — Excludes `target/`, `.env`, test artifacts
src/             — Crate source: `lib.rs`, `profile.rs`, `client.rs`
examples/        — Runnable examples (`cargo run --example <name>`), each demonstrating one use case
target/          — Cargo build output (gitignored)

## Conventions

- **Env for Chrome path.** Examples read `CLAWSER_CHROME_PATH` to locate `chrome.exe`. Don't hardcode paths in examples; keep them portable.
- **`disable_default_args()` is mandatory.** chromiumoxide injects 24 default flags including `--enable-automation`, which sets `navigator.webdriver = true` and breaks antidetect. The `BrowserBuilder` in `lib.rs` calls this — do not remove.
- **Version sync:** `src/profile.rs::CHROME_MAJOR`/`CHROME_FULL`/`GREASE_BRAND` must match `chrome/VERSION`, `clawser/fetch/clawser_fetch_impl.cc`, and `clawser/browser/clawser_browser_main.cc`. A mismatch between claimed UA version and the binary's actual JS engine features = instant Akamai block.
- **Published crate discipline.** Version bumps go through the workflow in `reference_publish_workflow` memory (build DLL → GitHub release → bump versions → `cargo publish`). Do not publish from a dirty tree or with lockfile unchanged.
- **MSRV-agnostic, edition 2021.** No `rust-version` pin here; keep changes compatible with current stable.

## Key Patterns

- Browser builder with antidetect defaults: `src/lib.rs::BrowserBuilder` — reference for how headful/headless, profile selection, user-data-dir, proxy, and extra args compose
- Deterministic profile: `src/profile.rs::generate_config_json(index, seed)` — reference for how `(index, seed)` → full JSON config that matches a real device
- Browser-less fingerprint: `src/client.rs::HttpClient` — reference for `wreq` + `wreq-util` Emulation when launching a full Chrome is too heavy

## Dependencies

- Imports from: all deps listed in `Cargo.toml` (chromiumoxide, tokio, wreq, reqwest, serde)
- Exports to: downstream users via crates.io (`clawser-browser = "0.2"`); also the `clawser-fetch` ecosystem shares profile data conceptually (100 profiles here vs. 97 on the C++ side)
- Runtime requirement: a built `chrome.exe` from this fork (`out/Release/chrome.exe` or equivalent), located via `CLAWSER_CHROME_PATH` env or explicit `BrowserBuilder::chrome_path()`

## Gotchas

- **Cookie persistence relies on a stable `user-data-dir`.** Default is `{chrome_exe_dir}/clawser_profiles/clawser_{index}_{seed}/` — next to `chrome.exe`, not in `%TEMP%`, so cookies survive across sessions. If you override `user_data_dir`, persistence becomes your problem.
- **`page.navigate()` vs `page.goto()` differ.** `navigate()` is fire-and-forget (good for SPAs where `load` may never fire); `goto()` waits for the load event. Picking the wrong one causes either hangs or "too early to evaluate" errors.
- **Build the DLL before publishing.** `cargo publish` won't catch a stale `clawser_fetch.dll` sibling — the publish workflow requires a fresh `out/Release` build first, even though the DLL itself isn't in this crate's package.

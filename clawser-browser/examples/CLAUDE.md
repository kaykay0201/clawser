# clawser-browser/examples/

## Purpose

Runnable `cargo --example` programs that double as the crate's smoke tests and end-to-end benchmarks. Each file targets one scenario — don't add general helpers here; if logic is reused, promote it to `src/`.

## Structure

smoke_test.rs   — Minimal launch + navigator/screen/UA sanity check (`webdriver`, `timeZone`, `hardwareConcurrency`, `screen.*`, `userAgent`); keeps the browser open until Ctrl+C
youtube_test.rs — Launch + navigate to youtube.com + print title/URL; quick real-site verification
game_test.rs    — WebSocket interception via CDP (`Network.webSocketCreated`/`Closed`); hits `GAME_API_URL` (defaults to abb1211.com/endpoint/play) to confirm the browser sees the API traffic
tls_check.rs    — TLS/HTTP2 fingerprint comparison vs. stock Chrome (loads `clawser-browser/.env` then `.env` for credentials)
batch_test.rs   — Parallel-browser benchmark: 3 browsers × 10 rounds with proxy rotation (5 s cooldown, 30 s browser lifetime); hardcoded proxy key + 30-user list for load testing

## Conventions

- **`CLAWSER_CHROME_PATH` env var sets the chrome.exe location.** Every example relies on it — don't hardcode paths.
- **Examples are disposable.** They exist to demonstrate and stress-test, not to be reliable CLIs. Panic (`.expect()`, `.unwrap()`) is fine; don't add `anyhow`/`thiserror` plumbing.
- **Ctrl+C loops.** Most examples end with `loop { tokio::time::sleep(60s).await }` so the browser stays open for manual inspection. Keep this pattern — it makes debugging easier than auto-close.
- **Credentials in `.env`, not code.** `tls_check.rs` and `batch_test.rs` use proxy credentials; the current `batch_test.rs` has a proxy key inline — when rotating, move it to `.env` rather than replacing in place.
- **Each example is self-contained.** No cross-example imports (Cargo doesn't allow it for examples anyway, but keep it that way).

## Key Patterns

- Profile selection: `Browser::builder().profile(7, 777)` — reference hardcoded index/seed used across smoke/youtube examples for repeatability
- CDP event subscription: `game_test.rs` — reference for subscribing to a chromiumoxide event stream via `page.cdp()` escape hatch
- Proxy + env loading: `tls_check.rs::load_dotenv()` — reference for minimal `.env` parsing without adding `dotenv` as a dep

## Dependencies

- Imports from: `clawser_browser` (crate root), `chromiumoxide` (for CDP events in `game_test.rs`), `tokio`, `futures_util`
- Runtime requirement: `CLAWSER_CHROME_PATH` pointing at a built `chrome.exe` from this fork

## Gotchas

- **`batch_test.rs` contains a live proxy API key.** Treat it as leaked-and-rotated on any upstream commit; if the key must be real, move it to `.env` and add a placeholder here.
- **`GAME_API_URL` default points at a third-party site.** Respect rate limits when running `game_test.rs` — it's intended for one-off checks, not scheduled CI.
- **Headful is the default in every example.** Switch to `.headless()` explicitly if running on a server; otherwise the examples will error with no display.

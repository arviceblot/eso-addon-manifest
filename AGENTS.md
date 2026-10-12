# eso_addon_manifest

A Rust library crate for parsing and validating ESO (Elder Scrolls Online) AddOn `Manifest.txt` files. Published on [crates.io](https://crates.io/crates/eso_addon_manifest).

## Quick Commands

```bash
cargo fmt --check          # lint formatting
cargo clippy               # lint logic
cargo test                 # run all tests (unit + doc)
cargo test --doc           # doc tests only
```

## Structure

- `src/lib.rs` — parser implementation, public API (`AddonManifestParser`, `AddonManifest`, `DependsEntry`), and all tests
- `src/error.rs` — `ManifestError` enum via `thiserror`
- `resources/test/` — test fixture manifest files
- Edition: **2024**

## Key Notes

- **Library crate only** — no `main.rs` or binary target.
- **`Cargo.lock` is not committed** (in `.gitignore`). CI generates it on-the-fly; local dev should run `cargo generate-lockfile` before `--locked` commands.
- **Test fixture dependency**: the `test_parse_file` test resolves paths via `CARGO_MANIFEST_DIR` and reads `resources/test/AddonName.txt`. That fixture intentionally has a malformed `APIVersion` line and the test asserts exactly 1 error.
- **Dependencies are minimal**: `regex` (parsing) + `thiserror` (error enum). No other crates.
- **CI follows the jonhoo/rust-ci-conf pattern**: fmt, clippy, test (stable + beta), doc (nightly), minimal-versions, cross-platform, sanitizers, coverage (grcov), and scheduled nightly/updated checks.

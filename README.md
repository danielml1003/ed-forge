# ed-forge

Lightweight Tauri starter for a game companion platform focused on core functionality without copying Overwolf UI or branding.

## Stack

- Tauri v2 (Rust backend + native window)
- Vanilla HTML/CSS/JS frontend (minimal runtime overhead)
- Cargo quality checks + GitHub Actions CI

## Product Surfaces

- `Discover`: provider aggregation, search/filter, app details, save-to-library
- `Library`: saved apps list, launch/remove actions, state tracking
- `Runtime`: low-resource and ingestion controls, sync cadence, runtime overview

## Project Layout

- `ui/index.html`, `ui/main.js`, `ui/styles.css`: Discover/Library/Runtime UI
- `src-tauri/src/lib.rs`: backend commands and runtime state management
- `src-tauri/src/main.rs`: desktop entrypoint
- `src-tauri/tauri.conf.json`: app window, bundle, security config
- `src-tauri/capabilities/default.json`: minimal permission capability
- `.github/workflows/ci.yml`: automated validation pipeline

## Backend Commands

Store commands:
- `store_list_providers`
- `store_list_items`
- `store_get_item`
- `store_refresh_cache`

Library commands:
- `library_list_apps`
- `library_save_item`
- `library_launch_item`
- `library_remove_item`

Runtime commands:
- `runtime_get_config`
- `runtime_update_config`

## Events

- `store-refreshed`
- `library-updated`

## First Run

1. Install dependencies:
   - `npm.cmd install`
2. Start dev app:
   - `npm.cmd run dev`
3. Build release bundle:
   - `npm.cmd run build`

## Validation Commands

- `cargo fmt --check --manifest-path src-tauri/Cargo.toml`
- `cargo clippy --all-targets --all-features --manifest-path src-tauri/Cargo.toml -- -D warnings`
- `cargo test --all-features --manifest-path src-tauri/Cargo.toml`
- `npm.cmd run build`

## Roadmap and Progress

- Roadmap: docs/roadmap.md`n- Checklist and log: docs/progress-checklist.md`n

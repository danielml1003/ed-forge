# ed-forge Progress Checklist

Last updated: 2026-02-20

## How to use

- Mark each checkbox when completed.
- Add a short log entry in "Progress Log" each time something meaningful ships.
- Keep dates in `YYYY-MM-DD` format.

## Milestone Checklist

### M1 - Discover, Library, Runtime Core

- [x] Discover surface scaffolded (search/filter/detail/save)
- [x] Library surface scaffolded (list/launch/remove)
- [x] Runtime surface scaffolded (config + overview)
- [x] Backend command contracts implemented for Store/Library/Runtime
- [x] Frontend wired to backend commands (interactive, not static)
- [x] CI baseline configured (`fmt`, `clippy`, `test`, `build`)
- [x] UX polish pass for flow clarity and consistency
- [x] Add integration tests for command-level behavior

### M2 - Live Provider Integrations

- [x] Add adapter trait and module structure
- [ ] Implement TFTMeta live adapter
- [ ] Implement Porofessor live adapter
- [ ] Add cache fallback and data freshness timestamps
- [ ] Add source health/error visibility in UI

### M3 - Game-Aware Runtime + Widgets

- [ ] Add game process detection service
- [ ] Add per-game activation profiles
- [ ] Add lightweight widget window manager
- [ ] Add resource guardrails for runtime workers
- [ ] Add runtime telemetry panel for worker states

### M4 - Security + Release

- [ ] Add capability/permission review checklist
- [ ] Add payload integrity checks
- [ ] Configure signed update pipeline
- [ ] Finish release readiness checklist
- [ ] Publish release candidate build

## Repeated Definition of Done

- [ ] Feature implemented and manually verified
- [ ] Rust checks pass (`fmt`, `clippy -D warnings`, `test`)
- [ ] App build passes (`npm run build`)
- [ ] Roadmap/checklist updated
- [ ] Commit pushed to GitHub

## Progress Log

- 2026-02-20: Completed M2.1 adapter trait and module structure with validated build/test pipeline.`r`n- 2026-02-20: Completed M1 polish and added command-level lifecycle tests for Library/Runtime.`r`n- 2026-02-20: Built original Discover/Library/Runtime architecture and shipped working Tauri build.
- 2026-02-20: Added CI workflow and validated local quality gates.
- 2026-02-20: Pushed foundation to `main` on GitHub repository.



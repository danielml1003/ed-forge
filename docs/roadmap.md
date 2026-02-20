# ed-forge Roadmap

Last updated: 2026-02-20

## Milestones

### M1 - Discover, Library, Runtime Core (2 weeks)

- Build Discover with search, provider filter, detail, save-to-library
- Build Library with launch/remove/state transitions
- Build Runtime controls for low-resource mode, ingestion, and sync interval
- Stabilize command/event contracts between frontend and Rust backend

### M2 - Live Provider Integrations (2 weeks)

- Implement provider adapter trait (`fetch_raw`, `normalize`, `validate`)
- Add live ingestion for TFTMeta source
- Add live ingestion for Porofessor source
- Add retry, timeout, stale-cache fallback strategy

### M3 - Game-Aware Runtime + Widgets (2 weeks)

- Detect active game process and map to profile
- Add per-game app activation rules
- Add optional lightweight widget windows and controls
- Add runtime protections for high CPU usage

### M4 - Security, Updates, Release Candidate (2 weeks)

- Add strict permission and capability review
- Add source integrity checks for downloaded payloads
- Implement signed updater flow and staged release option
- Finalize release hardening and launch checklist

## Workstreams

### Product

- Keep UI and branding original (no copied visual design/text)
- Keep no-login baseline for core flow

### Backend

- Keep Rust-first for performance-critical work
- Maintain small command handlers and explicit models

### Quality

- Enforce CI gates: fmt, clippy, tests, build
- Add and maintain unit + integration tests per new feature

### Performance

- Keep low-resource mode available and default-friendly
- Track startup time, idle CPU, memory ceiling trends

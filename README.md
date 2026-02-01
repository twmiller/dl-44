# DL-44

Open-source laser cutter/engraver control app.

## Stack
- UI: Svelte + Vite
- Backend: Rust + Tauri v2

## Repo Layout (proposed)
- `dl44-app/` Tauri desktop app (UI + Rust shell)
- `docs/` product plans, feature tiers, and design notes
- `LaserWeb4/` reference-only (ignored)

## Getting Started (macOS/Linux)

Prereqs:
- Node.js
- Rust toolchain

Run dev:
```sh
cd dl44-app
npm install
npm run tauri:dev
```

## Notes
- macOS and Linux are first-class; Windows is optional.
- Clean-room reimplementation with protocol behavior inferred as needed.

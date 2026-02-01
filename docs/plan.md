# DL-44 Development Plan

This plan uses the feature tiers in `docs/dl44-features.md` as the primary timeline backbone.

## Timeline (Tier-Based)

### Phase 0 - Foundation
- Repo hygiene, tooling, and a bootstrapped desktop shell.
- Goal: Svelte + Rust + Tauri app runs on macOS and Linux.

Phase 0 checklist:
- [x] Root `.gitignore`
- [x] Tauri v2 + Svelte app boots and runs
- [x] Rust toolchain file in place
- [x] README with setup/run steps
- [x] Repo layout agreed (UI + core crates plan)
- [ ] CI skeleton (optional for now)

### Phase 1 - Tier 1 (MVP)
- Core daily-driver capabilities: connect, import, layer ops, generate GCode, preview, stream, control.
- Goal: burn a simple SVG and a raster image reliably.

### Phase 2 - Tier 2 (Competitive)
- Material library, rotary basics, better raster options, alignment aids.
- Goal: parity for common diode workflows.

### Phase 3 - Tier 3/4 (Differentiation + Nice-to-have)
- ESP32 WiFi, advanced engraving modes, community presets, extras.
- Goal: step beyond LightBurn where it is weak.

## Epics and User Stories

### Epic A - Connection and Device Control (Tier 1)
- As a user, I can connect to a GRBL device over USB and see live status.
- As a user, I can home, jog, set origin, and run a frame preview.
- As a user, I can pause/resume/stop a job and use feed/power overrides.

Acceptance criteria:
- USB serial connect/disconnect, port list, baud setting.
- GRBL status polling and state changes reflected in UI.
- Overrides send correct real-time GRBL commands.

### Epic B - Import and Workspace (Tier 1)
- As a user, I can import SVG/DXF/bitmap and see it in the workspace.
- As a user, I can save/restore a workspace.

Acceptance criteria:
- SVG and bitmap import supported in UI and core.
- Basic document list/tree with visibility toggles.
- Workspace persists to a project file.

### Epic C - Operations and Layers (Tier 1)
- As a user, I can assign layers with power/speed/passes.
- As a user, I can order operations (cut last) and preview the toolpath.

Acceptance criteria:
- Layer list with reordering and per-layer params.
- GCode generation by layer ordering.
- Per-layer preview color and legend.

### Epic D - Raster Pipeline (Tier 1)
- As a user, I can engrave a bitmap using threshold or Floyd-Steinberg.
- As a user, raster output is smooth (S-value quantized).

Acceptance criteria:
- Dither options implemented in core.
- S-value quantization reduces GRBL buffering stalls.

### Epic E - Job Preview and Simulation (Tier 1)
- As a user, I can see a preview of the toolpath and a time estimate.
- As a user, I see job progress and ETA while running.

Acceptance criteria:
- Pre-job simulation using GCode parsing and motion timing.
- Progress shows % complete and time remaining.

### Epic F - Material Library (Tier 2)
- As a user, I can save presets per material and thickness.
- As a user, I can run a material test grid.

Acceptance criteria:
- Preset CRUD with import/export.
- Test pattern generator for power/speed grids.

### Epic G - Rotary (Tier 2)
- As a user, I can enable rotary mode and set diameter.

Acceptance criteria:
- Rotary mode changes Y-axis steps in a reversible way.
- Simple wrap preview for cylinders.

### Epic H - Camera Alignment (Tier 2)
- As a user, I can align artwork using a camera overlay.

Acceptance criteria:
- Camera feed shows in workspace.
- Calibration maps camera pixels to machine coords.

### Epic I - Differentiators (Tier 3)
- As a user, I can connect over native ESP32 WiFi and manage jobs.
- As a user, I can use dot mode or 3D engraving.

Acceptance criteria:
- WiFi job upload and TF card management.
- Dot mode and grayscale depth options.

## Proposed Build Order (within Tier 1)
1) USB connect -> status -> jog/home -> overrides
2) SVG/bitmap import -> workspace render
3) Layers/ops -> GCode generation
4) Preview + time estimate
5) Job streaming + pause/resume/stop
6) Raster pipeline improvements

## Architecture Notes

### Rust Crate Structure (src-tauri/src/)
```
grbl/
├── mod.rs          # Module exports
├── protocol.rs     # GRBL commands, constants, response parsing
├── serial.rs       # Port enumeration only
├── status.rs       # Machine status types and parsing
├── worker.rs       # Dedicated serial I/O thread
└── controller.rs   # High-level controller (delegates to worker)
commands.rs         # Tauri command handlers
lib.rs              # App entry point and setup
```

### Key Rust Crates
- `serialport` - Cross-platform serial I/O
- `thiserror` - Ergonomic error types
- `parking_lot` - Fast mutex for shared controller state
- `log` / `env_logger` - Logging

### Serial Worker Architecture

The GRBL serial layer uses a dedicated worker thread to prevent Tauri
commands from blocking on serial I/O:

```
┌─────────────────┐     mpsc channel      ┌─────────────────┐
│  Tauri Command  │ ──── Request ───────► │  Serial Worker  │
│   (main thread) │ ◄─── Response ─────── │    (thread)     │
└─────────────────┘    oneshot channel    └─────────────────┘
                                                  │
                                                  ▼
                                          ┌─────────────────┐
                                          │   Serial Port   │
                                          └─────────────────┘
```

**Request types:**
- `Connect(port, baud)` - Open serial port, wait for welcome message
- `Disconnect` - Close serial port
- `SendCommand(cmd, retries, timeout)` - Send command, wait for ok/error
- `SendRealtime(byte)` - Send single byte (no response expected)
- `QueryStatus(timeout)` - Send `?`, wait for status report

**Retry/timeout policy:**
- Commands that expect ok/error: 2 retries, 500ms per attempt
- Status queries: 300ms timeout, returns cached on timeout
- Errors surface to UI with structured codes (TIMEOUT, GRBL_ERROR, ALARM)

### UI Components (src/lib/)
```
components/
├── ConnectionPanel.svelte   # Port list, baud, connect/disconnect
├── StatusBar.svelte         # State, position display
├── JogControls.svelte       # XYZ jog buttons, step selector
├── MachinePanel.svelte      # Container panel
└── ErrorToast.svelte        # Error notification toast
stores/
└── machine.ts               # Svelte stores with error handling
```

### GRBL Protocol Layer (minimal)
- Status query: `?` → `<Idle|MPos:0.000,0.000,0.000|FS:0,0>`
- Jog: `$J=G91 X10.0 F1000`
- Home: `$H`
- Real-time: `!` (hold), `~` (resume), `0x18` (reset), `0x85` (jog cancel)

### Tier-1 Vertical Slice Progress
1) [x] USB connect -> status -> jog/home (scaffolding complete)
   - Serial port listing and connection
   - GRBL status parsing
   - Jog controls with step/feed selection
   - Home/unlock/reset commands
2) [ ] Overrides implementation
3) [ ] SVG/bitmap import -> workspace render
4) [ ] Layers/ops -> GCode generation
5) [ ] Preview + time estimate
6) [ ] Job streaming + pause/resume/stop
7) [ ] Raster pipeline improvements

## Notes
- macOS and Linux are first-class; Windows support is optional.
- Clean-room reimplementation, with protocol behavior inferred as needed.

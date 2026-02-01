# DL-44 Feature Matrix

An open-source laser cutter/engraver control application. This document catalogs the complete feature set needed to match and exceed LightBurn for GRBL-based diode lasers, with an honest assessment of what LaserWeb4 already provides versus what needs to be built.

Target hardware: Elegoo Phecda 20W (ESP32 controller, GRBL firmware, WiFi, rotary support)

---

## Connection & Communication

| Feature | LaserWeb4 | DL-44 Needed |
|---------|-----------|--------------|
| USB serial connection | ✅ Supported | — |
| GRBL 1.1+ protocol | ✅ Supported | — |
| Smoothieware protocol | ✅ Supported | Optional carry-forward |
| TinyG protocol | ✅ Supported | Optional carry-forward |
| MarlinKimbra protocol | ✅ Supported | Optional carry-forward |
| WiFi connection (ESP32 native) | ❌ Not supported | **Build** — Reverse-engineer Phecda WiFi protocol for native job submission |
| WiFi job upload to TF card | ❌ Not supported | **Build** — Send .nc/.gcode files over WiFi to onboard storage |
| TF card file management | ❌ Not supported | **Build** — Browse/upload/delete files on machine TF card |
| Machine auto-detection | ❌ Manual config | **Build** — Detect machine type and apply profile on connect |
| Connection over network (remote serial) | ✅ Supported (browser-based, can proxy) | — |

## File Import & Design

| Feature | LaserWeb4 | DL-44 Needed |
|---------|-----------|--------------|
| SVG import | ✅ Supported (CSS style attribute mode, quirks with some exporters) | **Improve** — Robust SVG parser, full CSS support |
| DXF import | ✅ Supported (R12 ASCII only) | **Improve** — Support R12 through R2018 formats |
| Bitmap import (JPG/PNG/BMP/GIF) | ✅ Supported | — |
| TIFF import | ❌ Not supported | **Build** |
| PDF import | ❌ Not supported | **Build** — Extract vectors from PDF |
| AI (Adobe Illustrator) import | ❌ Not supported | **Build** — Or convert via PDF path |
| GCode import/preview | ✅ Supported | — |
| Built-in vector drawing tools | ❌ Not supported | **Build** — Basic shapes, lines, text, bezier curves |
| Built-in text tool | ❌ Not supported | **Build** — Font selection, sizing, path text |
| Boolean operations (union/diff/intersect) | ❌ Not supported | **Build** |
| Node editing | ❌ Not supported | **Build** — Edit individual path nodes |
| Image tracing (bitmap to vector) | ⚠️ Partial (Potrace integration exists but buggy) | **Improve** — Clean Potrace/autotrace integration |
| Object alignment/distribution | ❌ Not supported | **Build** |
| Snap to grid / snap to object | ❌ Not supported | **Build** |
| Array/pattern fill | ❌ Not supported | **Build** — Tile objects in grid/circular patterns |
| Undo/redo | ✅ Supported (Redux state-based) | — |
| Workspace save/restore | ✅ Supported (JSON workspace files) | — |

## CAM Operations

| Feature | LaserWeb4 | DL-44 Needed |
|---------|-----------|--------------|
| Laser Cut (vector line following) | ✅ Supported | — |
| Laser Fill Path (vector hatching/infill) | ✅ Supported (slow on complex SVGs) | **Improve** — Performance optimization |
| Laser Raster (bitmap engraving) | ✅ Supported (dithering options present) | **Improve** — See Image Processing section |
| Laser Raster Merge | ✅ Supported (bugs with some firmware S-value handling) | **Fix** — Correct GCode generation for GRBL |
| Multi-layer jobs (different params per layer) | ⚠️ Partial — Multiple operations possible but UX is painful, ordering is confusing | **Build** — First-class layer system with color mapping, drag reorder, per-layer power/speed/passes |
| Layer color mapping from SVG | ❌ Not supported | **Build** — Map SVG stroke/fill colors to operation layers automatically |
| Operation ordering control | ⚠️ Partial — Operations run in list order, but pass interleaving is confusing | **Improve** — Clear inner-first, cut-last ordering with drag-and-drop |
| Multiple passes | ✅ Supported (per-operation pass count) | — |
| Per-element multi-pass (cut each shape N times before moving) | ❌ Not supported (passes interleave across all elements) | **Build** — Option for per-element vs per-operation passes |
| Tab/bridge support for cutouts | ❌ Not supported | **Build** — Prevent parts falling during cut |
| Lead-in/lead-out for cuts | ❌ Not supported | **Build** — Overburn compensation |
| Kerf offset | ❌ Not supported | **Build** — Compensate for laser beam width on cuts |

## Image Processing & Engraving

| Feature | LaserWeb4 | DL-44 Needed |
|---------|-----------|--------------|
| Brightness/contrast adjustment | ✅ Supported | — |
| Gamma correction | ❌ Not supported | **Build** |
| Image sharpening/blur | ❌ Not supported | **Build** |
| Dithering: Threshold | ✅ Supported | — |
| Dithering: Floyd-Steinberg | ✅ Supported | — |
| Dithering: Ordered/Bayer | ❌ Not supported | **Build** |
| Dithering: Stucki/Jarvis/Atkinson | ❌ Not supported | **Build** — Multiple algorithm options matter for different materials |
| Grayscale power mapping | ✅ Supported (but generates over-precise S values causing jerky motion) | **Fix** — Quantize S-values to integer steps for GRBL buffer performance |
| Scan angle rotation | ❌ Not supported | **Build** — Engrave at angles other than 0/90° |
| Bi-directional scanning | ✅ Supported | — |
| Overscan/overtravel | ❌ Not supported | **Build** — Acceleration margin beyond engrave area for consistent edge quality |
| Dot mode engraving | ❌ Not supported | **Build** — Fixed-time laser pulses for ceramic/stone/coated metal |
| 3D engrave (variable depth) | ❌ Not supported | **Build** — Grayscale maps to power for relief effect |
| Halftone patterns | ❌ Not supported | **Build** — Circle/line halftone for photo engraving |
| Newsprint/crosshatch patterns | ❌ Not supported | **Build** |

## Machine Control & Jogging

| Feature | LaserWeb4 | DL-44 Needed |
|---------|-----------|--------------|
| XY jogging (keyboard/button) | ✅ Supported | — |
| Configurable jog distances | ✅ Supported | — |
| Configurable jog speeds | ✅ Supported | — |
| Home machine | ✅ Supported | — |
| Set work origin | ✅ Supported | — |
| Go to origin | ✅ Supported | — |
| Laser test fire (timed pulse) | ✅ Supported (configurable power/duration) | — |
| Check size / frame preview | ⚠️ Partial — CHECK-SIZE power setting exists, basic outline trace | **Improve** — Smooth continuous perimeter trace with real-time power/speed display |
| Real-time position display | ✅ Supported (status polling) | — |
| Pause job | ✅ Supported | — |
| Resume job | ✅ Supported | — |
| Stop/abort job | ✅ Supported | — |
| Mid-job power adjustment | ❌ Not supported | **Build** — Adjust power % while job is running or paused |
| Mid-job speed adjustment | ❌ Not supported | **Build** — Adjust speed % while job is running or paused (GRBL real-time overrides) |
| GRBL real-time feed override | ❌ Not supported | **Build** — Send 0x91-0x9D bytes for ±10%/±1% feed adjust |
| GRBL real-time spindle override | ❌ Not supported | **Build** — Send 0x99-0x9C bytes for ±10%/±1% power adjust |
| GRBL alarm recovery ($X, $H) | ✅ Supported | — |
| Console / raw GCode entry | ✅ Supported | — |
| GRBL config editor ($$) | ❌ Not supported (manual console only) | **Build** — GUI for GRBL settings with descriptions |

## Job Preview & Simulation

| Feature | LaserWeb4 | DL-44 Needed |
|---------|-----------|--------------|
| 2D GCode path preview | ✅ Supported (WebGL) | — |
| GCode simulation playback | ✅ Supported (slider-based, buggy with rasters) | **Improve** — Smooth animation, accurate timing |
| Time estimate (pre-job) | ❌ Not supported (only post-job elapsed time) | **Build** — Parse GCode, calculate move durations including accel/decel |
| Progress indicator during job | ⚠️ Partial — Line count progress only | **Improve** — Percentage, time remaining, current pass/total passes |
| Preview differentiation (cut vs engrave vs travel) | ⚠️ Partial — Color exists but limited | **Improve** — Clear visual distinction, toggle layers |
| Burn simulation / preview rendering | ❌ Not supported | **Build** — Show approximate output appearance before burning |
| Cost/material calculator | ❌ Not supported | Nice-to-have |

## Rotary Attachment

| Feature | LaserWeb4 | DL-44 Needed |
|---------|-----------|--------------|
| Rotary enable/disable | ❌ Not supported | **Build** |
| Roller rotary mode | ❌ Not supported | **Build** — Y-axis drives rollers, calculate steps from object diameter |
| Chuck rotary mode | ❌ Not supported | **Build** — Direct drive, different step calculation |
| Object diameter input | ❌ Not supported | **Build** — Calculate circumference → Y-axis steps-per-mm |
| Rotary test (verify steps) | ❌ Not supported | **Build** — Rotate exact amount, user verifies |
| Rotary speed limiting | ❌ Not supported | **Build** — Prevent speeds that would slip rollers |
| Rotary wrapping preview | ❌ Not supported | **Build** — Show how flat design maps onto cylinder |
| Auto Y-axis swap for rotary | ❌ Not supported | **Build** — Reconfigure steps-per-mm when rotary enabled |

## Material Library & Presets

| Feature | LaserWeb4 | DL-44 Needed |
|---------|-----------|--------------|
| Material database | ❌ Not supported | **Build** — JSON store of material → thickness → power/speed/passes |
| Material test pattern generator | ❌ Not supported | **Build** — Generate grid of power/speed combinations for calibration |
| User-defined presets | ❌ Not supported | **Build** — Save and recall named parameter sets |
| Community shared presets | ❌ Not supported | Nice-to-have — Import/export preset files |
| Machine profile library | ✅ Supported (save/load machine profiles) | — |

## Camera & Visual Positioning

| Feature | LaserWeb4 | DL-44 Needed |
|---------|-----------|--------------|
| Webcam feed display | ⚠️ Partial — Basic webcam support exists, unreliable device detection | **Improve** — Reliable camera enumeration and display |
| Camera lens calibration | ❌ Not supported | **Build** — Barrel distortion correction wizard |
| Camera-to-workspace alignment | ❌ Not supported | **Build** — Map camera pixels to machine coordinates |
| Camera overlay on workspace | ❌ Not supported | **Build** — Show live/captured camera image behind design |
| Object trace from camera | ❌ Not supported | **Build** — Trace outlines from camera image for alignment |
| Print-and-cut registration | ❌ Not supported | Nice-to-have — Detect registration marks |

## Air Assist

| Feature | LaserWeb4 | DL-44 Needed |
|---------|-----------|--------------|
| Air assist GCode (M7/M8/M9) | ✅ Supported (configurable on/off GCode in machine settings) | — |
| Per-layer air assist toggle | ❌ Not supported | **Build** — Enable/disable air assist per operation layer |
| Air assist status display | ❌ Not supported | **Build** — Show current air assist state |

## Safety Features

| Feature | LaserWeb4 | DL-44 Needed |
|---------|-----------|--------------|
| Flame detection response | ❌ Not supported (handled by machine firmware) | Nice-to-have — React to firmware alarm |
| Tilt/move detection response | ❌ Not supported (handled by machine firmware) | Nice-to-have — React to firmware alarm |
| Job boundary check (vs machine limits) | ✅ Supported (machine dimension settings prevent out-of-bounds GCode) | — |
| Soft limits enforcement | ✅ Supported (via GRBL config) | — |
| Idle timeout / laser safety shutoff | ❌ Not supported | **Build** — Auto-off if communication lost |

## Application Architecture

| Feature | LaserWeb4 | DL-44 Needed |
|---------|-----------|--------------|
| Platform | Node.js + React + Redux + Electron | **Replace** — Tauri + Rust + Vue |
| Serial communication | Node.js server-side | **Replace** — Rust serial (tokio-serial or serialport crate) |
| GCode generation | JavaScript (browser) | **Replace** — Rust for performance, especially raster |
| Image processing | JavaScript (browser, slow) | **Replace** — Rust (image crate) for dithering/processing |
| State management | Redux | Vue reactive state or Pinia |
| 2D rendering | Three.js (WebGL) | Canvas 2D or WebGL (evaluate based on complexity) |
| Cross-platform | Electron (heavy) | Tauri (lightweight native wrapper) |
| Plugin/extension system | ❌ Not supported | Nice-to-have |
| Localization/i18n | ❌ Not supported | Nice-to-have |
| Auto-update | ❌ Not supported | **Build** via Tauri updater |

---

## Priority Tiers

### Tier 1 — Core (MVP, must ship)

These are the features that make DL-44 functional as a daily-driver laser tool:

- USB serial connection with GRBL 1.1+ 
- SVG/DXF/bitmap import
- Laser Cut, Laser Fill, Laser Raster operations
- Multi-layer jobs with per-layer power/speed/passes
- Basic image dithering (Floyd-Steinberg, threshold)
- Fix: S-value quantization for GRBL raster performance
- XY jogging, homing, origin setting
- Job preview with time estimate
- Pause/stop/resume
- GRBL real-time feed and spindle overrides
- Check size / perimeter preview
- Workspace save/restore
- Machine profiles

### Tier 2 — Competitive (matches LightBurn for diode users)

These close the gap that currently drives people to pay $60:

- Material library with presets
- Material test pattern generator
- Rotary attachment support (roller mode)
- Camera alignment and overlay
- Built-in text tool
- Advanced dithering (Jarvis, Stucki, Atkinson, ordered)
- Overscan / overtravel for engraving
- Per-element multi-pass
- Image trace (Potrace)
- Burn simulation preview
- Object alignment and distribution

### Tier 3 — Differentiation (things LightBurn doesn't do well or at all)

These are where DL-44 could pull ahead:

- Native WiFi connection to ESP32-based machines
- WiFi job upload and TF card management
- Machine auto-detection
- Dot mode engraving
- 3D engrave / variable depth
- Tab/bridge support
- Community preset sharing
- GRBL config GUI editor

### Tier 4 — Nice to have

- PDF/AI import
- Plugin system
- Localization
- Print-and-cut registration
- Cost/material calculator
- Boolean operations
- Chuck rotary mode

---

## Summary Counts

- **LaserWeb4 fully supports:** ~30 features
- **LaserWeb4 partially supports (needs improvement):** ~12 features
- **Needs to be built from scratch:** ~55 features
- **Nice-to-have / future:** ~10 features

# app-desktop Development Guidelines & Desktop Blueprint

## 1. Purpose & Scope

`app-desktop` is the native standalone Windows desktop application entry point. It compiles directly into a native `.exe` distribution without Chromium, Electron, or WebView dependencies.

It integrates `sim-ui`, `sim-components`, and `sim-core` with native operating system services.

---

## 2. Responsibilities & System Integration

1. **Native Windowing**: Configures `eframe` with native `wgpu` backends (DirectX 12 / Vulkan).
2. **Native File Dialogs**: Uses `rfd` (Rust File Dialog) for opening and saving projects (`.json`) and loading firmware files (`.bin`, `.hex`, `.elf`).
3. **Local Toolchain Execution**: Invokes local compilers (`arduino-cli`, `idf.py`) through child processes (`std::process::Command`), strictly separated from simulation updates.
4. **Desktop Packaging**: Builds lightweight standalone release binaries (`cargo build --release -p app-desktop`).

---

## 3. Frame Timing & Anti-Freeze Architecture

To guarantee smooth 60 FPS responsiveness during simulation:
1. **Delta Clamping**: Real-time delta per frame is clamped to `MAX_FRAME_SECONDS = 0.05` (50ms).
2. **Step Budgeting**: Fixed simulation ticks are limited to `MAX_STEPS_PER_FRAME = 25` per frame with sub-8ms budget.
3. **Lag Discarding**: If the simulation accumulator exceeds the step budget, remaining excess time is discarded to prevent catch-up spirals of death that cause Windows "Not Responding" freezes.
4. **Cycle Bounding**: Any hardware emulation cycles (Xtensa `s3emu`) are dynamically bounded to execute within sub-millisecond times per step.

---

## 4. Engineering Constraints

- **No Duplication**: Never implement circuit, pin, or component logic inside `app-desktop`.
- **Clean OS Boundaries**: All OS-specific APIs (`powershell`, file pickers, process spawning) must remain isolated behind cross-platform traits in `sim-ui` or application services.
- **Rule 5 & 6 Compliance**: Maintain small entrypoint files (<300 lines) with zero comments inside function bodies.

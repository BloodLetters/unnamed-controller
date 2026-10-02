# app-web Development Guidelines & WebAssembly Blueprint

## 1. Purpose & Scope

`app-web` is the WebAssembly (WASM) browser application entry point. It compiles the simulator into a client-side bundle running directly inside modern browsers via Trunk.

It integrates `sim-ui`, `sim-components`, and `sim-core` with HTML5 Canvas and web platform APIs.

---

## 2. Responsibilities & Browser Integration

1. **WASM Windowing**: Binds `eframe` to an HTML5 `<canvas id="sim_canvas">` element using WebGL or WebGPU.
2. **Browser Timekeeping**: Replaces native system timers with `web-time` and browser `performance.now()` wrappers to prevent WASM runtime panics.
3. **Browser File Uploads**: Replaces desktop file dialogs with browser File APIs for project importing and firmware loading.
4. **Trunk Packaging**: Manages `Trunk.toml`, asset bundling, and minified production WASM builds.

---

## 3. WebAssembly Specific Constraints

1. **No Blocking Operations**: The web thread is tied to the browser animation frame loop (`requestAnimationFrame`). Blocking I/O or unbounded loops freeze the browser tab.
2. **No Native Process Execution**: `std::process::Command` is unavailable on WASM. All local toolchains (`arduino-cli`, `idf.py`) must be guarded with `#[cfg(not(target_arch = "wasm32"))]`.
3. **Lightweight Firmware Execution**: In the browser, user sketches run via `sim-core`'s virtual compiler and `BytecodeVm`, providing instant execution without requiring any server-side compilation backend.

---

## 4. Engineering Constraints

- **No Duplication**: Never implement netlist, circuit, or component logic inside `app-web`.
- **Target Compatibility**: All dependencies pulled into `app-web` must support `wasm32-unknown-unknown`.
- **Rule 5 & 6 Compliance**: Maintain small entrypoint files (<300 lines) with zero comments inside function bodies.

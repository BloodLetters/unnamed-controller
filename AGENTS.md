# Architectural Blueprint & Development Guidelines: Rust Circuit & MCU Simulator

## 1. Project Overview & Architecture

A modular, high-performance electronics and MCU simulator built in Rust.
- **Targets**: Native Windows desktop standalone (`eframe`, `wgpu`) and WebAssembly (`wasm32-unknown-unknown`, Trunk).
- **Architecture**: Core simulation engine is decoupled from rendering, OS, and browser APIs.

### Workspace Layout & Dependency Direction

```text
app-web      ─┐
              ├── sim-ui ─── sim-components ─── sim-core
app-desktop ──┘
```

- **`sim-core`**: Pure netlist solver, pin resolution, analog analysis, clock/event scheduling. Platform-agnostic (NO browser DOM, Windows API, `egui`, `eframe`, or rendering backends).
- **`sim-components`**: Component models, logic gates, ICs, MCU cores, and peripheral protocols.
- **`sim-ui`**: Circuit canvas, component rendering, wire routing, and interaction widgets.
- **`app-desktop` / `app-web`**: Platform entry points, file dialogs, runtime hosting.

*Crate-specific guidelines:*
- [`crates/sim-core/AGENTS.md`](crates/sim-core/AGENTS.md)
- [`crates/sim-components/AGENTS.md`](crates/sim-components/AGENTS.md)
- [`crates/sim-ui/AGENTS.md`](crates/sim-ui/AGENTS.md)
- [`crates/app-desktop/AGENTS.md`](crates/app-desktop/AGENTS.md)
- [`crates/app-web/AGENTS.md`](crates/app-web/AGENTS.md)

### Platform Abstraction

| Concern | Web (WASM) | Desktop (Native) | Abstraction / Crate |
|---|---|---|---|
| Window / UI | Canvas (`eframe`) | Native Window (`eframe`) | `sim-ui` / `eframe` |
| Rendering | WebGL / WebGPU | `wgpu` | `eframe` / render layer |
| High-Res Time | Performance timer | Instant / High-res timer | `web-time` |
| Firmware & Files | Browser File API | Native file dialog (`rfd`) | Application layer |

---

## 2. Core Engineering Principles

### Idiomatic Rust & Robustness
- **Errors**: Return `Result<T, E>` with domain error types for recoverable errors. Return `Option<T>` for legitimate absence.
- **No panics in production**: Never use `unwrap()`, `expect()`, or `panic!` on runtime data, user inputs, circuit setups, or firmware files.
- **Strong Typing**: Use strong types (`PinId`, `ComponentId`, `NodeId`) instead of raw primitives. Use enums for discrete, mutually exclusive states.
- **Memory Discipline**: Avoid unnecessary `.clone()` and heap allocations on hot paths (ticks, render frames). Borrow when ownership is not required. Prefer contiguous collections (`Vec`, small arrays).

### Code Organization & Modularity
- **Single Responsibility**: Every module, type, and function must have one clear duty.
- **No God Files**: Target < 300 LOC per file. Decompose into `types.rs`, `traits.rs`, `error.rs`, `service.rs`, `engine.rs`, etc.
- **Reuse Before Abstraction**: Check existing code before introducing new functions, types, or helpers. Do not duplicate conversion or validation logic.
- **API Visibility**: Keep APIs minimal and intentional. Prefer `pub(crate)` or private items over blanket `pub`.

### Naming Conventions
- Types/Traits/Enums: `PascalCase` (`CircuitNode`, `PinState`).
- Functions/Variables/Modules: `snake_case` (`update_pin_state`, `node_id`).
- Constants: `SCREAMING_SNAKE_CASE` (`MAX_COMPONENTS`).
- Names must be descriptive and human-readable (avoid `tmp`, `data`, `x`).

---

## 3. Strict Comment Policy (MANDATORY)

1. **NO Comments Inside Function Bodies**:
   - Never write comments inside function bodies or between statements.
   - Code logic must be self-explanatory via clean abstractions, small functions, and expressive naming.
2. **Comment Tags Placed Directly Above Declarations**:
   - Documentation (`///`) and intent comments (`//`) belong strictly **above** the function, struct, or enum declaration.
3. **Describe Intent, Not Mechanics**:
   - Comments must explain *why*, invariants, or non-obvious constraints—never rephrase what the code visibly does.
   - Avoid unnecessary comments.

---

## 4. Performance & Concurrency

- **Hot Simulation Loop**: Zero temporary allocations per tick, no blocking I/O, no UI updates on the simulation tick thread.
- **Rendering Loop**: Never block on file operations, firmware loading, or heavy synchronous solving.
- **Concurrency**: State separation between UI and simulation must be explicit and thread-safe (channels/message passing over shared mutable state).

---

## 5. Testing & Validation

### Testing Standards
- **Unit Tests**: Netlist graph, pin transitions, event scheduling, protocol parsers.
- **Integration Tests**: Inter-crate component interactions, MCU GPIO to peripherals, firmware execution.
- **Regression Tests**: Every bug fix requires a reproducing regression test before the fix.

### Validation Checklist
Before concluding any task:
```bash
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets --all-features
```

---

## 6. Definition of Done & Priorities

A feature is complete only when:
1. Architecture boundaries and platform independence are strictly preserved.
2. Code is idiomatic, strongly typed, and free of unnecessary allocations or dependencies.
3. **Zero comments exist inside function bodies**; all docs are above declarations.
4. Unit/integration tests pass and `cargo fmt`, `check`, `test`, `clippy` succeed with zero warnings.

**Core Priority Hierarchy**: Correctness > Architecture Boundaries > Maintainability > Reuse > Testability > Performance > Simplicity.

<!-- antislop:start -->
## antislop
For UI, copy, people, mobile layout, or code comments work, read `antislop.md` (core) and then the skill for the task:
- UI / visual: `skills/antislop-ui/SKILL.md`
- Copy & text: `skills/antislop-copywriting/SKILL.md`
- People: `skills/antislop-human/SKILL.md`
- Mobile / responsive: `skills/antislop-layoutmobile/SKILL.md`
- Code comments: `skills/antislop-code/SKILL.md`
Before starting, ask the user when antislop applies: during the work, or after it is done.
<!-- antislop:end -->

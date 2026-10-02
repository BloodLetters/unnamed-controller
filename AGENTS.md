# Architectural Blueprint & Development Guidelines: Rust Circuit & MCU Simulator

## 1. Project Overview

This project is a modular, high-performance electronics and microcontroller simulator inspired by Wokwi and built entirely in Rust.

The architecture must keep the simulation engine independent from rendering, operating-system APIs, browser APIs, and other platform-specific concerns whenever possible.

### Core Capabilities

- Dual target deployment:
  - WebAssembly (WASM) running in modern browsers.
  - Native Windows desktop application distributed as a standalone `.exe`.
- Unified simulation engine:
  - Netlist management.
  - Pin and signal state propagation.
  - Digital and analog simulation state.
  - Clock and event scheduling.
  - Peripheral communication.
  - Microcontroller emulation.
- Interactive circuit editor:
  - Component placement.
  - Wire creation.
  - Pin inspection.
  - Circuit manipulation.
  - Real-time simulation visualization.
- Firmware support:
  - Loading compiled firmware.
  - Connecting MCU GPIO and peripherals to simulated components.
- Cross-platform architecture:
  - Shared domain and simulation logic.
  - Platform-specific application entry points.
  - No WebView or Chromium dependency for the native desktop target.

---

## 2. Mandatory Engineering Principles

All code added to the project must follow these principles.

### Rule 1: Clean Code

- Prefer readable and predictable code over clever code.
- Use descriptive names for functions, variables, structs, enums, traits, modules, and files.
- Keep functions focused on one responsibility.
- Avoid deeply nested control flow.
- Avoid duplicated logic.
- Reuse existing functions and abstractions before creating new ones.
- Do not create a second function that performs substantially the same operation as an existing function.
- Do not introduce abstractions without a concrete reason.
- Keep public APIs small and intentional.

### Rule 2: Idiomatic Rust

- Follow idiomatic Rust conventions and the Rust API Guidelines.
- Prefer `Result<T, E>` for recoverable errors.
- Prefer `Option<T>` for values that may legitimately be absent.
- Do not use `unwrap()` or `expect()` in production paths unless the invariant is explicit and the failure is genuinely impossible.
- Avoid unnecessary `clone()`.
- Avoid unnecessary heap allocations.
- Prefer borrowing when ownership is not required.
- Prefer iterators when they improve clarity.
- Do not force iterator-heavy code when a normal loop is clearer.
- Use exhaustive `match` expressions where appropriate.
- Use enums to represent meaningful state instead of loosely related booleans.
- Keep ownership boundaries clear.

### Rule 3: Single Responsibility

Every module, type, and function must have a clear responsibility.

Examples:

- Netlist code manages circuit connectivity.
- Pin code manages pin state.
- Clock code manages simulation timing.
- Component code models component behavior.
- Renderer code draws objects.
- UI code handles interaction.
- Application code connects platform-specific services.

Do not place rendering logic inside simulation components.

Do not place browser or operating-system APIs inside `sim-core`.

### Rule 4: Reuse Before Abstraction

Before adding a new function, type, trait, utility, or module:

1. Search the existing codebase for an equivalent implementation.
2. Determine whether the existing implementation can be reused.
3. Extend the existing abstraction if the new behavior belongs to it.
4. Create a new abstraction only when the responsibility is genuinely different.

Avoid:

- Duplicate helper functions.
- Duplicate validation logic.
- Duplicate conversion logic.
- Multiple versions of the same state-management function.
- Generic abstractions created only to avoid a few lines of code.

### Rule 5: No God Files

Avoid excessively large source files.

- Target less than 300 lines of functional Rust code per file.
- If a file grows beyond this size, evaluate whether responsibilities should be separated.
- Do not split files artificially when doing so would make the architecture harder to understand.
- Separate unrelated types, traits, services, rendering logic, and test suites when they have different responsibilities.

Recommended separation:

- `types.rs` for domain data structures.
- `traits.rs` for interfaces and contracts.
- `error.rs` for domain-specific errors.
- `service.rs` for service-level operations.
- `engine.rs` for simulation orchestration.
- `tests.rs` for larger test modules.
- `mod.rs` or the parent module file for API organization.

---

## 3. Comment and Documentation Policy

This project follows a strict comment policy.

### Rule 6: No Comments Inside Function Bodies

Do not place comments inside function bodies.

Function bodies must be understandable through:

- Clear names.
- Small functions.
- Explicit control flow.
- Appropriate types.
- Well-defined abstractions.

Incorrect:

```rust
fn simulate_step() {
    // Update all connected pins
    update_pins();
    advance_clock();
}
```

Correct:

```rust
// Updates all connected pins before advancing the simulation clock.
fn simulate_step() {
    update_pins();
    advance_clock();
}
```

### Rule 7: Comment Tags Must Be Above the Function Name

When a comment, documentation comment, or comment tag is required for a function, it must be placed directly above the function declaration.

The comment belongs to the function declaration, not to an instruction inside the function body.

Correct:

```rust
/// Advances the simulation by one clock cycle.
fn advance_cycle() {
    update_components();
    propagate_signals();
}
```

Also acceptable:

```rust
// Advances the simulation by one clock cycle.
fn advance_cycle() {
    update_components();
    propagate_signals();
}
```

Incorrect:

```rust
fn advance_cycle() {
    // Advances the simulation by one clock cycle.
    update_components();
    propagate_signals();
}
```

### Rule 8: Comment Tags Must Describe Intent

Comments should explain intent, responsibility, invariants, or non-obvious decisions.

Do not write comments that merely repeat the code.

Bad:

```rust
// Increment cycle
cycle += 1;
```

Better:

```rust
/// Advances the simulation clock after all component updates are complete.
fn advance_cycle() {
    cycle += 1;
}
```

### Rule 9: Avoid Unnecessary Comments

Do not add comments simply to make code appear documented.

Prefer:

```rust
fn calculate_voltage(current: f32, resistance: f32) -> f32 {
    current * resistance
}
```

instead of:

```rust
// Multiply current by resistance.
fn calculate_voltage(current: f32, resistance: f32) -> f32 {
    current * resistance
}
```

Use Rustdoc for public APIs where documentation provides meaningful information.

### Rule 10: No Comment Tags Between Statements

Do not use comments as section separators inside functions.

Incorrect:

```rust
fn process_component() {
    validate_component();

    // Process inputs
    read_inputs();

    // Process outputs
    write_outputs();
}
```

Refactor the function instead:

```rust
fn process_component() {
    validate_component();
    process_inputs();
    process_outputs();
}
```

---

## 4. Naming Conventions

Use standard Rust naming conventions.

### Types

Use `PascalCase`.

```rust
struct CircuitNode;
enum PinState;
trait Component;
```

### Functions and Variables

Use `snake_case`.

```rust
fn update_pin_state() {}
let simulation_time = 0.0;
```

### Constants

Use `SCREAMING_SNAKE_CASE`.

```rust
const MAX_COMPONENTS: usize = 1024;
```

### Modules

Use lowercase `snake_case`.

```text
netlist/
pin_state/
component_registry/
```

### Naming Quality

Names must describe what the value represents.

Avoid:

```rust
let x = ...;
let data = ...;
let tmp = ...;
let value2 = ...;
```

Prefer:

```rust
let node_id = ...;
let component_state = ...;
let pending_event = ...;
let output_voltage = ...;
```

Use natural, human-readable names. Do not intentionally create obscure names to make code shorter.

---

## 5. Error Handling

Errors must be explicit and meaningful.

### General Rules

- Use `Result<T, E>` for recoverable failures.
- Use `Option<T>` when absence is expected.
- Define domain-specific error types when callers need to distinguish failures.
- Preserve useful error context.
- Avoid silently ignoring errors.
- Do not use `unwrap()` as a shortcut for normal error handling.
- Do not use `panic!` for user input, firmware input, circuit configuration, or runtime failures.

Example:

```rust
fn load_firmware(path: &Path) -> Result<Firmware, FirmwareError> {
    let bytes = std::fs::read(path)?;
    Firmware::from_bytes(&bytes)
}
```

Use `panic!` only for genuine programmer invariants that cannot reasonably occur during normal operation.

---

## 6. Function Design

Functions should be small, composable, and reusable.

### Function Rules

- One primary responsibility per function.
- Avoid functions that perform unrelated operations.
- Prefer explicit parameters over hidden global state.
- Avoid excessive parameter lists.
- Return meaningful types.
- Keep side effects visible.
- Reuse existing functions when the same operation already exists.

If a function becomes difficult to understand, split it by responsibility rather than adding comments inside its body.

Example:

```rust
fn simulate_component(component: &mut ComponentState) {
    read_component_inputs(component);
    update_component_state(component);
    write_component_outputs(component);
}
```

Each helper must represent a genuinely separate responsibility.

Do not create unnecessary wrappers such as:

```rust
fn do_update(component: &mut ComponentState) {
    update_component(component);
}
```

unless the wrapper provides a meaningful abstraction.

---

## 7. Data and State Modeling

Use strong types to represent domain concepts.

Prefer:

```rust
struct PinId(u32);
struct ComponentId(u32);
struct NodeId(u32);
```

over passing unrelated primitive integers throughout the engine.

Use enums for mutually exclusive states.

```rust
enum PinState {
    High,
    Low,
    HighImpedance,
}
```

Keep mutable state localized.

Avoid global mutable state unless the architecture explicitly requires it and synchronization is correctly handled.

---

## 8. Architecture and Module Boundaries

The project uses a Cargo workspace to separate simulation logic, components, UI, and application targets.

```text
unnamed-controller/
├── Cargo.toml
├── AGENTS.md
├── crates/
│   ├── sim-core/
│   ├── sim-components/
│   ├── sim-ui/
│   ├── app-desktop/
│   └── app-web/
└── assets/
```

### Dependency Direction

The dependency direction should generally flow from infrastructure-independent logic toward platform-specific applications.

```text
app-web      ─┐
              ├── sim-ui ─── sim-components ─── sim-core
app-desktop ──┘
```

`sim-core` must not depend on:

- Browser APIs.
- Windows APIs.
- `eframe`.
- UI widgets.
- Rendering backends.
- File dialogs.
- Platform-specific application code.

Platform-specific behavior belongs at the application boundary.

---

## 9. Crate Responsibilities

*Note: Crate-specific architectural rules have been moved to their respective directories. See:*
- [`crates/sim-core/AGENTS.md`](crates/sim-core/AGENTS.md)
- [`crates/sim-components/AGENTS.md`](crates/sim-components/AGENTS.md)
- [`crates/sim-ui/AGENTS.md`](crates/sim-ui/AGENTS.md)
- [`crates/app-desktop/AGENTS.md`](crates/app-desktop/AGENTS.md)
- [`crates/app-web/AGENTS.md`](crates/app-web/AGENTS.md)

---

## 10. Cross-Platform Abstraction Strategy

| Concern | Web | Desktop | Abstraction |
|---|---|---|---|
| Window | Browser canvas | Native window | `eframe` |
| Rendering | WebGL/WebGPU | `wgpu` | `eframe` / rendering layer |
| Time | Browser-compatible timer | Native high-resolution timer | `web-time` |
| Firmware loading | Browser File API | Native file dialog | Application abstraction |
| Simulation | Shared | Shared | `sim-core` |
| Components | Shared | Shared | `sim-components` |
| UI | Shared where possible | Shared where possible | `sim-ui` |

Platform-specific code must be isolated behind explicit abstractions.

Do not add `#[cfg(...)]` throughout unrelated simulation code when a clean platform boundary can be created instead.

---

## 11. Simulation Engine Rules

*See [`crates/sim-core/AGENTS.md`](crates/sim-core/AGENTS.md)*

---

## 12. Component Architecture

*See [`crates/sim-components/AGENTS.md`](crates/sim-components/AGENTS.md)*

---

## 13. Microcontroller Architecture

*See [`crates/sim-components/AGENTS.md`](crates/sim-components/AGENTS.md)*

---

## 14. Rendering and UI Rules

*See [`crates/sim-ui/AGENTS.md`](crates/sim-ui/AGENTS.md)*

---

## 15. Performance Rules

Performance matters because the simulator may contain many components and frequent simulation ticks.

### General

- Avoid unnecessary allocations in hot paths.
- Avoid repeated cloning of large structures.
- Prefer contiguous data where appropriate.
- Use references instead of cloning when ownership is not required.
- Profile before introducing complex optimizations.
- Do not sacrifice maintainability for hypothetical performance improvements.

### Simulation Loop

The simulation loop must avoid unnecessary work.

Do not:

- Recalculate unchanged state without a reason.
- Allocate temporary collections every tick without need.
- Perform blocking I/O during simulation updates.
- Perform expensive UI operations from the core simulation thread.

### Rendering Loop

The renderer must avoid:

- Blocking file operations.
- Firmware parsing.
- Large synchronous computations.
- Unnecessary rebuilding of static resources.

---

## 16. Concurrency

Concurrency must be explicit and safe.

- Avoid shared mutable state whenever possible.
- Prefer message passing when ownership boundaries are clear.
- Use synchronization primitives only when necessary.
- Do not introduce threads simply because they are available.
- Ensure simulation state cannot be concurrently mutated by UI and simulation code without synchronization.

For desktop simulation, a dedicated simulation thread may be used when justified.

For WebAssembly, account for browser threading and target limitations before designing a multithreaded architecture.

---

## 17. Testing Requirements

Every significant piece of simulation logic should have tests.

### Unit Tests

Test:

- Pin transitions.
- Netlist connections.
- Signal propagation.
- Clock behavior.
- Event ordering.
- Component state changes.
- Protocol behavior.
- Error conditions.

### Integration Tests

Use integration tests for behavior that crosses multiple modules or crates.

Examples:

- Component connected to LED.
- MCU GPIO connected to a simulated output.
- UART communication between components.
- Firmware execution affecting circuit state.

### Regression Tests

When fixing a bug:

1. Reproduce the bug.
2. Add a regression test.
3. Fix the implementation.
4. Confirm the regression test passes.
5. Run the relevant broader test suite.

Do not remove a regression test merely because the original bug is fixed.

---

## 18. Validation and Tooling

Before considering a change complete, run appropriate Rust tooling.

Recommended checks:

```text
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets --all-features
```

Use the commands appropriate to the configured workspace and target platform.

Do not ignore compiler or Clippy warnings without a clear reason.

---

## 19. Dependencies

Do not add dependencies casually.

Before adding a crate:

1. Check whether the standard library already provides the required functionality.
2. Check whether the workspace already contains a dependency that can solve the problem.
3. Verify that the dependency supports the required targets.
4. Consider compile time and binary size.
5. Consider WebAssembly compatibility.
6. Consider maintenance and ecosystem maturity.

Avoid multiple crates that solve the same problem unless there is a documented reason.

---

## 20. File and Module Organization

Keep files organized by responsibility.

Example:

```text
sim-core/
└── src/
    ├── lib.rs
    ├── error.rs
    ├── traits.rs
    ├── netlist/
    │   ├── mod.rs
    │   ├── graph.rs
    │   ├── node.rs
    │   └── connection.rs
    ├── pins/
    │   ├── mod.rs
    │   ├── state.rs
    │   └── pin.rs
    └── clock/
        ├── mod.rs
        ├── scheduler.rs
        └── tick.rs
```

Every module should expose a deliberate public API.

Do not make everything `pub` simply to bypass module-boundary problems.

Prefer:

```rust
pub(crate)
```

or private items when external access is unnecessary.

---

## 21. Public API Rules

Public APIs must be intentional and stable.

Before making an item public, ask:

- Does another crate actually need this?
- Can the operation be exposed through an existing public abstraction?
- Is this implementation detail or domain API?
- Would exposing it make future refactoring harder?

Avoid exposing internal implementation details.

---

## 22. Serialization and Persistence

When serializing project or simulator data:

- Use explicit schemas.
- Keep serialized names stable.
- Prefer meaningful field names.
- Validate external data before using it.
- Handle missing or unknown fields according to the intended compatibility policy.
- Do not assume persisted data is valid.

If `serde` is used, naming conventions such as `camelCase` are acceptable when they are required by an external format or API contract.

The internal Rust naming convention should remain idiomatic Rust.

---

## 23. Security and Input Validation

Treat all external input as untrusted.

External input includes:

- Firmware files.
- Circuit/project files.
- Browser file uploads.
- Serialized state.
- User-created component configuration.

Validate:

- File size.
- Data structure.
- Numeric ranges.
- Enum values.
- Resource limits.
- Invalid references.
- Malformed firmware.

Do not allow malformed input to crash the entire application when it can be rejected safely.

---

## 24. Development Workflow

For every non-trivial feature:

1. Understand the existing architecture.
2. Search for existing implementations.
3. Identify the correct module boundary.
4. Define or reuse the appropriate data model.
5. Implement the smallest clean change.
6. Add tests.
7. Run formatting and validation.
8. Review for duplication.
9. Review ownership and allocation behavior.
10. Confirm platform boundaries are preserved.

Do not rewrite unrelated parts of the project while implementing a feature.

---

## 25. Change Discipline

Keep changes focused.

Do not:

- Rename unrelated modules.
- Reformat unrelated files.
- Replace working dependencies without a reason.
- Rewrite stable systems unnecessarily.
- Introduce unrelated abstractions.
- Mix feature work with large architectural migrations unless the migration is required.

A change should be easy to review and easy to revert.

---

## 26. Development Roadmap

### Phase 1: Workspace Skeleton and Core Netlist

- Configure the Cargo workspace.
- Create `sim-core`.
- Implement pin models.
- Implement nodes and connections.
- Implement the initial netlist solver.
- Implement basic passive components.

### Phase 2: Interactive Canvas

- Create the `sim-ui` crate.
- Implement the circuit canvas.
- Implement grid rendering.
- Implement component rendering.
- Implement pin visualization.
- Implement wire creation.
- Implement component selection and movement.

### Phase 3: Native Desktop Application

- Create `app-desktop`.
- Configure native rendering.
- Add native file dialogs.
- Connect the desktop application to shared simulation and UI crates.
- Produce a standalone Windows release.

### Phase 4: WebAssembly Application

- Create `app-web`.
- Configure Trunk.
- Configure WASM compilation.
- Connect shared simulation and UI crates.
- Verify browser compatibility.

### Phase 5: MCU Simulation

- Add the first instruction-level MCU emulator.
- Implement firmware loading.
- Connect GPIO to simulated components.
- Add peripheral communication.
- Add debugging and inspection capabilities.

---

## 27. Definition of Done

A feature is considered complete only when:

- The implementation follows the existing architecture.
- Existing functionality is not unnecessarily duplicated.
- Naming is clear and idiomatic.
- No unnecessary comments exist inside function bodies.
- Required comment tags are placed directly above the relevant function declaration.
- Errors are handled appropriately.
- Platform-specific code remains isolated.
- Relevant tests are added or updated.
- Formatting passes.
- Compilation passes.
- Relevant Clippy checks pass.
- The change does not introduce unnecessary dependencies.
- The implementation is understandable without comments inside function bodies.

---

## 28. Final Engineering Rules

When there is a conflict between a quick implementation and a clean architectural implementation, prefer the clean implementation unless the simpler implementation is demonstrably sufficient.

Always prioritize:

1. Correctness.
2. Architectural boundaries.
3. Maintainability.
4. Reuse.
5. Testability.
6. Performance.
7. Simplicity.

Never solve a local problem by creating a larger architectural problem elsewhere.

Before creating a new function, type, trait, or utility, verify that the existing codebase does not already provide the required behavior.

Keep function bodies clean and self-explanatory.

If a function needs a comment tag, place that tag above the function name/signature, never inside the function body.

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

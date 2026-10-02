# Adding a New Component

Standard workflow for adding simulated electronic components and boards to the project.

---

## 1. Prerequisite: Specification & Research

Before writing code, gather complete hardware specifications from official datasheets:

1. **Physical Form & Footprint**:
   - Exact body dimensions (width, height in mm / canvas units).
   - Pin layout, pitch, and relative offset coordinates $(x, y)$ from the component center.
   - Default visual orientation (pin 1 top-left, connectors facing correct direction).
2. **Datasheet & Electrical Parameters**:
   - Voltage operating range ($V_{min}$ - $V_{max}$) and logic threshold levels ($V_{IL}$, $V_{IH}$).
   - Pin directions: Power (`VCC`, `GND`), Input, Output, Open-Drain, Analog, Tristate.
   - Communication bus specifics (e.g. I2C 7-bit slave address, SPI clock phase/polarity, UART frame format).
3. **Protocol & State Machine**:
   - Initialization sequence, registers, command set, and internal memory layout (e.g. display VRAM buffer, control registers).
   - Real-world timing constraints (clock rates, reset duration, bus speeds).

---

## 2. Step-by-Step Implementation

### Step 1: Model Emulation in `crates/sim-components`
1. Create `crates/sim-components/src/component/<name>.rs` (or `board/<name>/` if micro-controller).
2. Define the struct with its `PinId`s, state machine, and communication interface (e.g. `I2cDevice` trait):
   ```rust
   #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
   pub struct MyComponent {
       vcc: PinId,
       gnd: PinId,
       // Bus or signal pins...
   }
   ```
3. Expose the module in `crates/sim-components/src/component/mod.rs` and re-export in `lib.rs`.
4. Add comprehensive unit tests in `crates/sim-components/tests/component_<name>_test.rs`.

---

### Step 2: Canvas Rendering & Inspector in `crates/sim-ui`
1. Create `crates/sim-ui/src/canvas/components/<name>.rs`:
   - Implement `draw_single_<name>(painter: &Painter, center: Pos2, zoom: f32, rot: u16, comp: &MyComponent, is_selected: bool)`.
   - Apply rotation to relative offsets using `crate::canvas::rotate_offset(offset, rotation)`.
   - Draw component package, labels, active visual feedback, and pin terminals.
   - (Optional) Implement `render_<name>_context_options(ui: &mut Ui, comp: &mut MyComponent)`.
2. (Optional) Create `crates/sim-ui/src/inspector/<name>.rs`:
   - Implement `render_<name>_inspector_content(ui: &mut Ui, comp: &mut MyComponent, engine: &Engine, on_delete: impl FnOnce())`.

---

### Step 3: Register in the Unified Component Architecture (`crates/sim-ui/src/component/`)
Thanks to the flexible component abstraction, adding a new discrete component only requires registering it in `crates/sim-ui/src/component/`:

1. **`crates/sim-ui/src/component/instance.rs`**:
   - Add variant `ComponentInstance::<Name>(MyComponent)`.
   - Implement delegation match arms in `name()`, `pins()`, `pin_offsets()`, `bounds()`, `draw()`, `update_electrical()`, `handle_i2c_write()`, `render_inspector()`, `render_context_menu()`, and `duplicate()`.
2. **`crates/sim-ui/src/component/kind.rs`**:
   - Add variant `ComponentKind::<Name>`.
   - Implement factory creation in `ComponentKind::create()`.
3. `pins.rs`, `sim.rs`, `view.rs`, `state/spawn.rs`, `state/deletion.rs`, `state/mutation.rs`, `state/snapshot.rs`, and `registry.rs` automatically work with the new component without any changes!

---

### Step 4: Catalog & Palette Integration
1. **`crates/sim-ui/src/add_component/catalog.rs`**:
   - Add catalog entry to `CATALOG_ITEMS`:
     ```rust
     ComponentItem {
         id: "<name>",
         name: "<Display Name>",
         category: CategoryKind::<Boards|Displays|Output|Sensors>,
         specs: "<Key specifications>",
         badge: Some("Ready"),
         is_pro: false,
         is_ready: true,
         spawn: SpawningComponent::Component(ComponentKind::<Name>),
     }
     ```
2. *(Optional)* Add quick-spawn button to `PALETTE` in `crates/sim-ui/src/registry.rs`.

---

## 3. Quality & Architecture Checklist

Strictly enforce repository rules:
- **Rule 5 (No God Files)**: Each file must stay below 300 lines of functional Rust code.
- **Rule 6 (Comment Policy)**: No comments inside function bodies. Only doc comments (`///`) directly above function declarations explaining intent.
- **Verification Commands**:
  ```bash
  cargo check --workspace
  cargo test --workspace
  cargo clippy --workspace --all-targets --all-features
  cargo fmt --all -- --check
  ```

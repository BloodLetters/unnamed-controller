# sim-ui Development Guidelines & Architecture Blueprint

## 1. Purpose & Scope

`sim-ui` is the presentation, layout, and interactive canvas layer for the circuit simulator. It is built with `egui` and `eframe` and deployed across both native Windows desktop (`app-desktop`) and WebAssembly browser (`app-web`) targets.

It strictly **consumes** the simulation engine state from `sim-core` and hardware models from `sim-components`. It never contains hardware simulation physics or execution logic.

---

## 2. Layout Architecture & Interface Hierarchy

The user interface follows a modern 3-column workspace layout:

```text
┌────────────────────────────────────────────────────────────────────────────────────────┐
│  ⚡  [File] [Edit] [View]  │  [< > Code] [[|] Both] [+ Circuit]  │  ▶ Run  ⟳  │  Target  │ + Add │
├───────────────┬────────────────────────────────┬───────────────────────────────────────┤
│   WORKSPACE   │          CODE EDITOR           │            CIRCUIT CANVAS             │
│               │                                │                                       │
│ ⚡ Boards     │  [Toolbar: Flash, Reset, Step] │  • Infinite interactive canvas        │
│ 📁 Sketches   │  [Tabs: sketch.ino, + New]     │  • Grid, wire drawing, component move │
│ 📚 Libraries  │  [Monospace code editor]       │  • Floating zoom widget: [-] 100% [+] │
│               │  [Docked Serial Monitor]       │  • Slide-in Inspector on item select  │
│               │                                │                                       │
└───────────────┴────────────────────────────────┴───────────────────────────────────────┘
```

### 2.1. Top Navigation Bar (`src/navbar.rs`)
- **Brand & Menus (Left)**: Logo (`⚡`), menu buttons (`File`, `Edit`, `View`).
- **View Mode Switcher (Center)**:
  - `< > Code`: Focuses code editor across full workspace width.
  - `[|] Both`: Default split 3-column layout (Workspace + Code Editor + Circuit Canvas).
  - `+ Circuit`: Focuses schematic canvas across full space with code editor hidden.
- **Simulation Controls**:
  - `▶ Run` (Green `#22C55E`) / `⏹ Stop` (Red `#EF4444`) to toggle simulation power.
  - `⟳ Reboot` to reboot targeted MCU core.
- **Tool Shortcuts**: `📚 Libraries` (Library Manager), `📟 Serial` (terminal dock), `🔬 Scope` (instruments bench).
- **Target & Primary CTA (Right)**:
  - Target MCU selector dropdown (e.g. `ESP32-S3 #0`).
  - **`+ Add` Button**: Prominent primary button (`#0D99FF`, bold white text) opening the Add Component modal.

### 2.2. Workspace Panel (`src/workspace.rs`)
- Far-left sidebar (default width: 190px, collapsible via `View` menu).
- Displays hierarchical project structure:
  - `⚡ Circuit Boards`: List of placed boards on canvas. Clicking selects board and opens Inspector.
  - `📁 Sketches`: Open sketch tabs (`sketch1.ino`, etc.). Clicking switches active editor tab.
  - `📚 libraries.json`: Direct link to integrated Arduino Library Manager.

### 2.3. Code Editor Panel (`src/editor/`)
- Middle column in `ViewMode::Both`.
- Toolbar with action buttons (*Build & Flash*, *Reboot*, *Step*).
- Tab strip supporting multiple sketch files.
- Text area with dark theme syntax highlighting.
- Dockable bottom drawer for Serial Monitor (`src/editor/view/serial.rs`).

### 2.4. Circuit Canvas (`src/view.rs`)
- Right column in `ViewMode::Both`, central panel in `ViewMode::Circuit`.
- Infinite pan and zoom schematic canvas with grid dots.
- Floating bottom-right zoom widget: `[-] 100% [+]`.
- Floating placement banner when component is armed for placement.
- Slide-in Inspector panel (`src/inspector/`) on right edge when an item is selected.

---

## 3. Canvas Mouse Interaction Rules

The circuit canvas adheres to intuitive infinite canvas UX conventions:

| User Action | Interaction Result | Cursor Icon |
|---|---|---|
| **Hold Left-Click + Drag on Empty Canvas** | Pans the infinite canvas (`app.pan += drag_delta`) | `Grabbing` |
| **Hold Left-Click + Drag on Component** | Moves selected component or group across canvas | `Grabbing` |
| **Hover on Empty Canvas** | Ready to pan / interact | `Grab` |
| **Hover on Component Body** | Component clickable / draggable | `Grab` |
| **Hover on Pin Terminal** | Wire connection point detected | `Crosshair` |
| **Click Pin + Drag to Another Pin** | Creates schematic wire between terminals | `Crosshair` |
| **Single Left-Click on Component** | Selects component and opens Property Inspector | `PointingHand` |
| **Single Left-Click on Empty Canvas** | Deselects currently selected item (`SelectedItem::None`) | `Grab` |
| **Shift + Left-Click Drag on Canvas** | Bounding box multi-selection (`draw_selection_box`) | `Crosshair` |
| **Scroll Wheel** | Zooms in/out toward cursor position (`0.2x` to `5.0x`) | Current |
| **Right / Middle Click Drag** | Fallback secondary pan | `Grabbing` |

---

## 4. Modal Dialog: Add Component (`src/add_component/`)

When clicking the prominent `+ Add` button:

1. **Header**: Title `"Add Component"`, interactive search filter input `🔍 Search components...`, close button `✕`.
2. **Left Categories Sidebar**:
   - `All Components`, `Boards`, `Sensors`, `Output`, `Displays`, `Input`, `Motors`, `Communication`, `Passive`, `Analog`, `Logic Gates`.
   - Each category displays badge counter of items matching current search.
3. **Right Cards Grid**:
   - Responsive card grid (`src/add_component/card.rs`).
   - Cards display package preview glyph, title in bold, specs subtitle, and status badges (`Ready`, `PRO`, `Planned`).
   - Hovering card displays blue glow border (`#0D99FF`).
   - **Clicking a `Ready` item**: Immediately spawns the item at current canvas view center, selects it, and closes the modal.
   - **Clicking a `Planned` item**: Displays roadmap notification in status bar.

---

## 5. Step-by-Step Guide: How to Integrate a New Component in UI

Follow this exact 5-step workflow when adding any new component to the UI:

### Step 1: Canvas Renderer (`src/canvas/components/<name>.rs`)
- Create procedural drawing function `pub fn draw_<name>s(painter: &Painter, origin: Pos2, app: &SimulatorApp)`.
- Use high-quality visual geometry:
  - Base body geometry (solder pads, package outline, silk screen markings).
  - Lead terminals and pins matching electrical coordinates from `sim-components`.
  - Realistic state visualization (LED dome glow halos, active display pixels, switch state).
  - Selection highlight border stroke when selected (`is_selected`).

### Step 2: Property Inspector Panel (`src/inspector/<name>.rs`)
- Create inspector renderer `pub fn render_<name>_inspector(ui: &mut Ui, app: &mut SimulatorApp, index: usize)`.
- Display component title, subtitle, interactive controls (color picker, value slider, flip polarity).
- Display real-time electrical telemetry (connected pin voltages, forward drop, logic levels).
- Provide delete button (`app.delete_selected()`).
- Expose in `src/inspector/mod.rs`.

### Step 3: Add Component Catalog Registration (`src/add_component/`)
- In `src/add_component/catalog.rs`: Add item to `CATALOG_ITEMS`:
  ```rust
  ComponentItem {
      id: "<name>",
      name: "<Human Name>",
      category: CategoryKind::<Category>,
      specs: "<Key Specifications & Ratings>",
      badge: Some("Ready"),
      is_pro: false,
      is_ready: true,
      spawn: SpawningComponent::<Variant>,
  }
  ```
- In `src/add_component/card.rs`: Add thumbnail preview drawing in `draw_card_preview()`.

### Step 4: Selection & Catalog Registry (`src/registry.rs`)
- Add `Spec` entry in `CATALOG` slice:
  - `make`: Builder for `SelectedItem::<Name>(index)`.
  - `index`: Extractor for `SelectedItem::<Name>`.
  - `count`: Instance count in `SimulatorApp`.
  - `pos`: Canvas position accessor.
  - `bounds`: Hit-testing box size (`Vec2`).
  - `pins`: Pin IDs accessor.
  - `remove`: Removal handler.
  - `move_by`: Canvas displacement handler.

### Step 5: Canvas Spawning Handler (`src/state/spawn.rs`)
- In `spawn_component_at(app: &mut SimulatorApp, local_pos: Pos2)`:
  - Allocate next deterministic pin IDs using `next_pin_id(app)`.
  - Instantiate component from `sim-components`.
  - Push `(local_pos, instance)` to app storage vector.
  - Set `app.selected = SelectedItem::<Name>(new_index)`.
  - Reset `app.spawning = SpawningComponent::None`.
  - Call `app.rebuild_netlist()`.

---

## 6. Visual Design Tokens & Antislop Principles

`sim-ui` strictly follows the `antislop-ui` design standard:

- **Color Discipline**:
  - Main background: `#18191c` (panels), `#121316` (canvas).
  - Surface cards & inputs: `#22242a` with `#2e313a` border.
  - Text: Primary `#f0f2f5`, secondary `#9ca3af`, muted `#6b7280`.
  - **Single Intentional Accent**: `#0D99FF` (Electric Blue) reserved strictly for primary CTA (`+ Add`, selection highlights, active tabs).
- **Prohibited Antislop Patterns**:
  - No generic blue-purple or rainbow gradients.
  - No excessive glassmorphism blur over every panel.
  - No muddy floating shadows across all components.
  - No uniform pill shapes for non-pill elements. Use crisp 4.0px radius for cards and buttons.

---

## 7. Mandatory Engineering Rules

1. **Rule 5 (No God Files)**: Every `.rs` file in `sim-ui` must target less than 300 lines of functional code. If a module grows, decompose into submodules (e.g. `navbar.rs`, `workspace.rs`, `dialog.rs`).
2. **Rule 6 (No Comments Inside Function Bodies)**: Code must be self-explanatory through clean types, clear variables, and small focused functions. Comments inside function bodies are forbidden.
3. **Rule 7 & 8 (Doc Comments Above Declarations)**: Intent and architectural contracts must be documented using `///` directly above function declarations.

# Design Direction: Electronics Workbench & EDA Simulator

## Identity & Purpose
- Product: Modular Rust Circuit & Microcontroller Simulator (desktop native & WebAssembly).
- Audience: Embedded firmware developers, electrical engineering students, and makers.
- Mood: Professional Electronic Design Automation (EDA) CAD workbench. High-contrast, precise vector schematic representation, utilitarian, and distraction-free.

## Dials
- Dial: ENERGY 2 / RHYTHM 2 / MOTION 1

## Visual Language & Palette
- Background / Canvas: Dark slate workbench (`#1e1e24`) with subtle geometric grid lines.
- Component Body: Professional IC matte charcoal (`#2a2e39`) with sharp/crisp outline stroke.
- Signal States (Functional, never purely decorative):
  - Digital High (3.3V / 5V): Crisp Emerald (`#4caf50`)
  - Digital Low (GND): Cobalt Blue (`#2196f3`)
  - High-Impedance / Floating: Neutral Slate (`#78909c`)
  - Analog / Clock: Amber (`#ffb300`)
- Pin Terminals: Clear circular terminals with distinct connection rings and pin function labels.
- Typography:
  - Labels & Titles: Clean sans-serif / proportional font.
  - Pin IDs, Bus Addresses, Bit Registers, Values: Monospace font for exact tabular alignment.

## Functional Standards
- Every UI element (knob, button, terminal, input field) must perform a real simulation action.
- Zero fake glow or decorative mesh gradients.
- Interactive states (hover, dragging, wire routing) give instant feedback.

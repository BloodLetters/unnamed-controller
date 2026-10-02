# sim-core Development Guidelines & Engine Blueprint

## 1. Purpose & Scope

`sim-core` is the foundational, platform-agnostic simulation engine. It provides the netlist solver, pin resolution, electrical analog analysis, clock scheduling, peripheral buses, and virtual firmware toolchains.

**Platform Independence**: `sim-core` must never depend on `egui`, `eframe`, browser DOM, or operating-system-specific APIs. All logic must compile identically for native desktop and WebAssembly.

---

## 2. Core Architecture & Subsystems

```text
sim-core/
├── netlist/     # Electrical graph connectivity, NodeId, PinId
├── pins/        # DigitalState, PinDrive, Resolver logic
├── analog/      # Nodal matrix analysis, transient RC/RL solver
├── bus/         # I2C, SPI, UART bus interfaces
├── clock/       # Discrete event scheduler and simulation time
├── audio/       # Waveform synthesis, DAC, RTTTL parser
├── firmware/    # Built-in compiler, opcodes, bytecode emitter
├── debugger/    # Disassembler, breakpoints, GDB RSP server
└── instruments/ # Digital multimeter, PWM analyzer, logic analyzer
```

---

## 3. Netlist & Pin Resolution Rules

1. **Deterministic Authority**: The netlist is the single source of truth for circuit connectivity. UI layers must not duplicate connectivity data.
2. **Node Resolution**:
   - Pins connected together form an electrical `NodeId`.
   - The resolver computes node state based on all contributing drives:
     - `PinDrive::Driven(High/Low)` overpowers pull resistors.
     - `PinDrive::PullUp` pulls floating nodes to High.
     - `PinDrive::PullDown` pulls floating nodes to Low.
     - Conflicting strong drives produce an electrical contention state.
3. **Strong Typing**: Use distinct wrapper types (`PinId(u32)`, `NodeId(u32)`) instead of raw integers.

---

## 4. Bus Communication Architecture

Components communicating over serial buses implement domain interfaces:

### I2C Bus (`src/bus/i2c.rs`)
- Slave devices implement `I2cDevice` (`address()`, `on_write()`, `on_read()`).
- The bus manager (`I2cBus`) routes transactions from firmware or MCUs to addressed peripherals.

### SPI Bus (`src/bus/spi.rs`)
- High-speed master-slave byte transfers with Chip Select lines.

---

## 5. Firmware Bytecode Toolchain (`src/firmware/`)

`sim-core` includes a built-in virtual sketch compiler:
1. **Parser (`parser.rs`)**: Strips comments, parses C/Arduino statements (`void setup`, `void loop`, `digitalWrite`, `digitalRead`, `delay`).
2. **Intermediate Representation (`types.rs`)**: Generates structured intermediate ops with jump labels.
3. **Emitter (`emitter.rs`)**: Resolves jump offsets and outputs compact binary bytecode using opcodes from `opcodes.rs`.
4. **Virtual Opcodes**: Standardized byte commands (`OP_SET_HIGH`, `OP_SET_LOW`, `OP_DELAY`, `OP_JUMP`, `OP_READ_PIN`, `OP_UART_SEND`, `OP_HALT`).

---

## 6. Performance & Testing Rules

1. **Hot Path Zero-Allocation**: The simulation step (`tick()`) must not allocate heap memory (`Vec::new()`, `String::new()`, `Box`) on every cycle. Buffers must be pre-allocated and reused.
2. **No Comments in Function Bodies (Rule 6)**: Keep function bodies clean and self-explanatory. Document contracts with doc comments `///` above function signatures.
3. **File Size Target (Rule 5)**: Target less than 300 functional lines per file.
4. **Test Coverage**: All netlist transitions, analog transient steps, and compiler opcodes must have deterministic unit tests in `tests/`.

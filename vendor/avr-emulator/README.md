# avr-emulator

`avr-emulator` is an embeddable, no-heap Rust AVR CPU emulator. It is
`no_std` by default and has no runtime dependency on an allocator, OpenSSL, or
any other C library.

The implementation is based on the official [Microchip AVR Instruction Set
Manual](https://onlinedocs.microchip.com/oxy/GUID-0B644D8F-67E7-49E6-82C9-1B2B9ABE6A0D-en-US-23/index.html).
The two local AVR projects were used as compatibility references only; their
missing instructions and incorrect behavior are not treated as specification.

## Supported cores

`Profile` selects the core capabilities and timing model:

- `Profile::avr()` — AVR core
- `Profile::avre()` — AVRe core
- `Profile::avre_plus()` — AVRe+ with a default 22-bit word PC
- `Profile::avrxm()` — AVRxm/XMEGA
- `Profile::avrxt()` — modern AVRxt (tinyAVR/Dx family)
- `Profile::avrrc()` — deliberately rejected; AVRrc is not partially modeled

Use `with_program_counter_bits(16)` or `(22)` for a device-specific PC size.
The return address is two bytes for a 16-bit word PC and three bytes for a
22-bit word PC. `RAMPX`, `RAMPY`, `RAMPZ`, `RAMPD`, and `EIND` are enabled by
the selected profile rather than by a device-name list.

## Embedding

The CPU has no memory image of its own:

```rust
use avr_emulator::{Cpu, NoHooks, Profile, SliceData, SliceIo, SliceProgram};

let words = [0xE005, 0x9508]; // ldi r16, 5; ret
let mut ram = [0u8; 0x100];
let mut ports = [0u8; 64];
let mut program = SliceProgram::new(&words);
let mut data = SliceData::new(&mut ram);
let mut io = SliceIo::new(&mut ports);
let mut hooks = NoHooks;
let mut cpu = Cpu::new(Profile::avre());

let result = cpu.step(&mut program, &mut data, &mut io, &mut hooks).unwrap();
assert_eq!(result.pc_after, 1);
assert_eq!(cpu.register(16), 5);
```

Custom hosts implement four small interfaces:

- `ProgramBus` reads word-addressed program memory.
- `DataBus` reads and writes data outside the CPU register file and I/O space.
- `IoBus` handles direct six-bit I/O access.
- `ExecutionHooks` receives SPM/NVM, sleep, BREAK, and watchdog events.

The CPU maps architectural registers and the standard SREG/SP/RAMP/EIND I/O
locations internally. Device peripherals remain the host's responsibility.
`Cpu::interrupt_entry` pushes the profile-sized return address, clears I, and
enters a word-addressed vector when I was set.

## Instruction coverage

The decoder and executor cover every real opcode/form currently emitted by
Ezra, including all register ALU forms, aliases, bit/branch/skip forms,
16-bit and two-word data access, X/Y/Z direct/pre/post/displacement modes,
LPM/ELPM, SPM and SPM Z+, indirect and extended control transfer, XCH/LAS/LAC/
LAT, DES, SLEEP, BREAK, and WDR.

Unknown and reserved first words return `DecodeError::Reserved`. Encodings
belonging to a known instruction that the selected core does not provide return
`DecodeError::Unsupported`. The documented undefined operand cases for
pre/post-increment loads/stores where the destination overlaps the pointer,
and `ELPM r30/r31,Z+`, return `StepError::UndefinedOperand` instead of silently
choosing reference-specific behavior.

DES is implemented in pure Rust. AVR DES performs one DES round and applies the
initial and inverse initial permutations on every instruction, so sixteen calls
with K=0..15 form one operation. The H flag selects encryption or decryption.

## Validation

The crate includes exhaustive first-word decoder classification tests, field
bounds, profile rejection, instruction execution tests, stack and PC wrapping,
addressing modes, skip lengths over one- and two-word instructions, bus and
execution hooks, interrupt entry, flags, DES, illegal/reserved encodings, and a
deterministic ignored-decode/execute benchmark test that prints checksums and
throughput.

Run from this directory:

```text
cargo fmt --all
cargo test --all-targets
cargo check --no-default-features
cargo clippy --all-targets --all-features -- -D warnings
cargo test --test benchmarks -- --nocapture
cargo publish --dry-run
```

## License

BSD-3-Clause. See `LICENSE`.

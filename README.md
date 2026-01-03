# cpu

This is the approach to create a minimal RISC-V CPU in JavaScript, later in C and eventually in hardware.
I wanted to do this for a long time, but never had the time to do it. Now I still don't have the time and I am doing it.

## Goals

- Demonstrate how js code could be mapped 1:1 to c code
- Demonstrate how c code could be mapped 1:1 to hardware
- Use most primitive hardware components

## Supported Extensions

This CPU implements the **RV32I** base instruction set, which includes:

- **Load Instructions**: `lb`, `lh`, `lw`, `lbu`, `lhu`
- **Store Instructions**: `sb`, `sh`, `sw`
- **Arithmetic/Logical (Immediate)**: `addi`, `slli`, `slti`, `sltiu`, `xori`, `srli`, `srai`, `ori`, `andi`
- **Upper Immediate**: `auipc`, `lui`
- **Arithmetic/Logical (Register)**: `add`, `sub`, `sll`, `slt`, `sltu`, `xor`, `srl`, `sra`, `or`, `and`
- **Branch Instructions**: `beq`, `bne`, `blt`, `bge`, `bltu`, `bgeu`
- **Jump Instructions**: `jal`, `jalr`

## How to use

To compile the test programs, run `make` or `make tests`.
Then run it with one of the implementations:
- JavaScript: `node js/main.js tests/calc.bin`
- C: `make c && c/main tests/calc.bin`
- Rust: `make rust && rust/target/debug/cpu tests/calc.bin`
- Zig: `make zig && zig/zig-out/bin/cpu tests/calc.bin`

## Contribution
Contributions are welcome! If you would like to contribute, please feel free to submit a pull request.
However, be prepared for nitpicking as I am a perfectionistic dictator.

## License
zlib

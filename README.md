# cpu

This is the approach to create a minimal RISC-V CPU in JavaScript, later in C and eventually in hardware.
I wanted to do this for a long time, but never had the time to do it. Now I still don't have the time and I am doing it.

## Goals

- Demonstrate how js code could be mapped 1:1 to c code
- Demonstrate how c code could be mapped 1:1 to hardware
- Use most primitive hardware components

## Supported Extensions

This CPU implements the **RV32G** instruction set, which includes:
- **RV32I**: Base instruction set
- **RV32M**: Integer multiplication and division  
- **RV32A**: Atomic instructions
- **RV32F**: Single-precision floating-point
- **RV32D**: Double-precision floating-point

### RV32I Base Instructions
- **Load Instructions**: `lb`, `lh`, `lw`, `lbu`, `lhu`
- **Store Instructions**: `sb`, `sh`, `sw`
- **Arithmetic/Logical (Immediate)**: `addi`, `slli`, `slti`, `sltiu`, `xori`, `srli`, `srai`, `ori`, `andi`
- **Upper Immediate**: `auipc`, `lui`
- **Arithmetic/Logical (Register)**: `add`, `sub`, `sll`, `slt`, `sltu`, `xor`, `srl`, `sra`, `or`, `and`
- **Branch Instructions**: `beq`, `bne`, `blt`, `bge`, `bltu`, `bgeu`
- **Jump Instructions**: `jal`, `jalr`

### RV32M Extension (Integer Multiplication and Division)
- **Multiplication**: `mul`, `mulh`, `mulhsu`, `mulhu`
- **Division**: `div`, `divu`
- **Remainder**: `rem`, `remu`

### RV32A Extension (Atomic Instructions)
- **Load-Reserved/Store-Conditional**: `lr.w`, `sc.w`
- **Atomic Memory Operations**: `amoadd.w`, `amoswap.w`, `amoxor.w`, `amoand.w`, `amoor.w`
- **Atomic Min/Max**: `amomin.w`, `amomax.w`, `amominu.w`, `amomaxu.w`

### RV32F Extension (Single-Precision Floating-Point)
- **Load/Store**: `flw`, `fsw`
- **Arithmetic**: `fadd.s`, `fsub.s`, `fmul.s`, `fdiv.s`, `fsqrt.s`
- **Fused Multiply-Add**: `fmadd.s`, `fmsub.s`, `fnmsub.s`, `fnmadd.s`
- **Conversion**: `fcvt.w.s`, `fcvt.wu.s`, `fcvt.s.w`, `fcvt.s.wu`, `fmv.x.w`, `fmv.w.x`
- **Comparison**: `feq.s`, `flt.s`, `fle.s`
- **Sign Injection**: `fsgnj.s`, `fsgnjn.s`, `fsgnjx.s`
- **Min/Max**: `fmin.s`, `fmax.s`
- **Classification**: `fclass.s`

### RV32D Extension (Double-Precision Floating-Point)
- **Load/Store**: `fld`, `fsd`
- **Arithmetic**: `fadd.d`, `fsub.d`, `fmul.d`, `fdiv.d`, `fsqrt.d`
- **Fused Multiply-Add**: `fmadd.d`, `fmsub.d`, `fnmsub.d`, `fnmadd.d`
- **Conversion**: `fcvt.w.d`, `fcvt.wu.d`, `fcvt.d.w`, `fcvt.d.wu`, `fcvt.s.d`, `fcvt.d.s`
- **Comparison**: `feq.d`, `flt.d`, `fle.d`
- **Sign Injection**: `fsgnj.d`, `fsgnjn.d`, `fsgnjx.d`
- **Min/Max**: `fmin.d`, `fmax.d`
- **Classification**: `fclass.d`

## How to use

To compile the test programs, run `make` or `make tests`.
Then run it with one of the implementations:
- JavaScript: `node js/main.js tests/calc.bin`
- C: `make c && c/main tests/calc.bin`
- Rust: `make rust && rust/target/debug/cpu tests/calc.bin`

## Contribution
Contributions are welcome! If you would like to contribute, please feel free to submit a pull request.
However, be prepared for nitpicking as I am a perfectionistic dictator.

## License
zlib

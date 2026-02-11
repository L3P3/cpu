# M Extension Implementation Guide

This document outlines the general approach for implementing the RISC-V M extension (integer multiplication and division) in this CPU emulator.

## General Approach

1. **Implement in JavaScript First**
   - Start with the JavaScript port (`js/main.js`)
   - Test thoroughly with assembly files
   - JavaScript provides easier debugging and faster iteration

2. **Test with Assembly Files**
   - Create comprehensive test cases in `tests/` directory
   - Use binutils-riscv64-unknown-elf package for building tests
   - Install with: `sudo apt-get install binutils-riscv64-unknown-elf`
   - Build tests with: `make tests`

3. **Port to C and Rust**
   - Only proceed after JavaScript implementation is working correctly
   - Port to C (`c/main.c`)
   - Port to Rust (`rust/main.rs`)
   - Maintain 1:1 mapping with JavaScript implementation

## M Extension Instructions

The M extension adds 8 instructions for multiplication and division:

### Multiplication Instructions
- **MUL**: Multiply (lower 32 bits)
- **MULH**: Multiply High (upper 32 bits, signed × signed)
- **MULHSU**: Multiply High Signed-Unsigned (upper 32 bits, signed × unsigned)
- **MULHU**: Multiply High Unsigned (upper 32 bits, unsigned × unsigned)

### Division Instructions
- **DIV**: Signed division
- **DIVU**: Unsigned division
- **REM**: Signed remainder
- **REMU**: Unsigned remainder

## Instruction Encoding

All M extension instructions use R-type format with:
- **opcode**: `0110011` (same as base integer operations)
- **funct7**: `0000001` (distinguishes M instructions)
- **funct3**: Varies by instruction (000-111)

| Instruction | funct7   | funct3 |
|-------------|----------|--------|
| MUL         | 0000001  | 000    |
| MULH        | 0000001  | 001    |
| MULHSU      | 0000001  | 010    |
| MULHU       | 0000001  | 011    |
| DIV         | 0000001  | 100    |
| DIVU        | 0000001  | 101    |
| REM         | 0000001  | 110    |
| REMU        | 0000001  | 111    |

## Implementation Notes

### Division by Zero
- DIV/DIVU: Return -1 (all bits set)
- REM/REMU: Return dividend (rs1)

### Overflow Handling
- Division of most negative number by -1:
  - DIV: Return most negative number
  - REM: Return 0

### Multiplication
- JavaScript doesn't natively support 64-bit integers
- Use Math.imul() for 32-bit multiplication when possible
- For high bits, need careful handling with BigInt or manual calculation

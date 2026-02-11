# General Approach

When working on the cpu, always stick to the following pattern:

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

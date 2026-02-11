# General Approach

When working on the cpu, always stick to the following pattern:

1. **Implement in JavaScript First**
   - Start with the JavaScript port (`js/main.js`)
   - Strictly stick to the code style already present
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

4. **Version bump**
   - For every pull request, update the version in `js/package.json`
   - For new features, update the minor part
   - For fixes/refactorings, update the patch part
   - Copy that version to `rust/Cargo.toml`

5. **Unwanted files**
   - Do not create or commit _codeql_... or similar files without being explicitly asked for

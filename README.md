# cpu

This is the approach to create a minimal RISC-V CPU in JavaScript, later in C and eventually in hardware.

## goals

- demonstrate how js code could be mapped 1:1 to c code
- demonstrate how c code could be mapped 1:1 to hardware
- use most primitive hardware components

# how to use

To compile the test programs, run `make`.
Then run it like `node js/main.js tests/calc.bin`.

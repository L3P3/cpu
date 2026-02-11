use std::env;
use std::fs::File;
use std::io::Read;
use std::time::Instant;

const MEMORY_SIZE: usize = 64 * 1024;

// Out of bounds bit masks for memory access validation
// Any address with these bits set is out of bounds
const OOB_BITS_8: i32 = !(MEMORY_SIZE as i32 - 1);        // byte access
const OOB_BITS_16: i32 = !(MEMORY_SIZE as i32 - 1 - 1);   // halfword access
const OOB_BITS_32: i32 = !(MEMORY_SIZE as i32 - 1 - 3);   // word access
const OOB_BITS_PC: u32 = !((MEMORY_SIZE / 4) as u32 - 1); // program counter (word index)

struct CPU {
    registers: [i32; 32],
    memory8: Vec<u8>,
    program_counter: u32,
    program_ended: bool,
    error_message: Option<&'static str>,
}

impl CPU {
    fn new() -> Self {
        CPU {
            registers: [0; 32],
            memory8: vec![0; MEMORY_SIZE],
            program_counter: 0,
            program_ended: false,
            error_message: None,
        }
    }

    #[inline(always)]
    fn memory16(&self) -> &[u16] {
        unsafe {
            std::slice::from_raw_parts(
                self.memory8.as_ptr() as *const u16,
                MEMORY_SIZE / 2,
            )
        }
    }

    #[inline(always)]
    fn memory16_mut(&mut self) -> &mut [u16] {
        unsafe {
            std::slice::from_raw_parts_mut(
                self.memory8.as_mut_ptr() as *mut u16,
                MEMORY_SIZE / 2,
            )
        }
    }

    #[inline(always)]
    fn memory32(&self) -> &[i32] {
        unsafe {
            std::slice::from_raw_parts(
                self.memory8.as_ptr() as *const i32,
                MEMORY_SIZE / 4,
            )
        }
    }

    #[inline(always)]
    fn memory32_mut(&mut self) -> &mut [i32] {
        unsafe {
            std::slice::from_raw_parts_mut(
                self.memory8.as_mut_ptr() as *mut i32,
                MEMORY_SIZE / 4,
            )
        }
    }

    #[inline(always)]
    fn registers_unsigned(&self, index: usize) -> u32 {
        self.registers[index] as u32
    }

    fn tick(&mut self) {
        // make it constant
        self.registers[0] = 0;

        let instruction = self.memory32()[self.program_counter as usize] as u32;

        let funct3 = (instruction >> 12) & 0b111;

        let register_destination = ((instruction >> 7) & 0b11111) as usize;
        let register_source1 = ((instruction >> 15) & 0b11111) as usize;
        let register_source2 = ((instruction >> 20) & 0b11111) as usize;

        let opcode_combined = ((((instruction >> 2) << 3) & 0xff) | funct3) as u8;

        match opcode_combined {
            // load
            0b00000000 => {
                // lb
                let addr = self.registers[register_source1]
                    .wrapping_add((instruction as i32) >> 20);
                if addr & OOB_BITS_8 != 0 {
                    self.error_message = Some("out of bounds");
                    return;
                }
                self.registers[register_destination] = self.memory8[addr as usize] as i8 as i32;
            }
            0b00000001 => {
                // lh
                let addr = self.registers[register_source1]
                    .wrapping_add((instruction as i32) >> 20);
                if addr & OOB_BITS_16 != 0 {
                    self.error_message = Some("out of bounds");
                    return;
                }
                self.registers[register_destination] =
                    self.memory16()[(addr >> 1) as usize] as i16 as i32;
            }
            0b00000010 => {
                // lw
                let addr = self.registers[register_source1]
                    .wrapping_add((instruction as i32) >> 20);
                if addr & OOB_BITS_32 != 0 {
                    self.error_message = Some("out of bounds");
                    return;
                }
                self.registers[register_destination] = self.memory32()[(addr >> 2) as usize];
            }
            0b00000100 => {
                // lbu
                let addr = self.registers[register_source1]
                    .wrapping_add((instruction as i32) >> 20);
                if addr & OOB_BITS_8 != 0 {
                    self.error_message = Some("out of bounds");
                    return;
                }
                self.registers[register_destination] = self.memory8[addr as usize] as i32;
            }
            0b00000101 => {
                // lhu
                let addr = self.registers[register_source1]
                    .wrapping_add((instruction as i32) >> 20);
                if addr & OOB_BITS_16 != 0 {
                    self.error_message = Some("out of bounds");
                    return;
                }
                self.registers[register_destination] = self.memory16()[(addr >> 1) as usize] as i32;
            }
            // fence
            // register+immediate
            0b00100000 => {
                // addi
                self.registers[register_destination] = self.registers[register_source1]
                    .wrapping_add((instruction as i32) >> 20);
            }
            0b00100001 => {
                // slli
                self.registers[register_destination] =
                    self.registers[register_source1] << ((instruction >> 20) & 0b11111);
            }
            0b00100010 => {
                // slti
                self.registers[register_destination] =
                    if self.registers[register_source1] < ((instruction as i32) >> 20) {
                        1
                    } else {
                        0
                    };
            }
            0b00100011 => {
                // sltiu
                self.registers[register_destination] =
                    if self.registers_unsigned(register_source1) < (instruction >> 20) {
                        1
                    } else {
                        0
                    };
            }
            0b00100100 => {
                // xori
                self.registers[register_destination] =
                    self.registers[register_source1] ^ ((instruction as i32) >> 20);
            }
            0b00100101 => {
                // srli/srai
                let shift_by = (instruction >> 20) & 0b11111;
                if instruction >> 30 != 0 {
                    self.registers[register_destination] =
                        self.registers[register_source1] >> shift_by;
                } else {
                    self.registers[register_destination] =
                        (self.registers_unsigned(register_source1) >> shift_by) as i32;
                }
            }
            0b00100110 => {
                // ori
                self.registers[register_destination] =
                    self.registers[register_source1] | ((instruction as i32) >> 20);
            }
            0b00100111 => {
                // andi
                self.registers[register_destination] =
                    self.registers[register_source1] & ((instruction as i32) >> 20);
            }
            0b00101000 | 0b00101001 | 0b00101010 | 0b00101011 | 0b00101100 | 0b00101101
            | 0b00101110 | 0b00101111 => {
                // auipc
                self.registers[register_destination] = ((self.program_counter << 2) as i32)
                    .wrapping_add((instruction & 0xfffff000) as i32);
            }
            // store
            0b01000000 => {
                // sb
                let addr = self.registers[register_source1].wrapping_add(
                    (((instruction as i32) >> 25) << 5) | (register_destination as i32),
                );
                if addr & OOB_BITS_8 != 0 {
                    self.error_message = Some("out of bounds");
                    return;
                }
                self.memory8[addr as usize] = self.registers[register_source2] as u8;
            }
            0b01000001 => {
                // sh
                let addr = self.registers[register_source1].wrapping_add(
                    (((instruction as i32) >> 25) << 5) | (register_destination as i32),
                );
                if addr & OOB_BITS_16 != 0 {
                    self.error_message = Some("out of bounds");
                    return;
                }
                self.memory16_mut()[(addr >> 1) as usize] = self.registers[register_source2] as u16;
            }
            0b01000010 => {
                // sw
                let addr = self.registers[register_source1].wrapping_add(
                    (((instruction as i32) >> 25) << 5) | (register_destination as i32),
                );
                if addr & OOB_BITS_32 != 0 {
                    self.error_message = Some("out of bounds");
                    return;
                }
                self.memory32_mut()[(addr >> 2) as usize] = self.registers[register_source2];
            }
            // register+register
            0b01100000 => {
                // add/sub/mul
                if instruction >> 25 & 1 != 0 {
                    // mul - multiply lower 32 bits
                    self.registers[register_destination] = self.registers[register_source1]
                        .wrapping_mul(self.registers[register_source2]);
                } else {
                    self.registers[register_destination] = if instruction >> 30 != 0 {
                        self.registers[register_source1].wrapping_sub(self.registers[register_source2])
                    } else {
                        self.registers[register_source1].wrapping_add(self.registers[register_source2])
                    };
                }
            }
            0b01100001 => {
                // sll/mulh
                if instruction >> 25 & 1 != 0 {
                    // mulh - multiply high signed
                    let a = self.registers[register_source1] as i64;
                    let b = self.registers[register_source2] as i64;
                    let result = a.wrapping_mul(b);
                    self.registers[register_destination] = (result >> 32) as i32;
                } else {
                    self.registers[register_destination] =
                        self.registers[register_source1] << (self.registers[register_source2] & 0b11111);
                }
            }
            0b01100010 => {
                // slt/mulhsu
                if instruction >> 25 & 1 != 0 {
                    // mulhsu - multiply high signed × unsigned
                    let a = self.registers[register_source1] as i64;
                    let b = self.registers_unsigned(register_source2) as u64 as i64;
                    let result = a.wrapping_mul(b);
                    self.registers[register_destination] = (result >> 32) as i32;
                } else {
                    self.registers[register_destination] =
                        if self.registers[register_source1] < self.registers[register_source2] {
                            1
                        } else {
                            0
                        };
                }
            }
            0b01100011 => {
                // sltu/mulhu
                if instruction >> 25 & 1 != 0 {
                    // mulhu - multiply high unsigned
                    let a = self.registers_unsigned(register_source1) as u64;
                    let b = self.registers_unsigned(register_source2) as u64;
                    let result = a.wrapping_mul(b);
                    self.registers[register_destination] = (result >> 32) as i32;
                } else {
                    self.registers[register_destination] = if self.registers_unsigned(register_source1)
                        < self.registers_unsigned(register_source2)
                    {
                        1
                    } else {
                        0
                    };
                }
            }
            0b01100100 => {
                // xor/div
                if instruction >> 25 & 1 != 0 {
                    // div - signed division
                    let dividend = self.registers[register_source1];
                    let divisor = self.registers[register_source2];
                    if divisor == 0 {
                        self.registers[register_destination] = -1;
                    } else if dividend == i32::MIN && divisor == -1 {
                        // overflow case
                        self.registers[register_destination] = i32::MIN;
                    } else {
                        self.registers[register_destination] = dividend.wrapping_div(divisor);
                    }
                } else {
                    self.registers[register_destination] =
                        self.registers[register_source1] ^ self.registers[register_source2];
                }
            }
            0b01100101 => {
                // srl/sra/divu
                if instruction >> 25 & 1 != 0 {
                    // divu - unsigned division
                    let dividend = self.registers_unsigned(register_source1);
                    let divisor = self.registers_unsigned(register_source2);
                    if divisor == 0 {
                        self.registers[register_destination] = -1;
                    } else {
                        self.registers[register_destination] = dividend.wrapping_div(divisor) as i32;
                    }
                } else {
                    let shift_by = self.registers[register_source2] & 0b11111;
                    if instruction >> 30 != 0 {
                        self.registers[register_destination] =
                            self.registers[register_source1] >> shift_by;
                    } else {
                        self.registers[register_destination] =
                            (self.registers_unsigned(register_source1) >> shift_by) as i32;
                    }
                }
            }
            0b01100110 => {
                // or/rem
                if instruction >> 25 & 1 != 0 {
                    // rem - signed remainder
                    let dividend = self.registers[register_source1];
                    let divisor = self.registers[register_source2];
                    if divisor == 0 {
                        self.registers[register_destination] = dividend;
                    } else if dividend == i32::MIN && divisor == -1 {
                        // overflow case
                        self.registers[register_destination] = 0;
                    } else {
                        self.registers[register_destination] = dividend.wrapping_rem(divisor);
                    }
                } else {
                    self.registers[register_destination] =
                        self.registers[register_source1] | self.registers[register_source2];
                }
            }
            0b01100111 => {
                // and/remu
                if instruction >> 25 & 1 != 0 {
                    // remu - unsigned remainder
                    let dividend = self.registers_unsigned(register_source1);
                    let divisor = self.registers_unsigned(register_source2);
                    if divisor == 0 {
                        self.registers[register_destination] = dividend as i32;
                    } else {
                        self.registers[register_destination] = dividend.wrapping_rem(divisor) as i32;
                    }
                } else {
                    self.registers[register_destination] =
                        self.registers[register_source1] & self.registers[register_source2];
                }
            }
            0b01101000 | 0b01101001 | 0b01101010 | 0b01101011 | 0b01101100 | 0b01101101
            | 0b01101110 | 0b01101111 => {
                // lui ;)
                self.registers[register_destination] = (instruction & 0xfffff000) as i32;
            }
            0b11000000 | 0b11000001 | 0b11000100 | 0b11000101 | 0b11000110 | 0b11000111 => {
                // branch
                let should_branch = match funct3 {
                    0b000 => {
                        // beq
                        self.registers[register_source1] == self.registers[register_source2]
                    }
                    0b001 => {
                        // bne
                        self.registers[register_source1] != self.registers[register_source2]
                    }
                    0b100 => {
                        // blt
                        self.registers[register_source1] < self.registers[register_source2]
                    }
                    0b101 => {
                        // bge
                        self.registers[register_source1] >= self.registers[register_source2]
                    }
                    0b110 => {
                        // bltu
                        self.registers_unsigned(register_source1)
                            < self.registers_unsigned(register_source2)
                    }
                    0b111 => {
                        // bgeu
                        self.registers_unsigned(register_source1)
                            >= self.registers_unsigned(register_source2)
                    }
                    _ => {
                        self.error_message = Some("invalid branch condition");
                        return;
                    }
                };

                if !should_branch {
                    self.program_counter = self.program_counter.wrapping_add(1);
                    if self.program_counter & OOB_BITS_PC != 0 {
                        self.error_message = Some("out of bounds");
                    }
                    return;
                }

                self.program_counter = self.program_counter.wrapping_add(
                    (((instruction as i32) >> 31) << 10
                        | ((register_destination as i32) & 0x1) << 9
                        | ((instruction >> 25) << 3) as i32
                        | (register_destination >> 2) as i32) as u32,
                );
                if self.program_counter & OOB_BITS_PC != 0 {
                    self.error_message = Some("out of bounds");
                }
                return;
            }
            0b11001000 => {
                // jalr
                self.registers[register_destination] = ((self.program_counter + 1) << 2) as i32;
                self.program_counter = (self.registers[register_source1]
                    .wrapping_add((instruction as i32) >> 20)
                    >> 2) as u32;
                if self.program_counter & OOB_BITS_PC != 0 {
                    self.error_message = Some("out of bounds");
                }
                return;
            }
            0b11011000 | 0b11011001 | 0b11011010 | 0b11011011 | 0b11011100 | 0b11011101
            | 0b11011110 | 0b11011111 => {
                // jal
                // exit on endless loop
                if instruction >> 12 == 0 {
                    self.program_ended = true;
                    return;
                }
                self.registers[register_destination] = ((self.program_counter + 1) << 2) as i32;
                self.program_counter = self.program_counter.wrapping_add(
                    (((instruction as i32) >> 31) << 18
                        | (((instruction >> 12) & 0xff) << 10) as i32
                        | (((instruction >> 20) & 0x1) << 9) as i32
                        | ((instruction >> 22) & 0x3ff) as i32) as u32,
                );
                if self.program_counter & OOB_BITS_PC != 0 {
                    self.error_message = Some("out of bounds");
                }
                return;
            }
            _ => {
                self.error_message = Some("illegal instruction");
                return;
            }
        }

        self.program_counter = self.program_counter.wrapping_add(1);
        if self.program_counter & OOB_BITS_PC != 0 {
            self.error_message = Some("out of bounds");
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program_path = if args.len() > 1 {
        &args[1]
    } else {
        "../tests/count.bin"
    };

    println!("loading {}", program_path);

    let mut file = File::open(program_path).expect("Failed to open file");
    let mut cpu = CPU::new();
    file.read(&mut cpu.memory8)
        .expect("Failed to read file");

    println!("running");

    let time_start = Instant::now();
    let mut instruction_count = 0u32;

    loop {
        cpu.tick();

        if cpu.program_ended {
            println!("-----\nprogram ended");
            break;
        }

        if let Some(error) = cpu.error_message {
            println!("-----\nprogram failed: {}", error);
            break;
        }

        instruction_count += 1;
        if instruction_count >= 10_000_000 {
            println!("-----\nprogram timed out");
            break;
        }
    }

    let runtime = time_start.elapsed().as_secs_f64() * 1000.0;

    println!(
        "ran {} instructions in {:.0} ms",
        instruction_count, runtime
    );
    println!(
        "execution speed: {:.0} MHz",
        instruction_count as f64 / runtime / 1000.0
    );

    println!("registers:");
    for i in 1..32 {
        println!(
            "  {:>3} = 0x{:08x} {}",
            format!("x{}", i),
            cpu.registers[i] as u32,
            cpu.registers[i]
        );
    }
}

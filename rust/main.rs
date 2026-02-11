use std::env;
use std::fs::File;
use std::io::Read;
use std::time::Instant;

const MEMORY_SIZE: usize = 64 * 1024;

// Out of bounds bit masks for memory access validation
// Any address with these bits set is out of bounds
const OOB_BITS_8: i32 = !(MEMORY_SIZE as i32 - 1); // byte access
const OOB_BITS_16: i32 = !(MEMORY_SIZE as i32 - 1 - 1); // halfword access
const OOB_BITS_32: i32 = !(MEMORY_SIZE as i32 - 1 - 3); // word access
const OOB_BITS_64: i32 = !(MEMORY_SIZE as i32 - 1 - 7); // double word access
const OOB_BITS_PC: u32 = !((MEMORY_SIZE / 4) as u32 - 1); // program counter (word index)

struct CPU {
	registers: [i32; 32],
	memory32: Vec<u32>,
	program_counter: u32,
	program_ended: bool,
	error_message: Option<&'static str>,
	reservation_address: i32,
	fp_registers: [u64; 32], // 64-bit storage for F and D extensions
}

impl CPU {
	fn new() -> Self {
		CPU {
			registers: [0; 32],
			memory32: vec![0; MEMORY_SIZE / 4],
			program_counter: 0,
			program_ended: false,
			error_message: None,
			reservation_address: -1,
			fp_registers: [0; 32],
		}
	}

	#[inline(always)]
	fn memory8(&self) -> &[u8] {
		unsafe {
			std::slice::from_raw_parts(
				self.memory32.as_ptr() as *const u8,
				MEMORY_SIZE,
			)
		}
	}

	#[inline(always)]
	fn memory8_mut(&mut self) -> &mut [u8] {
		unsafe {
			std::slice::from_raw_parts_mut(
				self.memory32.as_mut_ptr() as *mut u8,
				MEMORY_SIZE,
			)
		}
	}

	#[inline(always)]
	fn memory16(&self) -> &[u16] {
		unsafe {
			std::slice::from_raw_parts(
				self.memory32.as_ptr() as *const u16,
				MEMORY_SIZE / 2,
			)
		}
	}

	#[inline(always)]
	fn memory16_mut(&mut self) -> &mut [u16] {
		unsafe {
			std::slice::from_raw_parts_mut(
				self.memory32.as_mut_ptr() as *mut u16,
				MEMORY_SIZE / 2,
			)
		}
	}

	#[inline(always)]
	fn memory32_signed(&self) -> &[i32] {
		unsafe {
			std::slice::from_raw_parts(
				self.memory32.as_ptr() as *const i32,
				MEMORY_SIZE / 4,
			)
		}
	}

	#[inline(always)]
	fn memory32_signed_mut(&mut self) -> &mut [i32] {
		unsafe {
			std::slice::from_raw_parts_mut(
				self.memory32.as_mut_ptr() as *mut i32,
				MEMORY_SIZE / 4,
			)
		}
	}

	#[inline(always)]
	fn register_unsigned(&self, index: usize) -> u32 {
		self.registers[index] as u32
	}

	#[inline(always)]
	fn fp_f32(&self, index: usize) -> f32 {
		f32::from_bits((self.fp_registers[index] & 0xffffffff) as u32)
	}

	#[inline(always)]
	fn fp_f32_set(&mut self, index: usize, value: f32) {
		self.fp_registers[index] = (value.to_bits() as u64) | 0xffffffff00000000; // NaN-box
	}

	#[inline(always)]
	fn fp_f64(&self, index: usize) -> f64 {
		f64::from_bits(self.fp_registers[index])
	}

	#[inline(always)]
	fn fp_f64_set(&mut self, index: usize, value: f64) {
		self.fp_registers[index] = value.to_bits();
	}

	#[inline(always)]
	fn fp_u32(&self, index: usize) -> u32 {
		(self.fp_registers[index] & 0xffffffff) as u32
	}

	#[inline(always)]
	fn fp_u32_set(&mut self, index: usize, value: u32) {
		self.fp_registers[index] = (value as u64) | 0xffffffff00000000; // NaN-box
	}

	#[inline(always)]
	fn fp_i32(&self, index: usize) -> i32 {
		(self.fp_registers[index] & 0xffffffff) as i32
	}

	#[inline(always)]
	fn fp_i32_set(&mut self, index: usize, value: i32) {
		self.fp_registers[index] = (value as u32 as u64) | 0xffffffff00000000; // NaN-box
	}

	fn tick(&mut self) {
		// make it constant
		self.registers[0] = 0;

		let instruction = self.memory32_signed()[self.program_counter as usize] as u32;

		let funct3 = (instruction >> 12) & 0b111;

		let register_destination = ((instruction >> 7) & 0b11111) as usize;
		let register_source1 = ((instruction >> 15) & 0b11111) as usize;
		let register_source2 = ((instruction >> 20) & 0b11111) as usize;

		let opcode_combined = ((((instruction >> 2) << 3) & 0xff) | funct3) as u8;

		match opcode_combined {
		// load
		0b00000000 => { // lb
			let addr = self.registers[register_source1]
				.wrapping_add((instruction as i32) >> 20);
			if addr & OOB_BITS_8 != 0 {
				self.error_message = Some("out of bounds");
				return;
			}
			self.registers[register_destination] = self.memory8()[addr as usize] as i8 as i32;
		}
		0b00000001 => { // lh
			let addr = self.registers[register_source1]
				.wrapping_add((instruction as i32) >> 20);
			if addr & OOB_BITS_16 != 0 {
				self.error_message = Some("out of bounds");
				return;
			}
			self.registers[register_destination] =
				self.memory16()[(addr >> 1) as usize] as i16 as i32;
		}
		0b00000010 => { // lw
			let addr = self.registers[register_source1]
				.wrapping_add((instruction as i32) >> 20);
			if addr & OOB_BITS_32 != 0 {
				self.error_message = Some("out of bounds");
				return;
			}
			self.registers[register_destination] = self.memory32_signed()[(addr >> 2) as usize];
		}
		0b00000100 => { // lbu
			let addr = self.registers[register_source1]
				.wrapping_add((instruction as i32) >> 20);
			if addr & OOB_BITS_8 != 0 {
				self.error_message = Some("out of bounds");
				return;
			}
			self.registers[register_destination] = self.memory8()[addr as usize] as i32;
		}
		0b00000101 => { // lhu
			let addr = self.registers[register_source1]
				.wrapping_add((instruction as i32) >> 20);
			if addr & OOB_BITS_16 != 0 {
				self.error_message = Some("out of bounds");
				return;
			}
			self.registers[register_destination] = self.memory16()[(addr >> 1) as usize] as i32;
		}
		// floating-point load
		0b00001010 => { // flw
			let addr = self.registers[register_source1]
				.wrapping_add((instruction as i32) >> 20);
			if addr & OOB_BITS_32 != 0 {
				self.error_message = Some("out of bounds");
				return;
			}
			let value = self.memory32[(addr >> 2) as usize];
			self.fp_u32_set(register_destination, value);
		}
		0b00001011 => { // fld
			let addr = self.registers[register_source1]
				.wrapping_add((instruction as i32) >> 20);
			if addr & OOB_BITS_64 != 0 {
				self.error_message = Some("out of bounds");
				return;
			}
			let word_index = (addr >> 2) as usize;
			let lo = self.memory32[word_index] as u64;
			let hi = self.memory32[word_index + 1] as u64;
			self.fp_registers[register_destination] = lo | (hi << 32);
		}
		// fence
		// register+immediate
		0b00100000 => { // addi
			self.registers[register_destination] = self.registers[register_source1]
				.wrapping_add((instruction as i32) >> 20);
		}
		0b00100001 => { // slli
			self.registers[register_destination] =
				self.registers[register_source1] << ((instruction >> 20) & 0b11111);
		}
		0b00100010 => { // slti
			self.registers[register_destination] =
				if self.registers[register_source1] < ((instruction as i32) >> 20) {
					1
				} else {
					0
				};
		}
		0b00100011 => { // sltiu
			self.registers[register_destination] =
				if self.register_unsigned(register_source1) < (instruction >> 20) {
					1
				} else {
					0
				};
		}
		0b00100100 => { // xori
			self.registers[register_destination] =
				self.registers[register_source1] ^ ((instruction as i32) >> 20);
		}
		0b00100101 => { // srli/srai
			let shift_by = (instruction >> 20) & 0b11111;
			self.registers[register_destination] =
				if instruction >> 30 != 0 {
					self.registers[register_source1] >> shift_by
				} else {
					(self.register_unsigned(register_source1) >> shift_by) as i32
				};
		}
		0b00100110 => { // ori
			self.registers[register_destination] =
				self.registers[register_source1] | ((instruction as i32) >> 20);
		}
		0b00100111 => { // andi
			self.registers[register_destination] =
				self.registers[register_source1] & ((instruction as i32) >> 20);
		}
		0b00101000 |
		0b00101001 |
		0b00101010 |
		0b00101011 |
		0b00101100 |
		0b00101101 |
		0b00101110 |
		0b00101111 => { // auipc
			self.registers[register_destination] = ((self.program_counter << 2) as i32)
				.wrapping_add((instruction & 0xfffff000) as i32);
		}
		// store
		0b01000000 => { // sb
			let addr = self.registers[register_source1].wrapping_add(
				(((instruction as i32) >> 25) << 5) | (register_destination as i32),
			);
			if addr & OOB_BITS_8 != 0 {
				self.error_message = Some("out of bounds");
				return;
			}
			self.memory8_mut()[addr as usize] = self.registers[register_source2] as u8;
		}
		0b01000001 => { // sh
			let addr = self.registers[register_source1].wrapping_add(
				(((instruction as i32) >> 25) << 5) | (register_destination as i32),
			);
			if addr & OOB_BITS_16 != 0 {
				self.error_message = Some("out of bounds");
				return;
			}
			self.memory16_mut()[(addr >> 1) as usize] = self.registers[register_source2] as u16;
		}
		0b01000010 => { // sw
			let addr = self.registers[register_source1].wrapping_add(
				(((instruction as i32) >> 25) << 5) | (register_destination as i32),
			);
			if addr & OOB_BITS_32 != 0 {
				self.error_message = Some("out of bounds");
				return;
			}
			self.memory32_signed_mut()[(addr >> 2) as usize] = self.registers[register_source2];
		}
		// floating-point store
		0b01001010 => { // fsw
			let addr = self.registers[register_source1].wrapping_add(
				(((instruction as i32) >> 25) << 5) | (register_destination as i32),
			);
			if addr & OOB_BITS_32 != 0 {
				self.error_message = Some("out of bounds");
				return;
			}
			self.memory32[(addr >> 2) as usize] = self.fp_u32(register_source2);
		}
		0b01001011 => { // fsd
			let addr = self.registers[register_source1].wrapping_add(
				(((instruction as i32) >> 25) << 5) | (register_destination as i32),
			);
			if addr & OOB_BITS_64 != 0 {
				self.error_message = Some("out of bounds");
				return;
			}
			let word_index = (addr >> 2) as usize;
			let value = self.fp_registers[register_source2];
			self.memory32[word_index] = (value & 0xffffffff) as u32;
			self.memory32[word_index + 1] = (value >> 32) as u32;
		}
		// atomic
		0b01011010 => 'atomic: {
			let addr = self.registers[register_source1];
			if addr & OOB_BITS_32 != 0 {
				self.error_message = Some("out of bounds");
				return;
			}
			let addr_word = (addr >> 2) as usize;
			let funct5 = instruction >> 27;
			
			if funct5 == 0b00011 { // sc.w?
				self.registers[register_destination] =
					if self.reservation_address == addr { // success?
						self.memory32_signed_mut()[addr_word] = self.registers[register_source2];
						0
					} else {
						1
					};
				self.reservation_address = -1;
				break 'atomic;
			}
			let value_before = self.memory32_signed()[addr_word];
			self.registers[register_destination] = value_before;

			self.memory32_signed_mut()[addr_word] =
				match funct5 {
				0b00000 => { // amoadd.w
					value_before.wrapping_add(self.registers[register_source2])
				}
				0b00001 => { // amoswap.w
					self.registers[register_source2]
				}
				0b00010 => { // lr.w
					self.reservation_address = addr;
					break 'atomic;
				}
				// 0b00011: handled above
				0b00100 => { // amoxor.w
					value_before ^ self.registers[register_source2]
				}
				0b01000 => { // amoor.w
					value_before | self.registers[register_source2]
				}
				0b01100 => { // amoand.w
					value_before & self.registers[register_source2]
				}
				0b10000 => { // amomin.w
					if value_before < self.registers[register_source2] {
						value_before
					} else {
						self.registers[register_source2]
					}
				}
				0b10100 => { // amomax.w
					if value_before > self.registers[register_source2] {
						value_before
					} else {
						self.registers[register_source2]
					}
				}
				0b11000 => { // amominu.w
					if (value_before as u32) < self.register_unsigned(register_source2) {
						value_before
					} else {
						self.registers[register_source2]
					}
				}
				0b11100 => { // amomaxu.w
					if (value_before as u32) > self.register_unsigned(register_source2) {
						value_before
					} else {
						self.registers[register_source2]
					}
				}
				_ => {
					self.error_message = Some("illegal atomic operation");
					return;
				}
				};
		}
		// register+register
		0b01100000 => { // add/sub/mul
			self.registers[register_destination] =
				if instruction & (1 << 25) != 0 { // mul?
					self.registers[register_source1].wrapping_mul(self.registers[register_source2])
				} else if instruction >> 30 != 0 { // sub?
					self.registers[register_source1].wrapping_sub(self.registers[register_source2])
				} else {
					self.registers[register_source1].wrapping_add(self.registers[register_source2])
				};
		}
		0b01100001 => { // sll/mulh
			self.registers[register_destination] =
				if instruction & (1 << 25) != 0 { // mulh?
					((self.registers[register_source1] as i64)
						.wrapping_mul(self.registers[register_source2] as i64) >> 32) as i32
				} else {
					self.registers[register_source1] << (self.registers[register_source2] & 0b11111)
				};
		}
		0b01100010 => { // slt/mulhsu
			self.registers[register_destination] =
				if instruction & (1 << 25) != 0 { // mulhsu?
					((self.registers[register_source1] as i64)
						.wrapping_mul(self.register_unsigned(register_source2) as u64 as i64) >> 32) as i32
				} else if self.registers[register_source1] < self.registers[register_source2] {
					1
				} else {
					0
				};
		}
		0b01100011 => { // sltu/mulhu
			self.registers[register_destination] =
				if instruction & (1 << 25) != 0 { // mulhu?
					((self.register_unsigned(register_source1) as u64)
						.wrapping_mul(self.register_unsigned(register_source2) as u64) >> 32) as i32
				} else if self.register_unsigned(register_source1) < self.register_unsigned(register_source2) {
					1
				} else {
					0
				};
		}
		0b01100100 => { // xor/div
			self.registers[register_destination] =
				if instruction & (1 << 25) != 0 { // div?
					let dividend = self.registers[register_source1];
					let divisor = self.registers[register_source2];
					if divisor == 0 {
						-1
					} else if dividend == i32::MIN && divisor == -1 {
						i32::MIN
					} else {
						dividend.wrapping_div(divisor)
					}
				} else {
					self.registers[register_source1] ^ self.registers[register_source2]
				}
		}
		0b01100101 => { // srl/sra/divu
			self.registers[register_destination] =
				if instruction & (1 << 25) != 0 { // divu?
					let divisor = self.register_unsigned(register_source2);
					if divisor == 0 {
						-1
					} else {
						self.register_unsigned(register_source1).wrapping_div(divisor) as i32
					}
				} else {
					let shift_by = self.registers[register_source2] & 0b11111;
					if instruction >> 30 != 0 {
						self.registers[register_source1] >> shift_by
					} else {
						(self.register_unsigned(register_source1) >> shift_by) as i32
					}
				}
		}
		0b01100110 => { // or/rem
			self.registers[register_destination] =
				if instruction & (1 << 25) != 0 { // rem?
					let dividend = self.registers[register_source1];
					let divisor = self.registers[register_source2];
					if divisor == 0 {
						dividend
					} else if dividend == i32::MIN && divisor == -1 { // overflow?
						0
					} else {
						dividend.wrapping_rem(divisor)
					}
				} else {
					self.registers[register_source1] | self.registers[register_source2]
				}
		}
		0b01100111 => { // and/remu
			self.registers[register_destination] = 
				if instruction & (1 << 25) != 0 { // remu?
					let dividend = self.register_unsigned(register_source1);
					let divisor = self.register_unsigned(register_source2);
					if divisor == 0 {
						dividend as i32
					} else {
						dividend.wrapping_rem(divisor) as i32
					}
				} else {
					self.registers[register_source1] & self.registers[register_source2]
				}
		}
		0b01101000 |
		0b01101001 |
		0b01101010 |
		0b01101011 |
		0b01101100 |
		0b01101101 |
		0b01101110 |
		0b01101111 => { // lui ;)
			self.registers[register_destination] = (instruction & 0xfffff000) as i32;
		}
		// fused multiply-add (F and D extensions)
		0b10000010 | 0b10000011 => { // fmadd.s/fmadd.d
			let register_source3 = (instruction >> 27) as usize;
			let is_double = (funct3 & 1) != 0;
			if is_double {
				let result = self.fp_f64(register_source1) * self.fp_f64(register_source2) + self.fp_f64(register_source3);
				self.fp_f64_set(register_destination, result);
			}
			else {
				let result = self.fp_f32(register_source1) * self.fp_f32(register_source2) + self.fp_f32(register_source3);
				self.fp_f32_set(register_destination, result);
			}
		}
		0b10001010 | 0b10001011 => { // fmsub.s/fmsub.d
			let register_source3 = (instruction >> 27) as usize;
			let is_double = (funct3 & 1) != 0;
			if is_double {
				let result = self.fp_f64(register_source1) * self.fp_f64(register_source2) - self.fp_f64(register_source3);
				self.fp_f64_set(register_destination, result);
			}
			else {
				let result = self.fp_f32(register_source1) * self.fp_f32(register_source2) - self.fp_f32(register_source3);
				self.fp_f32_set(register_destination, result);
			}
		}
		0b10010010 | 0b10010011 => { // fnmsub.s/fnmsub.d
			let register_source3 = (instruction >> 27) as usize;
			let is_double = (funct3 & 1) != 0;
			if is_double {
				let result = -(self.fp_f64(register_source1) * self.fp_f64(register_source2)) + self.fp_f64(register_source3);
				self.fp_f64_set(register_destination, result);
			}
			else {
				let result = -(self.fp_f32(register_source1) * self.fp_f32(register_source2)) + self.fp_f32(register_source3);
				self.fp_f32_set(register_destination, result);
			}
		}
		0b10011010 | 0b10011011 => { // fnmadd.s/fnmadd.d
			let register_source3 = (instruction >> 27) as usize;
			let is_double = (funct3 & 1) != 0;
			if is_double {
				let result = -(self.fp_f64(register_source1) * self.fp_f64(register_source2) + self.fp_f64(register_source3));
				self.fp_f64_set(register_destination, result);
			}
			else {
				let result = -(self.fp_f32(register_source1) * self.fp_f32(register_source2) + self.fp_f32(register_source3));
				self.fp_f32_set(register_destination, result);
			}
		}
		// floating-point operations
		0b10100000 | 0b10100001 | 0b10100010 | 0b10100011 |
		0b10100100 | 0b10100101 | 0b10100110 | 0b10100111 => {
			let funct7 = instruction >> 25;
			let funct5 = funct7 >> 2;
			let is_double = (funct7 & 1) != 0;
			
			match funct5 {
			0b00000 => { // fadd
				if is_double {
					self.fp_f64_set(register_destination, self.fp_f64(register_source1) + self.fp_f64(register_source2));
				}
				else {
					self.fp_f32_set(register_destination, self.fp_f32(register_source1) + self.fp_f32(register_source2));
				}
			}
			0b00001 => { // fsub
				if is_double {
					self.fp_f64_set(register_destination, self.fp_f64(register_source1) - self.fp_f64(register_source2));
				}
				else {
					self.fp_f32_set(register_destination, self.fp_f32(register_source1) - self.fp_f32(register_source2));
				}
			}
			0b00010 => { // fmul
				if is_double {
					self.fp_f64_set(register_destination, self.fp_f64(register_source1) * self.fp_f64(register_source2));
				}
				else {
					self.fp_f32_set(register_destination, self.fp_f32(register_source1) * self.fp_f32(register_source2));
				}
			}
			0b00011 => { // fdiv
				if is_double {
					self.fp_f64_set(register_destination, self.fp_f64(register_source1) / self.fp_f64(register_source2));
				}
				else {
					self.fp_f32_set(register_destination, self.fp_f32(register_source1) / self.fp_f32(register_source2));
				}
			}
			0b01011 => { // fsqrt
				if is_double {
					self.fp_f64_set(register_destination, self.fp_f64(register_source1).sqrt());
				}
				else {
					self.fp_f32_set(register_destination, self.fp_f32(register_source1).sqrt());
				}
			}
			0b00100 => { // fsgnj/fsgnjn/fsgnjx
				if is_double {
					let val1 = self.fp_registers[register_source1];
					let val2 = self.fp_registers[register_source2];
					let sign1 = (val1 >> 63) as u32;
					let sign2 = (val2 >> 63) as u32;
					let result = match funct3 {
						0b000 => (val1 & 0x7fffffffffffffff) | ((sign2 as u64) << 63), // fsgnj
						0b001 => (val1 & 0x7fffffffffffffff) | (((sign2 ^ 1) as u64) << 63), // fsgnjn
						_ => (val1 & 0x7fffffffffffffff) | (((sign1 ^ sign2) as u64) << 63), // fsgnjx
					};
					self.fp_registers[register_destination] = result;
				}
				else {
					let val1 = self.fp_u32(register_source1);
					let val2 = self.fp_u32(register_source2);
					let sign1 = val1 >> 31;
					let sign2 = val2 >> 31;
					let result = match funct3 {
						0b000 => (val1 & 0x7fffffff) | (sign2 << 31), // fsgnj
						0b001 => (val1 & 0x7fffffff) | ((sign2 ^ 1) << 31), // fsgnjn
						_ => (val1 & 0x7fffffff) | ((sign1 ^ sign2) << 31), // fsgnjx
					};
					self.fp_u32_set(register_destination, result);
				}
			}
			0b00101 => { // fmin/fmax
				if is_double {
					let val1 = self.fp_f64(register_source1);
					let val2 = self.fp_f64(register_source2);
					let result = if funct3 == 0b000 { val1.min(val2) } else { val1.max(val2) };
					self.fp_f64_set(register_destination, result);
				}
				else {
					let val1 = self.fp_f32(register_source1);
					let val2 = self.fp_f32(register_source2);
					let result = if funct3 == 0b000 { val1.min(val2) } else { val1.max(val2) };
					self.fp_f32_set(register_destination, result);
				}
			}
			0b01000 => { // fcvt.s.d/fcvt.d.s
				if is_double { // fcvt.d.s
					self.fp_f64_set(register_destination, self.fp_f32(register_source1) as f64);
				}
				else { // fcvt.s.d
					self.fp_f32_set(register_destination, self.fp_f64(register_source1) as f32);
				}
			}
			0b10100 => { // fcmp (feq/flt/fle)
				if is_double {
					let val1 = self.fp_f64(register_source1);
					let val2 = self.fp_f64(register_source2);
					self.registers[register_destination] = match funct3 {
						0b010 => (val1 == val2) as i32, // feq
						0b001 => (val1 < val2) as i32, // flt
						_ => (val1 <= val2) as i32, // fle
					};
				}
				else {
					let val1 = self.fp_f32(register_source1);
					let val2 = self.fp_f32(register_source2);
					self.registers[register_destination] = match funct3 {
						0b010 => (val1 == val2) as i32, // feq
						0b001 => (val1 < val2) as i32, // flt
						_ => (val1 <= val2) as i32, // fle
					};
				}
			}
			0b11000 => { // fcvt.w.s/fcvt.w.d/fcvt.wu.s/fcvt.wu.d
				if is_double {
					let val = self.fp_f64(register_source1);
					if register_source2 == 0b00000 { // fcvt.w.d
						self.registers[register_destination] = val as i32;
					}
					else if register_source2 == 0b00001 { // fcvt.wu.d
						self.registers[register_destination] = (val as u32) as i32;
					}
				}
				else {
					let val = self.fp_f32(register_source1);
					if register_source2 == 0b00000 { // fcvt.w.s
						self.registers[register_destination] = val as i32;
					}
					else if register_source2 == 0b00001 { // fcvt.wu.s
						self.registers[register_destination] = (val as u32) as i32;
					}
				}
			}
			0b11010 => { // fcvt.s.w/fcvt.d.w/fcvt.s.wu/fcvt.d.wu
				if is_double {
					if register_source2 == 0b00000 { // fcvt.d.w
						self.fp_f64_set(register_destination, self.registers[register_source1] as f64);
					}
					else if register_source2 == 0b00001 { // fcvt.d.wu
						self.fp_f64_set(register_destination, self.register_unsigned(register_source1) as f64);
					}
				}
				else {
					if register_source2 == 0b00000 { // fcvt.s.w
						self.fp_f32_set(register_destination, self.registers[register_source1] as f32);
					}
					else if register_source2 == 0b00001 { // fcvt.s.wu
						self.fp_f32_set(register_destination, self.register_unsigned(register_source1) as f32);
					}
				}
			}
			0b11100 => { // fmv.x.w/fmv.x.d/fclass
				if funct3 == 0b000 { // fmv.x.w/fmv.x.d
					if is_double { // fmv.x.d (RV64 only, not implemented)
						self.error_message = Some("illegal instruction");
						return;
					}
					else { // fmv.x.w
						self.registers[register_destination] = self.fp_i32(register_source1);
					}
				}
				else if funct3 == 0b001 { // fclass
					if is_double {
						let bits = self.fp_registers[register_source1];
						let sign = (bits >> 63) as u32;
						let exp = ((bits >> 52) & 0x7ff) as u32;
						let mantissa = bits & 0xfffffffffffff;
						
						self.registers[register_destination] = if exp == 0 && mantissa == 0 {
							if sign != 0 { 0x008 } else { 0x001 } // -0 or +0
						} else if exp == 0 {
							if sign != 0 { 0x010 } else { 0x002 } // -subnormal or +subnormal
						} else if exp == 0x7ff && mantissa == 0 {
							if sign != 0 { 0x080 } else { 0x004 } // -inf or +inf
						} else if exp == 0x7ff {
							0x200 // qNaN or sNaN
						} else {
							if sign != 0 { 0x040 } else { 0x020 } // -normal or +normal
						};
					}
					else {
						let bits = self.fp_u32(register_source1);
						let sign = bits >> 31;
						let exp = (bits >> 23) & 0xff;
						let mantissa = bits & 0x7fffff;
						
						self.registers[register_destination] = if exp == 0 && mantissa == 0 {
							if sign != 0 { 0x008 } else { 0x001 } // -0 or +0
						} else if exp == 0 {
							if sign != 0 { 0x010 } else { 0x002 } // -subnormal or +subnormal
						} else if exp == 0xff && mantissa == 0 {
							if sign != 0 { 0x080 } else { 0x004 } // -inf or +inf
						} else if exp == 0xff {
							0x200 // qNaN or sNaN
						} else {
							if sign != 0 { 0x040 } else { 0x020 } // -normal or +normal
						};
					}
				}
			}
			0b11110 => { // fmv.w.x
				if is_double { // fmv.d.x (RV64 only, not implemented)
					self.error_message = Some("illegal instruction");
					return;
				}
				else { // fmv.w.x
					self.fp_i32_set(register_destination, self.registers[register_source1]);
				}
			}
			_ => {
				self.error_message = Some("illegal instruction");
				return;
			}
			}
		}
		0b11000000 |
		0b11000001 |
		0b11000100 |
		0b11000101 |
		0b11000110 |
		0b11000111 => { // branch
			let should_branch =
				match funct3 {
				0b000 => { // beq
					self.registers[register_source1] == self.registers[register_source2]
				}
				0b001 => { // bne
					self.registers[register_source1] != self.registers[register_source2]
				}
				0b100 => { // blt
					self.registers[register_source1] < self.registers[register_source2]
				}
				0b101 => { // bge
					self.registers[register_source1] >= self.registers[register_source2]
				}
				0b110 => { // bltu
					self.register_unsigned(register_source1) < self.register_unsigned(register_source2)
				}
				0b111 => { // bgeu
					self.register_unsigned(register_source1) >= self.register_unsigned(register_source2)
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
			self.program_counter = self.program_counter.wrapping_add( // 12 bit offset, shifted one to the right
				(((instruction as i32) >> 31) << 10 | // 31 -> 10
				((register_destination as i32) & 0x1) << 9 | // dest -> 9
				((instruction >> 25) << 3) as i32 | // 30-25 -> 8-3
				(register_destination >> 2) as i32) as u32, // dest -> 2-0
			);
			if self.program_counter & OOB_BITS_PC != 0 {
				self.error_message = Some("out of bounds");
			}
			return;
		}
		0b11001000 => { // jalr
			self.registers[register_destination] = ((self.program_counter + 1) << 2) as i32;
			self.program_counter = (
				self.registers[register_source1]
					.wrapping_add((instruction as i32) >> 20) >> 2
			) as u32;
			if self.program_counter & OOB_BITS_PC != 0 {
				self.error_message = Some("out of bounds");
			}
			return;
		}
		0b11011000 |
		0b11011001 |
		0b11011010 |
		0b11011011 |
		0b11011100 |
		0b11011101 |
		0b11011110 |
		0b11011111 => { // jal
			// exit on endless loop
			if instruction >> 12 == 0 {
				self.program_ended = true;
				return;
			}
			self.registers[register_destination] = ((self.program_counter + 1) << 2) as i32;
			self.program_counter = self.program_counter.wrapping_add(
				(((instruction as i32) >> 31) << 18 |
				(((instruction >> 12) & 0xff) << 10) as i32 |
				(((instruction >> 20) & 0x1) << 9) as i32 |
				((instruction >> 22) & 0x3ff) as i32) as u32,
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
	let program_path =
		if args.len() > 1 {
			&args[1]
		} else {
			"../tests/count.bin"
		};

	println!("loading {}", program_path);

	let mut file = File::open(program_path).expect("Failed to open file");
	let mut cpu = CPU::new();
	file.read(cpu.memory8_mut())
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

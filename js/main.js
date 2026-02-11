'use strict';

const MEMORY_SIZE = 64 * 1024;

// Out of bounds bit masks for memory access validation
// Any address with these bits set is out of bounds
const OOB_BITS_8 = ~(MEMORY_SIZE - 1); // byte access
const OOB_BITS_16 = ~(MEMORY_SIZE - 1 - 1); // halfword access
const OOB_BITS_32 = ~(MEMORY_SIZE - 1 - 3); // word access
const OOB_BITS_PC = ~(MEMORY_SIZE / 4 - 1); // program counter (word index)
const OOB_BITS_64 = ~(MEMORY_SIZE - 1 - 7); // double word access

const registers_unsigned = new Uint32Array(32);
const registers = new Int32Array(registers_unsigned.buffer);
const memory32_unsigned = new Uint32Array(MEMORY_SIZE / 4);
const memory32 = new Int32Array(memory32_unsigned.buffer);
const memory16 = new Uint16Array(memory32_unsigned.buffer);
const memory8 = new Uint8Array(memory32_unsigned.buffer);

// floating-point registers (64-bit for D extension compatibility)
const fp_registers_buffer = new ArrayBuffer(32 * 8);
const fp_registers_f32 = new Float32Array(fp_registers_buffer);
const fp_registers_f64 = new Float64Array(fp_registers_buffer);
const fp_registers_u32 = new Uint32Array(fp_registers_buffer);
const fp_registers_i32 = new Int32Array(fp_registers_buffer);

// index for 32 bit!
let program_counter = 0;

// reservation tracking for LR/SC
let reservation_address = -1;

function tick() {
	// make it constant
	registers[0] = 0;

	const instruction = memory32[program_counter];

	const funct3 = (instruction >>> 12) & 0b111;

	const register_destination = (instruction >>> 7) & 0b11111;
	const register_source1 = (instruction >>> 15) & 0b11111;
	const register_source2 = (instruction >>> 20) & 0b11111;

	// Note: JavaScript's >> operator is arithmetic right shift (sign-preserving).
	// Using (instruction >> 20) correctly sign-extends 12-bit immediates.
	// The | 0 suffix converts to 32-bit signed integer.
	opcode: switch ((instruction >>> 2 << 3) & 0xff | funct3) {
	// load
	case 0b00000000: { // lb
		const addr = registers[register_source1] + (instruction >> 20) | 0;
		if (addr & OOB_BITS_8) throw 'out of bounds';
		registers[register_destination] = memory8[addr] << 24 >> 24;
		break;
	}
	case 0b00000001: { // lh
		const addr = registers[register_source1] + (instruction >> 20) | 0;
		if (addr & OOB_BITS_16) throw 'out of bounds';
		registers[register_destination] = memory16[addr >>> 1] << 16 >> 16;
		break;
	}
	case 0b00000010: { // lw
		const addr = registers[register_source1] + (instruction >> 20) | 0;
		if (addr & OOB_BITS_32) throw 'out of bounds';
		registers[register_destination] = memory32[addr >>> 2];
		break;
	}
	case 0b00000100: { // lbu
		const addr = registers[register_source1] + (instruction >> 20) | 0;
		if (addr & OOB_BITS_8) throw 'out of bounds';
		registers[register_destination] = memory8[addr];
		break;
	}
	case 0b00000101: { // lhu
		const addr = registers[register_source1] + (instruction >> 20) | 0;
		if (addr & OOB_BITS_16) throw 'out of bounds';
		registers[register_destination] = memory16[addr >>> 1];
		break;
	}
	// floating-point load
	case 0b00001010: { // flw
		const addr = registers[register_source1] + (instruction >> 20) | 0;
		if (addr & OOB_BITS_32) throw 'out of bounds';
		fp_registers_u32[register_destination * 2] = memory32_unsigned[addr >>> 2];
		fp_registers_u32[register_destination * 2 + 1] = 0xffffffff; // NaN-box
		break;
	}
	case 0b00001011: { // fld
		const addr = registers[register_source1] + (instruction >> 20) | 0;
		if (addr & OOB_BITS_64) throw 'out of bounds';
		const word_index = addr >>> 2;
		fp_registers_u32[register_destination * 2] = memory32_unsigned[word_index];
		fp_registers_u32[register_destination * 2 + 1] = memory32_unsigned[word_index + 1];
		break;
	}
	// fence
	// register+immediate
	case 0b00100000: // addi
		registers[register_destination] = registers[register_source1] + (instruction >> 20);
		break;
	case 0b00100001: // slli
		registers[register_destination] = registers[register_source1] << ((instruction >>> 20) & 0b11111);
		break;
	case 0b00100010: // slti
		registers[register_destination] = registers[register_source1] < instruction >> 20 ? 1 : 0;
		break;
	case 0b00100011: // sltiu
		registers[register_destination] = registers_unsigned[register_source1] < instruction >>> 20 ? 1 : 0;
		break;
	case 0b00100100: // xori
		registers[register_destination] = registers[register_source1] ^ instruction >> 20;
		break;
	case 0b00100101: { // srli/srai
		const shift_by = (instruction >>> 20) & 0b11111;
		if (instruction >>> 30) {
			registers[register_destination] = registers[register_source1] >> shift_by;
		}
		else {
			registers_unsigned[register_destination] = registers_unsigned[register_source1] >>> shift_by;
		}
		break;
	}
	case 0b00100110: // ori
		registers[register_destination] = registers[register_source1] | instruction >> 20;
		break;
	case 0b00100111: // andi
		registers[register_destination] = registers[register_source1] & instruction >> 20;
		break;
	case 0b00101000: // auipc
	case 0b00101001:
	case 0b00101010:
	case 0b00101011:
	case 0b00101100:
	case 0b00101101:
	case 0b00101110:
	case 0b00101111:
		registers[register_destination] = (program_counter << 2) + (instruction & 0xfffff000);
		break;
	// store
	case 0b01000000: { // sb
		const addr = registers[register_source1] + (instruction >> 25 << 5 | register_destination) | 0;
		if (addr & OOB_BITS_8) throw 'out of bounds';
		memory8[addr] = registers[register_source2];
		break;
	}
	case 0b01000001: { // sh
		const addr = registers[register_source1] + (instruction >> 25 << 5 | register_destination) | 0;
		if (addr & OOB_BITS_16) throw 'out of bounds';
		memory16[addr >>> 1] = registers[register_source2];
		break;
	}
	case 0b01000010: { // sw
		const addr = registers[register_source1] + (instruction >> 25 << 5 | register_destination) | 0;
		if (addr & OOB_BITS_32) throw 'out of bounds';
		memory32[addr >>> 2] = registers[register_source2];
		break;
	}
	// floating-point store
	case 0b01001010: { // fsw
		const addr = registers[register_source1] + (instruction >> 25 << 5 | register_destination) | 0;
		if (addr & OOB_BITS_32) throw 'out of bounds';
		memory32_unsigned[addr >>> 2] = fp_registers_u32[register_source2 * 2];
		break;
	}
	case 0b01001011: { // fsd
		const addr = registers[register_source1] + (instruction >> 25 << 5 | register_destination) | 0;
		if (addr & OOB_BITS_64) throw 'out of bounds';
		const word_index = addr >>> 2;
		memory32_unsigned[word_index] = fp_registers_u32[register_source2 * 2];
		memory32_unsigned[word_index + 1] = fp_registers_u32[register_source2 * 2 + 1];
		break;
	}
	// atomic
	case 0b01011010: {
		const addr = registers[register_source1];
		if (addr & OOB_BITS_32) throw 'out of bounds';
		const addr_word = addr >>> 2;
		const funct5 = instruction >>> 27;

		if (funct5 === 0b00011) { // sc.w?
			registers[register_destination] = (
				reservation_address === addr // success?
				?	(
					memory32[addr_word] = registers[register_source2],
					0
				)
				:	1
			);
			reservation_address = -1;
			break;
		}
		const value_before = registers[register_destination] = memory32[addr_word];

		switch (funct5) {
		case 0b00000: // amoadd.w
			memory32[addr_word] = value_before + registers[register_source2];
			break;
		// case 0b00011: handled above
		case 0b00001: // amoswap.w
			memory32[addr_word] = registers[register_source2];
			break;
		case 0b00010: // lr.w
			reservation_address = addr;
			break;
		case 0b00100: // amoxor.w
			memory32[addr_word] = value_before ^ registers[register_source2];
			break;
		case 0b01000: // amoor.w
			memory32[addr_word] = value_before | registers[register_source2];
			break;
		case 0b01100: // amoand.w
			memory32[addr_word] = value_before & registers[register_source2];
			break;
		case 0b10000: // amomin.w
			memory32[addr_word] = (
				value_before < registers[register_source2]
				?	value_before
				:	registers[register_source2]
			);
			break;
		case 0b10100: // amomax.w
			memory32[addr_word] = (
				value_before > registers[register_source2]
				?	value_before
				:	registers[register_source2]
			);
			break;
		case 0b11000: // amominu.w
			memory32[addr_word] = (
				memory32_unsigned[addr_word] < registers_unsigned[register_source2]
				?	value_before
				:	registers[register_source2]
			);
			break;
		case 0b11100: // amomaxu.w
			memory32[addr_word] = (
				memory32_unsigned[addr_word] > registers_unsigned[register_source2]
				?	value_before
				:	registers[register_source2]
			);
			break;
		default:
			throw 'illegal atomic operation';
		}
		break;
	}
	// register+register
	case 0b01100000: // add/sub/mul
		registers[register_destination] = (
			instruction & (1 << 25) // mul?
			?	registers[register_source1] * registers[register_source2]
			: instruction >>> 30 // sub?
			?	registers[register_source1] - registers[register_source2]
			:	registers[register_source1] + registers[register_source2]
		);
		break;
	case 0b01100001: // sll/mulh
		registers[register_destination] = (
			instruction & (1 << 25) // mulh?
			?	Number((BigInt(registers[register_source1]) * BigInt(registers[register_source2])) >> 32n)
			:	registers[register_source1] << (registers[register_source2] & 0b11111)
		);
		break;
	case 0b01100010: // slt/mulhsu
		registers[register_destination] = (
			instruction & (1 << 25) // mulhsu?
			?	Number((BigInt(registers[register_source1]) * BigInt(registers_unsigned[register_source2])) >> 32n)
			:	registers[register_source1] < registers[register_source2] ? 1 : 0
		);
		break;
	case 0b01100011: // sltu/mulhu
		registers[register_destination] = (
			instruction & (1 << 25) // mulhu
			?	Number((BigInt(registers_unsigned[register_source1]) * BigInt(registers_unsigned[register_source2])) >> 32n)
			:	registers_unsigned[register_source1] < registers_unsigned[register_source2] ? 1 : 0
		);
		break;
	case 0b01100100: // xor/div
		if (instruction & (1 << 25)) { // div?
			const dividend = registers[register_source1];
			const divisor = registers[register_source2];
			registers[register_destination] = (
				divisor === 0
				?	-1
				: dividend === -2147483648 && divisor === -1
				?	-2147483648
				:	dividend / divisor
			);
			break;
		}
		registers[register_destination] = registers[register_source1] ^ registers[register_source2];
		break;
	case 0b01100101: { // srl/sra/divu
		if (instruction & (1 << 25)) { // divu?
			const divisor = registers_unsigned[register_source2];
			registers_unsigned[register_destination] = (
				divisor === 0
				?	0xffffffff
				:	(registers_unsigned[register_source1] / divisor) >>> 0
			);
			break;
		}
		const shift_by = registers[register_source2] & 0b11111;
		if (instruction >>> 30) {
			registers[register_destination] = registers[register_source1] >> shift_by;
		}
		else {
			registers_unsigned[register_destination] = registers_unsigned[register_source1] >>> shift_by;
		}
		break;
	}
	case 0b01100110: // or/rem
		if (instruction & (1 << 25)) { // rem?
			const dividend = registers[register_source1];
			const divisor = registers[register_source2];
			registers[register_destination] = (
				divisor === 0
				?	dividend
				: dividend === -2147483648 && divisor === -1 // overflow?
				?	0
				:	dividend % divisor
			);
			break;
		}
		registers[register_destination] = registers[register_source1] | registers[register_source2];
		break;
	case 0b01100111: // and/remu
		if (instruction & (1 << 25)) { // remu?
			const dividend = registers_unsigned[register_source1];
			const divisor = registers_unsigned[register_source2];
			registers_unsigned[register_destination] = (
				divisor === 0
				?	dividend
				:	dividend % divisor
			);
			break;
		}
		registers[register_destination] = registers[register_source1] & registers[register_source2];
		break;
	case 0b01101000: // lui ;)
	case 0b01101001:
	case 0b01101010:
	case 0b01101011:
	case 0b01101100:
	case 0b01101101:
	case 0b01101110:
	case 0b01101111:
		registers[register_destination] = instruction & 0xfffff000;
		break;
	// fused multiply-add (F and D extensions)
	case 0b10000010: // fmadd.s
	case 0b10000011: { // fmadd.d
		const register_source3 = instruction >>> 27;
		const is_double = funct3 & 1;
		if (is_double) {
			fp_registers_f64[register_destination] = fp_registers_f64[register_source1] * fp_registers_f64[register_source2] + fp_registers_f64[register_source3];
		}
		else {
			fp_registers_f32[register_destination * 2] = fp_registers_f32[register_source1 * 2] * fp_registers_f32[register_source2 * 2] + fp_registers_f32[register_source3 * 2];
			fp_registers_u32[register_destination * 2 + 1] = 0xffffffff; // NaN-box
		}
		break;
	}
	case 0b10001010: // fmsub.s
	case 0b10001011: { // fmsub.d
		const register_source3 = instruction >>> 27;
		const is_double = funct3 & 1;
		if (is_double) {
			fp_registers_f64[register_destination] = fp_registers_f64[register_source1] * fp_registers_f64[register_source2] - fp_registers_f64[register_source3];
		}
		else {
			fp_registers_f32[register_destination * 2] = fp_registers_f32[register_source1 * 2] * fp_registers_f32[register_source2 * 2] - fp_registers_f32[register_source3 * 2];
			fp_registers_u32[register_destination * 2 + 1] = 0xffffffff; // NaN-box
		}
		break;
	}
	case 0b10010010: // fnmsub.s
	case 0b10010011: { // fnmsub.d
		const register_source3 = instruction >>> 27;
		const is_double = funct3 & 1;
		if (is_double) {
			fp_registers_f64[register_destination] = -(fp_registers_f64[register_source1] * fp_registers_f64[register_source2]) + fp_registers_f64[register_source3];
		}
		else {
			fp_registers_f32[register_destination * 2] = -(fp_registers_f32[register_source1 * 2] * fp_registers_f32[register_source2 * 2]) + fp_registers_f32[register_source3 * 2];
			fp_registers_u32[register_destination * 2 + 1] = 0xffffffff; // NaN-box
		}
		break;
	}
	case 0b10011010: // fnmadd.s
	case 0b10011011: { // fnmadd.d
		const register_source3 = instruction >>> 27;
		const is_double = funct3 & 1;
		if (is_double) {
			fp_registers_f64[register_destination] = -(fp_registers_f64[register_source1] * fp_registers_f64[register_source2] + fp_registers_f64[register_source3]);
		}
		else {
			fp_registers_f32[register_destination * 2] = -(fp_registers_f32[register_source1 * 2] * fp_registers_f32[register_source2 * 2] + fp_registers_f32[register_source3 * 2]);
			fp_registers_u32[register_destination * 2 + 1] = 0xffffffff; // NaN-box
		}
		break;
	}
	// floating-point operations
	case 0b10100000: // fadd.s/fadd.d
	case 0b10100001:
	case 0b10100010:
	case 0b10100011:
	case 0b10100100:
	case 0b10100101:
	case 0b10100110:
	case 0b10100111: {
		const funct7 = instruction >>> 25;
		const funct5 = funct7 >>> 2;
		const is_double = (funct7 & 1);
		
		switch (funct5) {
		case 0b00000: // fadd
			if (is_double) {
				fp_registers_f64[register_destination] = fp_registers_f64[register_source1] + fp_registers_f64[register_source2];
			}
			else {
				fp_registers_f32[register_destination * 2] = fp_registers_f32[register_source1 * 2] + fp_registers_f32[register_source2 * 2];
				fp_registers_u32[register_destination * 2 + 1] = 0xffffffff; // NaN-box
			}
			break;
		case 0b00001: // fsub
			if (is_double) {
				fp_registers_f64[register_destination] = fp_registers_f64[register_source1] - fp_registers_f64[register_source2];
			}
			else {
				fp_registers_f32[register_destination * 2] = fp_registers_f32[register_source1 * 2] - fp_registers_f32[register_source2 * 2];
				fp_registers_u32[register_destination * 2 + 1] = 0xffffffff; // NaN-box
			}
			break;
		case 0b00010: // fmul
			if (is_double) {
				fp_registers_f64[register_destination] = fp_registers_f64[register_source1] * fp_registers_f64[register_source2];
			}
			else {
				fp_registers_f32[register_destination * 2] = fp_registers_f32[register_source1 * 2] * fp_registers_f32[register_source2 * 2];
				fp_registers_u32[register_destination * 2 + 1] = 0xffffffff; // NaN-box
			}
			break;
		case 0b00011: // fdiv
			if (is_double) {
				fp_registers_f64[register_destination] = fp_registers_f64[register_source1] / fp_registers_f64[register_source2];
			}
			else {
				fp_registers_f32[register_destination * 2] = fp_registers_f32[register_source1 * 2] / fp_registers_f32[register_source2 * 2];
				fp_registers_u32[register_destination * 2 + 1] = 0xffffffff; // NaN-box
			}
			break;
		case 0b01011: // fsqrt
			if (is_double) {
				fp_registers_f64[register_destination] = Math.sqrt(fp_registers_f64[register_source1]);
			}
			else {
				fp_registers_f32[register_destination * 2] = Math.sqrt(fp_registers_f32[register_source1 * 2]);
				fp_registers_u32[register_destination * 2 + 1] = 0xffffffff; // NaN-box
			}
			break;
		case 0b00100: // fsgnj/fsgnjn/fsgnjx
			if (is_double) {
				const sign1 = fp_registers_u32[register_source1 * 2 + 1] >>> 31;
				const sign2 = fp_registers_u32[register_source2 * 2 + 1] >>> 31;
				fp_registers_u32[register_destination * 2] = fp_registers_u32[register_source1 * 2];
				fp_registers_u32[register_destination * 2 + 1] = (
					funct3 === 0b000 // fsgnj
					?	(fp_registers_u32[register_source1 * 2 + 1] & 0x7fffffff) | (sign2 << 31)
					: funct3 === 0b001 // fsgnjn
					?	(fp_registers_u32[register_source1 * 2 + 1] & 0x7fffffff) | ((sign2 ^ 1) << 31)
					:	(fp_registers_u32[register_source1 * 2 + 1] & 0x7fffffff) | ((sign1 ^ sign2) << 31) // fsgnjx
				);
			}
			else {
				const sign1 = fp_registers_u32[register_source1 * 2] >>> 31;
				const sign2 = fp_registers_u32[register_source2 * 2] >>> 31;
				fp_registers_u32[register_destination * 2] = (
					funct3 === 0b000 // fsgnj
					?	(fp_registers_u32[register_source1 * 2] & 0x7fffffff) | (sign2 << 31)
					: funct3 === 0b001 // fsgnjn
					?	(fp_registers_u32[register_source1 * 2] & 0x7fffffff) | ((sign2 ^ 1) << 31)
					:	(fp_registers_u32[register_source1 * 2] & 0x7fffffff) | ((sign1 ^ sign2) << 31) // fsgnjx
				);
				fp_registers_u32[register_destination * 2 + 1] = 0xffffffff; // NaN-box
			}
			break;
		case 0b00101: // fmin/fmax
			if (is_double) {
				const val1 = fp_registers_f64[register_source1];
				const val2 = fp_registers_f64[register_source2];
				fp_registers_f64[register_destination] = (
					funct3 === 0b000 // fmin
					?	Math.min(val1, val2)
					:	Math.max(val1, val2) // fmax
				);
			}
			else {
				const val1 = fp_registers_f32[register_source1 * 2];
				const val2 = fp_registers_f32[register_source2 * 2];
				fp_registers_f32[register_destination * 2] = (
					funct3 === 0b000 // fmin
					?	Math.min(val1, val2)
					:	Math.max(val1, val2) // fmax
				);
				fp_registers_u32[register_destination * 2 + 1] = 0xffffffff; // NaN-box
			}
			break;
		case 0b01000: // fcvt.s.d/fcvt.d.s
			if (is_double) { // fcvt.d.s
				fp_registers_f64[register_destination] = fp_registers_f32[register_source1 * 2];
			}
			else { // fcvt.s.d
				fp_registers_f32[register_destination * 2] = fp_registers_f64[register_source1];
				fp_registers_u32[register_destination * 2 + 1] = 0xffffffff; // NaN-box
			}
			break;
		case 0b10100: // fcmp (feq/flt/fle)
			if (is_double) {
				const val1 = fp_registers_f64[register_source1];
				const val2 = fp_registers_f64[register_source2];
				registers[register_destination] = (
					funct3 === 0b010 // feq
					?	(val1 === val2 ? 1 : 0)
					: funct3 === 0b001 // flt
					?	(val1 < val2 ? 1 : 0)
					:	(val1 <= val2 ? 1 : 0) // fle
				);
			}
			else {
				const val1 = fp_registers_f32[register_source1 * 2];
				const val2 = fp_registers_f32[register_source2 * 2];
				registers[register_destination] = (
					funct3 === 0b010 // feq
					?	(val1 === val2 ? 1 : 0)
					: funct3 === 0b001 // flt
					?	(val1 < val2 ? 1 : 0)
					:	(val1 <= val2 ? 1 : 0) // fle
				);
			}
			break;
		case 0b11000: // fcvt.w.s/fcvt.w.d/fcvt.wu.s/fcvt.wu.d
			if (is_double) {
				const val = fp_registers_f64[register_source1];
				if (register_source2 === 0b00000) { // fcvt.w.d
					registers[register_destination] = val;
				}
				else if (register_source2 === 0b00001) { // fcvt.wu.d
					registers_unsigned[register_destination] = val >>> 0;
				}
			}
			else {
				const val = fp_registers_f32[register_source1 * 2];
				if (register_source2 === 0b00000) { // fcvt.w.s
					registers[register_destination] = val;
				}
				else if (register_source2 === 0b00001) { // fcvt.wu.s
					registers_unsigned[register_destination] = val >>> 0;
				}
			}
			break;
		case 0b11010: // fcvt.s.w/fcvt.d.w/fcvt.s.wu/fcvt.d.wu
			if (is_double) {
				if (register_source2 === 0b00000) { // fcvt.d.w
					fp_registers_f64[register_destination] = registers[register_source1];
				}
				else if (register_source2 === 0b00001) { // fcvt.d.wu
					fp_registers_f64[register_destination] = registers_unsigned[register_source1];
				}
			}
			else {
				if (register_source2 === 0b00000) { // fcvt.s.w
					fp_registers_f32[register_destination * 2] = registers[register_source1];
				}
				else if (register_source2 === 0b00001) { // fcvt.s.wu
					fp_registers_f32[register_destination * 2] = registers_unsigned[register_source1];
				}
				fp_registers_u32[register_destination * 2 + 1] = 0xffffffff; // NaN-box
			}
			break;
		case 0b11100: // fmv.x.w/fmv.x.d/fclass
			if (funct3 === 0b000) { // fmv.x.w/fmv.x.d
				if (is_double) { // fmv.x.d (RV64 only, not implemented)
					throw 'illegal instruction';
				}
				else { // fmv.x.w
					registers[register_destination] = fp_registers_i32[register_source1 * 2];
				}
			}
			else if (funct3 === 0b001) { // fclass
				if (is_double) {
					const bits_hi = fp_registers_u32[register_source1 * 2 + 1];
					const bits_lo = fp_registers_u32[register_source1 * 2];
					const sign = bits_hi >>> 31;
					const exp = (bits_hi >>> 20) & 0x7ff;
					const mantissa = ((bits_hi & 0xfffff) << 32) | bits_lo;
					
					registers[register_destination] = (
						exp === 0 && mantissa === 0
						?	(sign ? 0x008 : 0x001) // -0 or +0
						: exp === 0
						?	(sign ? 0x010 : 0x002) // -subnormal or +subnormal
						: exp === 0x7ff && mantissa === 0
						?	(sign ? 0x080 : 0x004) // -inf or +inf
						: exp === 0x7ff
						?	0x200 // qNaN or sNaN
						:	(sign ? 0x040 : 0x020) // -normal or +normal
					);
				}
				else {
					const bits = fp_registers_u32[register_source1 * 2];
					const sign = bits >>> 31;
					const exp = (bits >>> 23) & 0xff;
					const mantissa = bits & 0x7fffff;
					
					registers[register_destination] = (
						exp === 0 && mantissa === 0
						?	(sign ? 0x008 : 0x001) // -0 or +0
						: exp === 0
						?	(sign ? 0x010 : 0x002) // -subnormal or +subnormal
						: exp === 0xff && mantissa === 0
						?	(sign ? 0x080 : 0x004) // -inf or +inf
						: exp === 0xff
						?	0x200 // qNaN or sNaN
						:	(sign ? 0x040 : 0x020) // -normal or +normal
					);
				}
			}
			break;
		case 0b11000: // fcvt.w.s/fcvt.w.d/fcvt.wu.s/fcvt.wu.d
			if (is_double) {
				const val = fp_registers_f64[register_source1];
				if (register_source2 === 0b00000) { // fcvt.w.d
					registers[register_destination] = val;
				}
				else if (register_source2 === 0b00001) { // fcvt.wu.d
					registers_unsigned[register_destination] = val >>> 0;
				}
			}
			else {
				const val = fp_registers_f32[register_source1 * 2];
				if (register_source2 === 0b00000) { // fcvt.w.s
					registers[register_destination] = val;
				}
				else if (register_source2 === 0b00001) { // fcvt.wu.s
					registers_unsigned[register_destination] = val >>> 0;
				}
			}
			break;
		case 0b11110: // fmv.w.x
			if (is_double) { // fmv.d.x (RV64 only, not implemented)
				throw 'illegal instruction';
			}
			else { // fmv.w.x
				fp_registers_i32[register_destination * 2] = registers[register_source1];
				fp_registers_u32[register_destination * 2 + 1] = 0xffffffff; // NaN-box
			}
			break;
		default:
			throw 'illegal floating-point instruction';
		}
		break;
	}
	case 0b11000000: // branch
	case 0b11000001:
	case 0b11000100:
	case 0b11000101:
	case 0b11000110:
	case 0b11000111:
		switch (funct3) {
		case 0b000: // beq
			if (registers[register_source1] === registers[register_source2]) break;
			break opcode;
		case 0b001: // bne
			if (registers[register_source1] !== registers[register_source2]) break;
			break opcode;
		case 0b100: // blt
			if (registers[register_source1] < registers[register_source2]) break;
			break opcode;
		case 0b101: // bge
			if (registers[register_source1] >= registers[register_source2]) break;
			break opcode;
		case 0b110: // bltu
			if (registers_unsigned[register_source1] < registers_unsigned[register_source2]) break;
			break opcode;
		case 0b111: // bgeu
			if (registers_unsigned[register_source1] >= registers_unsigned[register_source2]) break;
			break opcode;
		default:
			throw 'invalid branch condition';
		}
		program_counter = program_counter + ( // 12 bit offset, shifted one to the right
			instruction >> 31 << 10 | // 31 -> 10
			(register_destination & 0x1) << 9 | // dest -> 9
			instruction >>> 25 << 3 | // 30-25 -> 8-3
			register_destination >>> 2 // dest -> 2-0
		) | 0;
		if (program_counter & OOB_BITS_PC) throw 'out of bounds';
		return;
	case 0b11001000: // jalr
		registers[register_destination] = program_counter + 1 << 2;
		program_counter = registers[register_source1] + (instruction >> 20) >>> 2;
		if (program_counter & OOB_BITS_PC) throw 'out of bounds';
		return;
	case 0b11011000: // jal
	case 0b11011001:
	case 0b11011010:
	case 0b11011011:
	case 0b11011100:
	case 0b11011101:
	case 0b11011110:
	case 0b11011111:
		// exit on endless loop
		if (instruction >> 12 === 0) throw null;
		registers[register_destination] = program_counter + 1 << 2;
		program_counter = program_counter + ( // 20 bit offset, shifted one to the right
			instruction >> 31 << 18 | // 31 -> 19
			(instruction >>> 12 & 0xff) << 10 | // 19-12 -> 18-11
			(instruction >>> 20 & 0x1) << 9 | // 20 -> 10
			(instruction >>> 22 & 0x3ff) // 30-21 -> 9-0
		) | 0;
		if (program_counter & OOB_BITS_PC) throw 'out of bounds';
		return;
	default:
		throw 'illegal instruction';
	}

	program_counter = program_counter + 1 | 0;
	if (program_counter & OOB_BITS_PC) throw 'out of bounds';
}

// load program
const program_path = process.argv[2] || '../tests/count.bin';
console.log('loading ' + program_path);
require('fs').readFileSync(program_path).copy(memory8);
console.log('running');

const time_start = performance.now();
let instruction_count = 0;
try {
	do tick();
	while (++instruction_count < 1e7);
	console.log(`-----\nprogram timed out`);
}
catch (e) {
	if (e === null) console.log('-----\nprogram ended');
	else if (typeof e === 'string') {
		console.log('-----\nprogram failed: ' + e);
	}
	else throw e;
}
const runtime = performance.now() - time_start;
console.log(`ran ${instruction_count} instructions in ${Math.round(runtime)} ms`);
console.log(`execution speed: ${Math.round(instruction_count / runtime / 1e3)} MHz`);

console.log('registers:');
for (let i = 1; i < 32; i++) {
	console.log(`  ${
		('x' + i).padStart(3, ' ')
	} = 0x${
		(registers[i] >>> 0).toString(16).padStart(8, '0')
	} ${
		registers[i]
	}`);
}

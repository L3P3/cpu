'use strict';

const MEMORY_SIZE = 64 * 1024;

// Out of bounds bit masks for memory access validation
// Any address with these bits set is out of bounds
const OOB_BITS_8 = ~(MEMORY_SIZE - 1);        // byte access
const OOB_BITS_16 = ~(MEMORY_SIZE - 1 - 1);   // halfword access
const OOB_BITS_32 = ~(MEMORY_SIZE - 1 - 3);   // word access
const OOB_BITS_PC = ~(MEMORY_SIZE / 4 - 1);   // program counter (word index)

const registers = new Int32Array(32);
const registers_unsigned = new Uint32Array(registers.buffer);
const memory8 = new Uint8Array(MEMORY_SIZE);
const memory16 = new Uint16Array(memory8.buffer);
const memory32 = new Int32Array(memory8.buffer);

// index for 32 bit!
let program_counter = 0;

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
	case 0b00000000: {// lb
		const addr = registers[register_source1] + (instruction >> 20) | 0;
		if (addr & OOB_BITS_8) throw 'out of bounds';
		registers[register_destination] = memory8[addr] << 24 >> 24;
		break;
	}
	case 0b00000001: {// lh
		const addr = registers[register_source1] + (instruction >> 20) | 0;
		if (addr & OOB_BITS_16) throw 'out of bounds';
		registers[register_destination] = memory16[addr >>> 1] << 16 >> 16;
		break;
	}
	case 0b00000010: {// lw
		const addr = registers[register_source1] + (instruction >> 20) | 0;
		if (addr & OOB_BITS_32) throw 'out of bounds';
		registers[register_destination] = memory32[addr >>> 2];
		break;
	}
	case 0b00000100: {// lbu
		const addr = registers[register_source1] + (instruction >> 20) | 0;
		if (addr & OOB_BITS_8) throw 'out of bounds';
		registers[register_destination] = memory8[addr];
		break;
	}
	case 0b00000101: {// lhu
		const addr = registers[register_source1] + (instruction >> 20) | 0;
		if (addr & OOB_BITS_16) throw 'out of bounds';
		registers[register_destination] = memory16[addr >>> 1];
		break;
	}
	// fence
	// register+immediate
	case 0b00100000:// addi
		registers[register_destination] = registers[register_source1] + (instruction >> 20) | 0;
		break;
	case 0b00100001:// slli
		registers[register_destination] = registers[register_source1] << ((instruction >>> 20) & 0b11111);
		break;
	case 0b00100010:// slti
		registers[register_destination] = registers[register_source1] < instruction >> 20 ? 1 : 0;
		break;
	case 0b00100011:// sltiu
		registers[register_destination] = registers_unsigned[register_source1] < instruction >>> 20 ? 1 : 0;
		break;
	case 0b00100100:// xori
		registers[register_destination] = registers[register_source1] ^ instruction >> 20;
		break;
	case 0b00100101: {// srli/srai
		const shift_by = (instruction >>> 20) & 0b11111;
		if (instruction >>> 30) {
			registers[register_destination] = registers[register_source1] >> shift_by;
		}
		else {
			registers_unsigned[register_destination] = registers_unsigned[register_source1] >>> shift_by;
		}
		break;
	}
	case 0b00100110:// ori
		registers[register_destination] = registers[register_source1] | instruction >> 20;
		break;
	case 0b00100111:// andi
		registers[register_destination] = registers[register_source1] & instruction >> 20;
		break;
	case 0b00101000:// auipc
	case 0b00101001:
	case 0b00101010:
	case 0b00101011:
	case 0b00101100:
	case 0b00101101:
	case 0b00101110:
	case 0b00101111:
		registers[register_destination] = (program_counter << 2) + (instruction & 0xfffff000) | 0;
		break;
	// store
	case 0b01000000: {// sb
		const addr = registers[register_source1] + (instruction >> 25 << 5 | register_destination) | 0;
		if (addr & OOB_BITS_8) throw 'out of bounds';
		memory8[addr] = registers[register_source2];
		break;
	}
	case 0b01000001: {// sh
		const addr = registers[register_source1] + (instruction >> 25 << 5 | register_destination) | 0;
		if (addr & OOB_BITS_16) throw 'out of bounds';
		memory16[addr >>> 1] = registers[register_source2];
		break;
	}
	case 0b01000010: {// sw
		const addr = registers[register_source1] + (instruction >> 25 << 5 | register_destination) | 0;
		if (addr & OOB_BITS_32) throw 'out of bounds';
		memory32[addr >>> 2] = registers[register_source2];
		break;
	}
	// register+register
	case 0b01100000:// add/sub
		registers[register_destination] = (
			instruction >>> 30
			?	registers[register_source1] - registers[register_source2]
			:	registers[register_source1] + registers[register_source2]
		);
		break;
	case 0b01100001:// sll
		registers[register_destination] = registers[register_source1] << (registers[register_source2] & 0b11111);
		break;
	case 0b01100010:// slt
		registers[register_destination] = registers[register_source1] < registers[register_source2] ? 1 : 0;
		break;
	case 0b01100011:// sltu
		registers[register_destination] = registers_unsigned[register_source1] < registers_unsigned[register_source2] ? 1 : 0;
		break;
	case 0b01100100:// xor
		registers[register_destination] = registers[register_source1] ^ registers[register_source2];
		break;
	case 0b01100101: {// srl/sra
		const shift_by = registers[register_source2] & 0b11111;
		if (instruction >>> 30) {
			registers[register_destination] = registers[register_source1] >> shift_by;
		}
		else {
			registers_unsigned[register_destination] = registers_unsigned[register_source1] >>> shift_by;
		}
		break;
	}
	case 0b01100110:// or
		registers[register_destination] = registers[register_source1] | registers[register_source2];
		break;
	case 0b01100111:// and
		registers[register_destination] = registers[register_source1] & registers[register_source2];
		break;
	case 0b01101000:// lui ;)
	case 0b01101001:
	case 0b01101010:
	case 0b01101011:
	case 0b01101100:
	case 0b01101101:
	case 0b01101110:
	case 0b01101111:
		registers[register_destination] = instruction & 0xfffff000;
		break;
	case 0b11000000:// branch
	case 0b11000001:
	case 0b11000100:
	case 0b11000101:
	case 0b11000110:
	case 0b11000111:
		switch (funct3) {
		case 0b000:// beq
			if (registers[register_source1] === registers[register_source2]) break;
			break opcode;
		case 0b001:// bne
			if (registers[register_source1] !== registers[register_source2]) break;
			break opcode;
		case 0b100:// blt
			if (registers[register_source1] < registers[register_source2]) break;
			break opcode;
		case 0b101:// bge
			if (registers[register_source1] >= registers[register_source2]) break;
			break opcode;
		case 0b110:// bltu
			if (registers_unsigned[register_source1] < registers_unsigned[register_source2]) break;
			break opcode;
		case 0b111:// bgeu
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
	case 0b11001000:// jalr
		registers[register_destination] = program_counter + 1 << 2;
		program_counter = registers[register_source1] + (instruction >> 20) >>> 2;
		if (program_counter & OOB_BITS_PC) throw 'out of bounds';
		return;
	case 0b11011000:// jal
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

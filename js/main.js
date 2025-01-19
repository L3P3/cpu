const MEMORY_SIZE = 64 * 1024;

const registers = new Int32Array(32);
const registers_unsigned = new Uint32Array(registers);
const memory8 = new Uint8Array(MEMORY_SIZE);
const memory16 = new Uint16Array(memory8);
const memory32 = new Int32Array(memory8);
// index for 32 bit!
let program_counter = 0;

function tick() {
	// make it constant
	registers[0] = 0;

	const instruction = memory32[program_counter];
	const opcode_1 = (instruction >>> 2) & 0b11111;
	const register_destination = (instruction >>> 7) & 0b11111;
	const opcode_2 = (instruction >>> 12) & 0b111;
	const register_source1 = (instruction >>> 15) & 0b11111;
	const register_source2 = (instruction >>> 20) & 0b11111;

	opcode: switch (opcode_1) {
	case 0b00000:// load
		switch (opcode_2) {
		case 0b000:// lb
			registers[register_destination] = (memory8[registers[register_source1] + (instruction >> 20)] << 24) >>> 24;
			break;
		case 0b001:// lh
			registers[register_destination] = (memory8[(registers[register_source1] + (instruction >> 20)) >>> 1] << 16) >>> 16;
			break;
		case 0b010:// lw
			registers[register_destination] = memory32[(registers[register_source1] + (instruction >> 20)) >>> 2];
			break;
		case 0b100:// lbu
			registers[register_destination] = memory8[registers[register_source1] + (instruction >> 20)];
			break;
		case 0b101:// lhu
			registers[register_destination] = memory16[(registers[register_source1] + (instruction >> 20)) >>> 1];
			break;
		default:
			throw new Error('invalid load mode');
		}
		break;
	case 0b00011:// fence
		throw new Error('fence not implemented, wtf is that?');
	case 0b00100:// register+immediate
		switch (opcode_2) {
		case 0b000:// addi
			registers[register_destination] = (registers[register_source1] + (instruction >> 20)) | 0;
			break;
		case 0b001:// slli
			registers[register_destination] = registers[register_source1] << (instruction >>> 20);
			break;
		case 0b010:// slti
			registers[register_destination] = (registers[register_source1] < (instruction >> 20)) | 0;
			break;
		case 0b011:// sltiu
			registers[register_destination] = (registers_unsigned[register_source1] < (instruction >>> 20)) | 0;
			break;
		case 0b100:// xori
			registers[register_destination] = registers[register_source1] ^ (instruction >> 20);
			break;
		case 0b101:// srli
			registers[register_destination] = registers[register_source1] >>> (instruction >>> 20);
			break;
		case 0b110:// ori
			registers[register_destination] = registers[register_source1] | (instruction >> 20);
			break;
		case 0b111:// andi
			registers[register_destination] = registers[register_source1] & (instruction >> 20);
		}
		break;
	case 0b00101:// auipc
		register[register_destination] = ((program_counter << 2) + (instruction & 0xfffff000)) | 0;
		break;
	case 0b01000: {// store
		const offset = instruction >> 25;
		switch (opcode_2) {
		case 0b000:// sb
			memory8[registers_unsigned[register_source1] + offset] = registers[register_source2] & 0xff;
			break;
		case 0b001:// sh
			memory16[(registers_unsigned[register_source1] + offset) >>> 1] = registers[register_source2] & 0xffff;
			break;
		case 0b010:// sw
			memory32[(registers_unsigned[register_source1] + offset) >>> 2] = registers[register_source2];
			break;
		default:
			throw new Error('invalid store size');
		}
		break;
	}
	case 0b01100:// register+register
		switch (opcode_2) {
		case 0b000:// add/sub
			registers[register_destination] = (
				instruction >>> 30
				?	(registers[register_source1] - registers[register_source2]) | 0
				:	(registers[register_source1] + registers[register_source2]) | 0
			);
			break;
		case 0b001:// sll
			registers[register_destination] = registers[register_source1] << (registers[register_source2] & 0b11111);
			break;
		case 0b010:// slt
			registers[register_destination] = (registers[register_source1] < registers[register_source2]) | 0;
			break;
		case 0b011:// sltu
			registers[register_destination] = (registers_unsigned[register_source1] < registers_unsigned[register_source2]) | 0;
			break;
		case 0b100:// xor
			registers[register_destination] = registers[register_source1] ^ registers[register_source2];
			break;
		case 0b101: {// srl/sra
			const shift_by = registers[register_source2] & 0b11111;
			registers[register_destination] = (
				instruction >>> 30
				?	registers[register_source1] >>> shift_by
				:	registers[register_source1] >> shift_by
			);
			break;
		}
		case 0b110:// or
			registers[register_destination] = registers[register_source1] | registers[register_source2];
			break;
		case 0b111:// and
			registers[register_destination] = registers[register_source1] & registers[register_source2];
		}
		break;
	case 0b01101:// lui ;)
		registers[register_destination] = instruction & 0xfffff000;
		break;
	case 0b11000:// branch
		switch (opcode_2) {
		case 0b000:// beq
			if (registers[register_source1] !== registers[register_source2]) break opcode;
			break;
		case 0b001:// bne
			if (registers[register_source1] === registers[register_source2]) break opcode;
			break;
		case 0b100:// blt
			if (registers[register_source1] >= registers[register_source2]) break opcode;
			break;
		case 0b101:// bge
			if (registers[register_source1] < registers[register_source2]) break opcode;
			break;
		case 0b110:// bltu
			if (registers_unsigned[register_source1] >= registers_unsigned[register_source2]) break opcode;
			break;
		case 0b111:// bgeu
			if (registers_unsigned[register_source1] < registers_unsigned[register_source2]) break opcode;
			break;
		default:
			throw new Error('invalid branch condition');
		}
		program_counter = (program_counter + ((instruction << 20) >> 7)) | 0;
		return;
	case 0b11001:// jalr
		registers[register_destination] = (program_counter + 1) << 2;
		program_counter = (registers[register_source1] + (instruction >> 20)) >>> 2;
		return;
	case 0b11011:// jal
		registers[register_destination] = (program_counter + 1) << 2;
		program_counter = instruction >> 12;
		return;
	case 0b11100:// advanced stuff
		throw new Error('shit is not implemented');
	default:
		throw new Error('unknown opcode');
	}

	program_counter = (program_counter + 1) | 0;
}

// test program
memory32[0] = 0x00050593;

for (let instruction_count = 0; instruction_count < 1e6; instruction_count++) {
	tick();
}

console.log('result:');
for (let i = 0; i < 32; i++) {
	console.log(`x${i} = ${registers[i]}`);
}

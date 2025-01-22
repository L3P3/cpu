#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

#define MEMORY_SIZE 64 * 1024

int32_t registers[32];
uint32_t *registers_unsigned = (uint32_t *) registers;
uint8_t memory8[MEMORY_SIZE];
uint16_t *memory16 = (uint16_t *) memory8;
int32_t *memory32 = (int32_t *) memory8;

// index for 32 bit!
uint32_t program_counter = 0;

void error(const char *message) {
	fprintf(stderr, "Error: %s\n", message);
	exit(1);
}

void memory_bounds_check(uint32_t index) {
	if (index >= MEMORY_SIZE / 4) {
		error("out of bounds memory access");
	}
}

void tick() {
	// make it constant
	registers[0] = 0;

	memory_bounds_check(program_counter);
	uint32_t instruction = memory32[program_counter];
	uint32_t opcode_1 = (instruction >> 2) & 0b11111;
	uint32_t register_destination = (instruction >> 7) & 0b11111;
	uint32_t opcode_2 = (instruction >> 12) & 0b111;
	uint32_t register_source1 = (instruction >> 15) & 0b11111;
	uint32_t register_source2 = (instruction >> 20) & 0b11111;

	switch (opcode_1) {
	case 0b00000: {// load
		memory_bounds_check(instruction >> 22);
		int32_t data = memory[instruction >> 22];
		switch (opcode_2) {
		case 0b000:// lb
			registers[register_destination] = (int8_t) memory8[registers[register_source1] + (instruction >> 20)];
			break;
		case 0b001:// lh
			registers[register_destination] = (int16_t) memory16[(registers[register_source1] + (instruction >> 20)) >> 1];
			break;
		case 0b010:// lw
			registers[register_destination] = memory[(registers[register_source1] + (instruction >> 20)) >> 2];
			break;
		case 0b100:// lbu
			registers[register_destination] = memory8[registers[register_source1] + (instruction >> 20)];
			break;
		case 0b101:// lhu
			registers[register_destination] = memory16[(registers[register_source1] + (instruction >> 20)) >> 1];
			break;
		default:
			error("invalid load mode");
		}
		break;
	}
	case 0b00011:// fence
		error("fence not implemented, wtf is that?");
	case 0b00100:// register+immediate
		switch (opcode_2) {
		case 0b000:// addi
			registers[register_destination] = (registers[register_source1] + (instruction >> 20));
			break;
		case 0b001:// slli
			registers[register_destination] = registers[register_source1] << (instruction >> 20);
			break;
		case 0b010:// slti
			registers[register_destination] = (registers[register_source1] < (instruction >> 20));
			break;
		case 0b011:// sltiu
			registers[register_destination] = (registers_unsigned[register_source1] < (instruction >> 20));
			break;
		case 0b100:// xori
			registers[register_destination] = registers[register_source1] ^ (instruction >> 20);
			break;
		case 0b101:// srli
			registers[register_destination] = registers[register_source1] >> (instruction >> 20);
			break;
		case 0b110:// ori
			registers[register_destination] = registers[register_source1] | (instruction >> 20);
			break;
		case 0b111:// andi
			registers[register_destination] = registers[register_source1] & (instruction >> 20);
		}
		break;
	case 0b00101:// auipc
		registers[register_destination] = ((program_counter << 2) + (instruction & 0xfffff000));
		break;
	case 0b01000: {// store
		uint32_t offset = instruction >> 25;
		memory_bounds_check(registers_unsigned[register_source1] + offset);
		switch (opcode_2) {
		case 0b000:// sb
			memory8[registers_unsigned[register_source1] + offset] = registers[register_source2] & 0xff;
			break;
		case 0b001:// sh
			memory_bounds_check((registers_unsigned[register_source1] + offset) & ~1); // Ensure 2-byte alignment for sh
			memory16[(registers_unsigned[register_source1] + offset) >> 1] = registers[register_source2] & 0xffff;
			break;
		case 0b010:// sw
			memory_bounds_check((registers_unsigned[register_source1] + offset) & ~3); // Ensure 4-byte alignment for sw
			memory[(registers_unsigned[register_source1] + offset) >> 2] = registers[register_source2];
			break;
		default:
			error("invalid store size");
		}
		break;
	}
	case 0b01100:// register+register
		switch (opcode_2) {
		case 0b000:// add/sub
			registers[register_destination] = (
				instruction >> 30
				?	(registers[register_source1] - registers[register_source2])
				:	(registers[register_source1] + registers[register_source2])
			);
			break;
		case 0b001:// sll
			registers[register_destination] = registers[register_source1] << (registers[register_source2] & 0b11111);
			break;
		case 0b010:// slt
			registers[register_destination] = (registers[register_source1] < registers[register_source2]);
			break;
		case 0b011:// sltu
			registers[register_destination] = (registers_unsigned[register_source1] < registers_unsigned[register_source2]);
			break;
		case 0b100:// xor
			registers[register_destination] = registers[register_source1] ^ registers[register_source2];
			break;
		case 0b101: {// srl/sra
			uint32_t shift_by = registers[register_source2] & 0b11111;
			registers[register_destination] = (
				instruction >> 30
				?	registers[register_source1] >> shift_by
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
			if (registers[register_source1] != registers[register_source2]) goto after_switch;
			program_counter += (offset << 1);
			return;
		case 0b001:// bne
			if (registers[register_source1] == registers[register_source2]) goto after_switch;
			program_counter += (offset << 1);
			return;
		case 0b100:// blt
			if (registers[register_source1] >= registers[register_source2]) goto after_switch;
			program_counter += (offset << 1);
			return;
		case 0b101:// bge
			if (registers[register_source1] < registers[register_source2]) goto after_switch;
			program_counter += (offset << 1);
			return;
		case 0b110:// bltu
			if (registers_unsigned[register_source1] >= registers_unsigned[register_source2]) goto after_switch;
			program_counter += (offset << 1);
			return;
		case 0b111:// bgeu
			if (registers_unsigned[register_source1] < registers_unsigned[register_source2]) goto after_switch;
			break;
		default:
			error("invalid branch condition");
		}
		program_counter += ((instruction << 20) >> 7);
		return;
	case 0b11001:// jalr
		registers[register_destination] = (program_counter + 1) << 2;
		program_counter = (registers[register_source1] + (instruction >> 20)) & ~0x3;
		return;
	case 0b11011:// jal
		registers[register_destination] = (program_counter + 1) << 2;
		program_counter = instruction >> 12;
		return;
	case 0b11100:// advanced stuff 
		error("shit is not implemented");
	default:
		error("unknown opcode");
	}
after_switch:
	program_counter++;
}

int main() {
	// test program
	memory[0] = 0x00050593;

	for (uint32_t instruction_count = 0; instruction_count < 1e6; instruction_count++) {
		tick();
	}

	printf("result:\n");
	for (uint8_t i = 0; i < 32; i++) {
		printf("x%d = %d\n", i, registers[i]);
	}
}

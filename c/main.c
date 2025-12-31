#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <stdbool.h>

#define MEMORY_SIZE 64 * 1024

int32_t registers[32];
uint32_t *registers_unsigned = (uint32_t *) registers;
uint8_t memory8[MEMORY_SIZE];
uint16_t *memory16 = (uint16_t *) memory8;
int32_t *memory32 = (int32_t *) memory8;

// index for 32 bit!
uint32_t program_counter = 0;

// Exit flag for program termination
bool program_ended = false;
const char *error_message = NULL;

void tick() {
	// make it constant
	registers[0] = 0;

	uint32_t instruction = memory32[program_counter];

	uint32_t funct3 = (instruction >> 12) & 0b111;

	uint32_t register_destination = (instruction >> 7) & 0b11111;
	uint32_t register_source1 = (instruction >> 15) & 0b11111;
	uint32_t register_source2 = (instruction >> 20) & 0b11111;

	uint8_t opcode_combined = (((instruction >> 2) << 3) & 0xff) | funct3;

	switch (opcode_combined) {
	// load
	case 0b00000000:// lb
		registers[register_destination] = (int8_t) memory8[registers[register_source1] + (int32_t)(instruction >> 20)];
		break;
	case 0b00000001:// lh
		registers[register_destination] = (int16_t) memory16[(registers[register_source1] + (int32_t)(instruction >> 20)) >> 1];
		break;
	case 0b00000010:// lw
		registers[register_destination] = memory32[(registers[register_source1] + (int32_t)(instruction >> 20)) >> 2];
		break;
	case 0b00000100:// lbu
		registers[register_destination] = memory8[registers[register_source1] + (int32_t)(instruction >> 20)];
		break;
	case 0b00000101:// lhu
		registers[register_destination] = memory16[(registers[register_source1] + (int32_t)(instruction >> 20)) >> 1];
		break;
	// fence
	// register+immediate
	case 0b00100000:// addi
		registers[register_destination] = registers[register_source1] + (int32_t)(instruction >> 20);
		break;
	case 0b00100001:// slli
		registers[register_destination] = registers[register_source1] << (instruction >> 20);
		break;
	case 0b00100010:// slti
		registers[register_destination] = registers[register_source1] < (int32_t)(instruction >> 20) ? 1 : 0;
		break;
	case 0b00100011:// sltiu
		registers[register_destination] = registers_unsigned[register_source1] < (instruction >> 20) ? 1 : 0;
		break;
	case 0b00100100:// xori
		registers[register_destination] = registers[register_source1] ^ (int32_t)(instruction >> 20);
		break;
	case 0b00100101:// srli
		registers_unsigned[register_destination] = registers_unsigned[register_source1] >> (instruction >> 20);
		break;
	case 0b00100110:// ori
		registers[register_destination] = registers[register_source1] | (int32_t)(instruction >> 20);
		break;
	case 0b00100111:// andi
		registers[register_destination] = registers[register_source1] & (int32_t)(instruction >> 20);
		break;
	case 0b00101000:// auipc
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
	case 0b01000000:// sb
		memory8[registers[register_source1] + ((int32_t)(instruction >> 25) << 5 | register_destination)] = registers[register_source2];
		break;
	case 0b01000001:// sh
		memory16[(registers[register_source1] + ((int32_t)(instruction >> 25) << 5 | register_destination)) >> 1] = registers[register_source2];
		break;
	case 0b01000010:// sw
		memory32[(registers[register_source1] + ((int32_t)(instruction >> 25) << 5 | register_destination)) >> 2] = registers[register_source2];
		break;
	// register+register
	case 0b01100000: {// add/sub
		registers[register_destination] = (
			instruction >> 30
			?	registers[register_source1] - registers[register_source2]
			:	registers[register_source1] + registers[register_source2]
		);
		break;
	}
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
		uint32_t shift_by = registers[register_source2] & 0b11111;
		if (instruction >> 30) {
			registers_unsigned[register_destination] = registers_unsigned[register_source1] >> shift_by;
		}
		else {
			registers[register_destination] = registers[register_source1] >> shift_by;
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
			if (registers[register_source1] == registers[register_source2]) break;
			goto no_branch;
		case 0b001:// bne
			if (registers[register_source1] != registers[register_source2]) break;
			goto no_branch;
		case 0b100:// blt
			if (registers[register_source1] < registers[register_source2]) break;
			goto no_branch;
		case 0b101:// bge
			if (registers[register_source1] >= registers[register_source2]) break;
			goto no_branch;
		case 0b110:// bltu
			if (registers_unsigned[register_source1] < registers_unsigned[register_source2]) break;
			goto no_branch;
		case 0b111:// bgeu
			if (registers_unsigned[register_source1] >= registers_unsigned[register_source2]) break;
			goto no_branch;
		default:
			error_message = "invalid branch condition";
			return;
		}
		program_counter = program_counter + ( // 12 bit offset, shifted one to the right
			(int32_t)(instruction >> 31) << 10 | // 31 -> 10
			(register_destination & 0x1) << 9 | // dest -> 9
			(instruction >> 25) << 3 | // 30-25 -> 8-3
			register_destination >> 2 // dest -> 2-0
		);
		return;
	case 0b11001000:// jalr
		registers[register_destination] = (program_counter + 1) << 2;
		program_counter = (registers[register_source1] + (int32_t)(instruction >> 20)) >> 2;
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
		if (instruction >> 12 == 0) {
			program_ended = true;
			return;
		}
		registers[register_destination] = (program_counter + 1) << 2;
		program_counter = program_counter + ( // 20 bit offset, shifted one to the right
			(int32_t)(instruction >> 31) << 18 | // 31 -> 19
			((instruction >> 12) & 0xff) << 10 | // 19-12 -> 18-11
			((instruction >> 20) & 0x1) << 9 | // 20 -> 10
			((instruction >> 22) & 0x3ff) // 30-21 -> 9-0
		);
		return;
	default:
		error_message = "illegal instruction";
		return;
	}

no_branch:
	program_counter = program_counter + 1;
}

int main(int argc, char *argv[]) {
	// load program
	const char *program_path = argc > 1 ? argv[1] : "../tests/count.bin";
	printf("loading %s\n", program_path);
	
	FILE *file = fopen(program_path, "rb");
	if (!file) {
		fprintf(stderr, "Failed to open file: %s\n", program_path);
		return 1;
	}
	
	fread(memory8, 1, MEMORY_SIZE, file);
	fclose(file);
	
	printf("running\n");
	
	clock_t time_start = clock();
	uint32_t instruction_count = 0;
	
	do {
		tick();
		
		if (program_ended) {
			printf("-----\nprogram ended\n");
			break;
		}
		
		if (error_message != NULL) {
			printf("-----\nprogram failed: %s\n", error_message);
			break;
		}
	} while (++instruction_count < 10000000);
	
	// If we reach here without break, program timed out
	if (!program_ended && error_message == NULL) {
		printf("-----\nprogram timed out\n");
	}
	
	clock_t time_end = clock();
	double runtime = ((double)(time_end - time_start)) / CLOCKS_PER_SEC * 1000;
	
	printf("ran %u instructions in %.0f ms\n", instruction_count, runtime);
	printf("execution speed: %.0f MHz\n", instruction_count / runtime / 1000);
	
	printf("registers:\n");
	for (uint8_t i = 1; i < 32; i++) {
		printf("  x%-2d = 0x%08x %d\n", i, (uint32_t)registers[i], registers[i]);
	}
	
	return 0;
}

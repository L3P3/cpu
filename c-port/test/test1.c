#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

#define NUM_REGISTERS 32
#define MEMORY_SIZE 64 * 1024

int32_t registers[NUM_REGISTERS];
uint32_t registers_unsigned[NUM_REGISTERS];
uint8_t memory8[MEMORY_SIZE];
uint16_t memory16[MEMORY_SIZE / 2];
int32_t memory32[MEMORY_SIZE / 4];

uint32_t program_counter = 0;

void checkmembounds(uint32_t address, uint32_t size) {
    // null
}

void tick() {
    registers[0] = 0;

    uint32_t instruction = memory32[program_counter];
    uint32_t opcode_1 = (instruction >> 2) & 0b11111;
    uint32_t register_destination = (instruction >> 7) & 0b11111;
    uint32_t opcode_2 = (instruction >> 12) & 0b111;
    uint32_t register_source1 = (instruction >> 15) & 0b11111;
    uint32_t register_source2 = (instruction >> 20) & 0b11111;

    switch (opcode_1) {
        case 0b00000: // load
            switch (opcode_2) {
                case 0b000: // lb
                    registers[register_destination] = (int8_t) memory8[registers[register_source1] + (instruction >> 20)];
                    break;
                case 0b001: // lh
                    registers[register_destination] = (int16_t) memory16[(registers[register_source1] + (instruction >> 20)) >> 1];
                    break;
                case 0b010: // lw
                    registers[register_destination] = memory32[(registers[register_source1] + (instruction >> 20)) >> 2];
                    break;
                case 0b100: // lbu
                    registers[register_destination] = memory8[registers[register_source1] + (instruction >> 20)];
                    break;
                case 0b101: // lhu
                    registers[register_destination] = memory16[(registers[register_source1] + (instruction >> 20)) >> 1];
                    break;
                default:
                    return;
            }
            break;

        case 0b00100: // register + immediate
            switch (opcode_2) {
                case 0b000: // addi
                    registers[register_destination] = registers[register_source1] + (instruction >> 20);
                    break;
                case 0b001: // slli
                    registers[register_destination] = registers[register_source1] << (instruction >> 20);
                    break;
                case 0b010: // slti
                    registers[register_destination] = registers[register_source1] < (instruction >> 20);
                    break;
                case 0b011: // sltiu
                    registers[register_destination] = registers_unsigned[register_source1] < (instruction >> 20);
                    break;
                case 0b100: // xori
                    registers[register_destination] = registers[register_source1] ^ (instruction >> 20);
                    break;
                case 0b101: // srli
                    registers[register_destination] = registers[register_source1] >> (instruction >> 20);
                    break;
                case 0b110: // ori
                    registers[register_destination] = registers[register_source1] | (instruction >> 20);
                    break;
                case 0b111: // andi
                    registers[register_destination] = registers[register_source1] & (instruction >> 20);
                    break;
                default:
                    return;
            }
            break;

        case 0b01100: // register + register
            switch (opcode_2) {
                case 0b000: // add/sub
                    registers[register_destination] = (instruction >> 30)
                        ? (registers[register_source1] - registers[register_source2])
                        : (registers[register_source1] + registers[register_source2]);
                    break;
                case 0b001: // sll
                    registers[register_destination] = registers[register_source1] << (registers[register_source2] & 0b11111);
                    break;
                case 0b010: // slt
                    registers[register_destination] = registers[register_source1] < registers[register_source2];
                    break;
                case 0b011: // sltu
                    registers[register_destination] = registers_unsigned[register_source1] < registers_unsigned[register_source2];
                    break;
                case 0b100: // xor
                    registers[register_destination] = registers[register_source1] ^ registers[register_source2];
                    break;
                case 0b101: // srl/sra
                    registers[register_destination] = registers[register_source1] >> (registers[register_source2] & 0b11111);
                    break;
                case 0b110: // or
                    registers[register_destination] = registers[register_source1] | registers[register_source2];
                    break;
                case 0b111: // and
                    registers[register_destination] = registers[register_source1] & registers[register_source2];
                    break;
                default:
                    return;
            }
            break;

        case 0b11001: // jalr
            registers[register_destination] = (program_counter + 1) << 2;
            program_counter = (registers[register_source1] + (instruction >> 20)) & ~0x3;
            return;

        case 0b11011: // jal
            registers[register_destination] = (program_counter + 1) << 2;
            program_counter = instruction >> 12;
            return;

        case 0b11100: {
            fprintf(stderr, "Shit is not implemented\n");
            return;
        }

        default:
            return;
    }

    program_counter = (program_counter + 1) & ~0x3;
}

int main() {
    memset(memory8, 0, MEMORY_SIZE);
    memset(registers, 0, sizeof(registers));
    memset(memory32, 0, sizeof(memory32));

    uint32_t test_instruction = 0x00050593;
    memory32[0] = test_instruction;

    clock_t start = clock();
    uint32_t instruction_count = 0;

    
    while ((clock() - start) < CLOCKS_PER_SEC) {
        instruction_count++;
        tick();
    }

    printf("Executed %u instructions in 1 second.\n", instruction_count);

    return 0;
}

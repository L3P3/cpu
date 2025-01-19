#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define NUM_REGISTERS 32
#define MEMORY_SIZE 64 * 1024

int32_t registers[NUM_REGISTERS];
uint32_t registers_unsigned[NUM_REGISTERS];
uint8_t memory8[MEMORY_SIZE];
uint16_t memory16[MEMORY_SIZE / 2];
int32_t memory32[MEMORY_SIZE / 4];

uint32_t program_counter = 0;

void checkmembounds(uint32_t address, uint32_t size) {
  //  if (address >= MEMORY_SIZE || (address + size) > MEMORY_SIZE) {
    //    fprintf(stderr, "Error: Memory access out of bounds: address %u, size %u\n", address, size);
      //  exit(1);
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
                    checkmembounds(registers[register_source1] + (instruction >> 20), 1);
                    registers[register_destination] = (int8_t) memory8[registers[register_source1] + (instruction >> 20)];
                    break;
                case 0b001: // lh
                    checkmembounds((registers[register_source1] + (instruction >> 20)) & ~1, 2);
                    registers[register_destination] = (int16_t) memory16[(registers[register_source1] + (instruction >> 20)) >> 1];
                    break;
                case 0b010: // lw
                    checkmembounds((registers[register_source1] + (instruction >> 20)) & ~3, 4);
                    registers[register_destination] = memory32[(registers[register_source1] + (instruction >> 20)) >> 2];
                    break;
                case 0b100: // lbu
                    checkmembounds(registers[register_source1] + (instruction >> 20), 1);
                    registers[register_destination] = memory8[registers[register_source1] + (instruction >> 20)];
                    break;
                case 0b101: // lhu
                    checkmembounds((registers[register_source1] + (instruction >> 20)) & ~1, 2);
                    registers[register_destination] = memory16[(registers[register_source1] + (instruction >> 20)) >> 1];
                    break;
                default:
                    fprintf(stderr, "Error: invalid load mode\n");
                    exit(1);
            }
            break;

        case 0b00011: // fence (not implemented)
            fprintf(stderr, "Error: fence not implemented, wtf is that?\n");
            exit(1);

        case 0b00100: // register+immediate
            switch (opcode_2) {
                case 0b000: // addi
                    registers[register_destination] = (registers[register_source1] + (instruction >> 20));
                    break;
                case 0b001: // slli
                    registers[register_destination] = registers[register_source1] << (instruction >> 20);
                    break;
                case 0b010: // slti
                    registers[register_destination] = (registers[register_source1] < (instruction >> 20));
                    break;
                case 0b011: // sltiu
                    registers[register_destination] = (registers_unsigned[register_source1] < (instruction >> 20));
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
                    fprintf(stderr, "Error: unknown immediate operation\n");
                    exit(1);
            }
            break;

        case 0b00101: // auipc
            registers[register_destination] = ((program_counter << 2) + (instruction & 0xfffff000));
            break;

        case 0b01000: { // store
            uint32_t offset = instruction >> 25;
            checkmembounds(registers_unsigned[register_source1] + offset, 1); // Validate store memory bounds
            switch (opcode_2) {
                case 0b000: // sb
                    memory8[registers_unsigned[register_source1] + offset] = registers[register_source2] & 0xff;
                    break;
                case 0b001: // sh
                    checkmembounds((registers_unsigned[register_source1] + offset) & ~1, 2); // Ensure 2-byte alignment for sh
                    memory16[(registers_unsigned[register_source1] + offset) >> 1] = registers[register_source2] & 0xffff;
                    break;
                case 0b010: // sw
                    checkmembounds((registers_unsigned[register_source1] + offset) & ~3, 4); // Ensure 4-byte alignment for sw
                    memory32[(registers_unsigned[register_source1] + offset) >> 2] = registers[register_source2];
                    break;
                default:
                    fprintf(stderr, "Error: invalid store size\n");
                    exit(1);
            }
            break;
        }

        case 0b01100: // register+register
            switch (opcode_2) {
                case 0b000: // add/sub
                    registers[register_destination] = (
                        instruction >> 30
                        ? (registers[register_source1] - registers[register_source2])
                        : (registers[register_source1] + registers[register_source2])
                    );
                    break;
                case 0b001: // sll
                    registers[register_destination] = registers[register_source1] << (registers[register_source2] & 0b11111);
                    break;
                case 0b010: // slt
                    registers[register_destination] = (registers[register_source1] < registers[register_source2]);
                    break;
                case 0b011: // sltu
                    registers[register_destination] = (registers_unsigned[register_source1] < registers_unsigned[register_source2]);
                    break;
                case 0b100: // xor
                    registers[register_destination] = registers[register_source1] ^ registers[register_source2];
                    break;
                case 0b101: { // srl/sra
                    uint32_t shift_by = registers[register_source2] & 0b11111;
                    registers[register_destination] = (
                        instruction >> 30
                        ? registers[register_source1] >> shift_by
                        : registers[register_source1] >> shift_by
                    );
                    break;
                }
                case 0b110: // or
                    registers[register_destination] = registers[register_source1] | registers[register_source2];
                    break;
                case 0b111: // and
                    registers[register_destination] = registers[register_source1] & registers[register_source2];
                    break;
            }
            break;

        case 0b01101: // lui
            registers[register_destination] = instruction & 0xfffff000;
            break;

        case 0b11000: { // branch
            uint32_t offset = (instruction >> 7) & 0xFFF;
            switch (opcode_2) {
                case 0b000: // beq
                    if (registers[register_source1] != registers[register_source2]) break;
                    program_counter += (offset << 1);
                    return;
                case 0b001: // bne
                    if (registers[register_source1] == registers[register_source2]) break;
                    program_counter += (offset << 1);
                    return;
                case 0b100: // blt
                    if (registers[register_source1] >= registers[register_source2]) break;
                    program_counter += (offset << 1);
                    return;
                case 0b101: // bge
                    if (registers[register_source1] < registers[register_source2]) break;
                    program_counter += (offset << 1);
                    return;
                case 0b110: // bltu
                    if (registers_unsigned[register_source1] >= registers_unsigned[register_source2]) break;
                    program_counter += (offset << 1);
                    return;
                case 0b111: // bgeu
                    if (registers_unsigned[register_source1] < registers_unsigned[register_source2]) break;
                    program_counter += (offset << 1);
                    return;
                default:
                    fprintf(stderr, "Error: invalid branch condition\n");
                    exit(1);
            }
            break;
        }

        case 0b11001: // jalr
            registers[register_destination] = (program_counter + 1) << 2;
            program_counter = (registers[register_source1] + (instruction >> 20)) & ~0x3;
            return;

        case 0b11011: // jal
            registers[register_destination] = (program_counter + 1) << 2;
            program_counter = instruction >> 12;
            return;

        case 0b11100: // advanced stuff 
            fprintf(stderr, "Error: Shit is not implemented! GO BACK TO X86 NOWWWWW!\n");
            exit(1);

        default:
            fprintf(stderr, "Unknown opcode %d\n", opcode_1);
            exit(1);
    }

    
    program_counter = (program_counter + 1) & ~0x3;
}

int main() {
    
    memset(memory8, 0, MEMORY_SIZE);
    memset(registers, 0, sizeof(registers));
    memset(memory32, 0, sizeof(memory32));

    uint32_t test_instruction = 0x00050593;
    memory32[0] = test_instruction;

    // LET IT TICK! ITS NOT COOKING ITS TICKING WAHAHAHAHAHA
    uint32_t instruction_count = 11111;
    while (1) {
        
        if (instruction_count++ > 10000000000) {
            printf("Instruction limit reached. Exiting program.\n");
            break;
        }

        printf("Executing instruction %u\n", instruction_count);
        tick();
    }

    printf("\nFinal register state:\n");
    for (int i = 0; i < NUM_REGISTERS; i++) {
        printf("x%d = %d\n", i, registers[i]);
    }

    return 0;
}


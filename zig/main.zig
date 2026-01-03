const std = @import("std");

const MEMORY_SIZE = 64 * 1024;

// Out of bounds bit masks for memory access validation
// Any address with these bits set is out of bounds
const OOB_BITS_8: i32 = ~@as(i32, MEMORY_SIZE - 1); // byte access
const OOB_BITS_16: i32 = ~@as(i32, MEMORY_SIZE - 1 - 1); // halfword access
const OOB_BITS_32: i32 = ~@as(i32, MEMORY_SIZE - 1 - 3); // word access
const OOB_BITS_PC: u32 = ~@as(u32, MEMORY_SIZE / 4 - 1); // program counter (word index)

var registers: [32]i32 = undefined;
var registers_unsigned: [*]u32 = undefined;
var memory8: [MEMORY_SIZE]u8 = undefined;
var memory16: [*]u16 = undefined;
var memory32: [*]i32 = undefined;

// index for 32 bit!
var program_counter: u32 = 0;

fn tick() !void {
    // make it constant
    registers[0] = 0;

    const instruction = memory32[program_counter];

    const funct3: u32 = @intCast((instruction >> 12) & 0b111);

    const register_destination: usize = @intCast((instruction >> 7) & 0b11111);
    const register_source1: usize = @intCast((instruction >> 15) & 0b11111);
    const register_source2: usize = @intCast((instruction >> 20) & 0b11111);

    // Note: JavaScript's >> operator is arithmetic right shift (sign-preserving).
    // Using (instruction >> 20) correctly sign-extends 12-bit immediates.
    // The | 0 suffix converts to 32-bit signed integer.
    opcode: switch (((instruction >> 2) << 3) & 0xff | funct3) {
        // load
        0b00000000 => { // lb
            const addr: i32 = registers[register_source1] +% (@as(i32, @bitCast(instruction)) >> 20);
            if (addr & OOB_BITS_8 != 0) return error.OutOfBounds;
            registers[register_destination] = @as(i8, @bitCast(memory8[@intCast(addr)]));
        },
        0b00000001 => { // lh
            const addr: i32 = registers[register_source1] +% (@as(i32, @bitCast(instruction)) >> 20);
            if (addr & OOB_BITS_16 != 0) return error.OutOfBounds;
            registers[register_destination] = @as(i16, @bitCast(memory16[@intCast(@as(u32, @bitCast(addr)) >> 1)]));
        },
        0b00000010 => { // lw
            const addr: i32 = registers[register_source1] +% (@as(i32, @bitCast(instruction)) >> 20);
            if (addr & OOB_BITS_32 != 0) return error.OutOfBounds;
            registers[register_destination] = memory32[@intCast(@as(u32, @bitCast(addr)) >> 2)];
        },
        0b00000100 => { // lbu
            const addr: i32 = registers[register_source1] +% (@as(i32, @bitCast(instruction)) >> 20);
            if (addr & OOB_BITS_8 != 0) return error.OutOfBounds;
            registers[register_destination] = memory8[@intCast(addr)];
        },
        0b00000101 => { // lhu
            const addr: i32 = registers[register_source1] +% (@as(i32, @bitCast(instruction)) >> 20);
            if (addr & OOB_BITS_16 != 0) return error.OutOfBounds;
            registers[register_destination] = memory16[@intCast(@as(u32, @bitCast(addr)) >> 1)];
        },
        // fence
        // register+immediate
        0b00100000 => { // addi
            registers[register_destination] = registers[register_source1] +% (@as(i32, @bitCast(instruction)) >> 20);
        },
        0b00100001 => { // slli
            registers[register_destination] = registers[register_source1] << @intCast((instruction >> 20) & 0b11111);
        },
        0b00100010 => { // slti
            registers[register_destination] = if (registers[register_source1] < (@as(i32, @bitCast(instruction)) >> 20)) 1 else 0;
        },
        0b00100011 => { // sltiu
            registers[register_destination] = if (registers_unsigned[register_source1] < (instruction >> 20)) 1 else 0;
        },
        0b00100100 => { // xori
            registers[register_destination] = registers[register_source1] ^ (@as(i32, @bitCast(instruction)) >> 20);
        },
        0b00100101 => { // srli/srai
            const shift_by: u5 = @intCast((instruction >> 20) & 0b11111);
            if (instruction >> 30 != 0) {
                registers[register_destination] = registers[register_source1] >> shift_by;
            } else {
                registers_unsigned[register_destination] = registers_unsigned[register_source1] >> shift_by;
            }
        },
        0b00100110 => { // ori
            registers[register_destination] = registers[register_source1] | (@as(i32, @bitCast(instruction)) >> 20);
        },
        0b00100111 => { // andi
            registers[register_destination] = registers[register_source1] & (@as(i32, @bitCast(instruction)) >> 20);
        },
        0b00101000, 0b00101001, 0b00101010, 0b00101011, 0b00101100, 0b00101101, 0b00101110, 0b00101111 => { // auipc
            registers[register_destination] = @as(i32, @bitCast((program_counter << 2) +% (instruction & 0xfffff000)));
        },
        // store
        0b01000000 => { // sb
            const addr: i32 = registers[register_source1] +% ((@as(i32, @bitCast(instruction)) >> 25) << 5 | @as(i32, @intCast(register_destination)));
            if (addr & OOB_BITS_8 != 0) return error.OutOfBounds;
            memory8[@intCast(addr)] = @intCast(registers[register_source2]);
        },
        0b01000001 => { // sh
            const addr: i32 = registers[register_source1] +% ((@as(i32, @bitCast(instruction)) >> 25) << 5 | @as(i32, @intCast(register_destination)));
            if (addr & OOB_BITS_16 != 0) return error.OutOfBounds;
            memory16[@intCast(@as(u32, @bitCast(addr)) >> 1)] = @intCast(registers[register_source2]);
        },
        0b01000010 => { // sw
            const addr: i32 = registers[register_source1] +% ((@as(i32, @bitCast(instruction)) >> 25) << 5 | @as(i32, @intCast(register_destination)));
            if (addr & OOB_BITS_32 != 0) return error.OutOfBounds;
            memory32[@intCast(@as(u32, @bitCast(addr)) >> 2)] = registers[register_source2];
        },
        // register+register
        0b01100000 => { // add/sub
            registers[register_destination] = if (instruction >> 30 != 0)
                registers[register_source1] -% registers[register_source2]
            else
                registers[register_source1] +% registers[register_source2];
        },
        0b01100001 => { // sll
            registers[register_destination] = registers[register_source1] << @intCast(registers[register_source2] & 0b11111);
        },
        0b01100010 => { // slt
            registers[register_destination] = if (registers[register_source1] < registers[register_source2]) 1 else 0;
        },
        0b01100011 => { // sltu
            registers[register_destination] = if (registers_unsigned[register_source1] < registers_unsigned[register_source2]) 1 else 0;
        },
        0b01100100 => { // xor
            registers[register_destination] = registers[register_source1] ^ registers[register_source2];
        },
        0b01100101 => { // srl/sra
            const shift_by: u5 = @intCast(registers[register_source2] & 0b11111);
            if (instruction >> 30 != 0) {
                registers[register_destination] = registers[register_source1] >> shift_by;
            } else {
                registers_unsigned[register_destination] = registers_unsigned[register_source1] >> shift_by;
            }
        },
        0b01100110 => { // or
            registers[register_destination] = registers[register_source1] | registers[register_source2];
        },
        0b01100111 => { // and
            registers[register_destination] = registers[register_source1] & registers[register_source2];
        },
        0b01101000, 0b01101001, 0b01101010, 0b01101011, 0b01101100, 0b01101101, 0b01101110, 0b01101111 => { // lui
            registers[register_destination] = @bitCast(instruction & 0xfffff000);
        },
        0b11000000, 0b11000001, 0b11000100, 0b11000101, 0b11000110, 0b11000111 => { // branch
            switch (funct3) {
                0b000 => { // beq
                    if (registers[register_source1] == registers[register_source2]) {} else break :opcode;
                },
                0b001 => { // bne
                    if (registers[register_source1] != registers[register_source2]) {} else break :opcode;
                },
                0b100 => { // blt
                    if (registers[register_source1] < registers[register_source2]) {} else break :opcode;
                },
                0b101 => { // bge
                    if (registers[register_source1] >= registers[register_source2]) {} else break :opcode;
                },
                0b110 => { // bltu
                    if (registers_unsigned[register_source1] < registers_unsigned[register_source2]) {} else break :opcode;
                },
                0b111 => { // bgeu
                    if (registers_unsigned[register_source1] >= registers_unsigned[register_source2]) {} else break :opcode;
                },
                else => return error.InvalidBranchCondition,
            }
            program_counter = program_counter +% @as(u32, @bitCast( // 12 bit offset, shifted one to the right
                ((@as(i32, @bitCast(instruction)) >> 31) << 10) | // 31 -> 10
                ((@as(i32, @intCast(register_destination)) & 0x1) << 9) | // dest -> 9
                ((@as(i32, @intCast(instruction >> 25)) << 3)) | // 30-25 -> 8-3
                @as(i32, @intCast(register_destination >> 2)) // dest -> 2-0
            ));
            if (program_counter & OOB_BITS_PC != 0) return error.OutOfBounds;
            return;
        },
        0b11001000 => { // jalr
            registers[register_destination] = @bitCast((program_counter + 1) << 2);
            program_counter = @intCast((registers[register_source1] +% (@as(i32, @bitCast(instruction)) >> 20)) >> 2);
            if (program_counter & OOB_BITS_PC != 0) return error.OutOfBounds;
            return;
        },
        0b11011000, 0b11011001, 0b11011010, 0b11011011, 0b11011100, 0b11011101, 0b11011110, 0b11011111 => { // jal
            // exit on endless loop
            if (instruction >> 12 == 0) return error.ProgramEnded;
            registers[register_destination] = @bitCast((program_counter + 1) << 2);
            program_counter = program_counter +% @as(u32, @bitCast( // 20 bit offset, shifted one to the right
                ((@as(i32, @bitCast(instruction)) >> 31) << 18) | // 31 -> 19
                ((@as(i32, @intCast((instruction >> 12) & 0xff)) << 10)) | // 19-12 -> 18-11
                ((@as(i32, @intCast((instruction >> 20) & 0x1)) << 9)) | // 20 -> 10
                @as(i32, @intCast((instruction >> 22) & 0x3ff)) // 30-21 -> 9-0
            ));
            if (program_counter & OOB_BITS_PC != 0) return error.OutOfBounds;
            return;
        },
        else => return error.IllegalInstruction,
    }

    program_counter = program_counter +% 1;
    if (program_counter & OOB_BITS_PC != 0) return error.OutOfBounds;
}

pub fn main() !void {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    const args = try std.process.argsAlloc(allocator);
    defer std.process.argsFree(allocator, args);

    const program_path = if (args.len > 1) args[1] else "../tests/count.bin";

    const stdout = std.io.getStdOut().writer();
    try stdout.print("loading {s}\n", .{program_path});

    // load program
    const file = try std.fs.cwd().openFile(program_path, .{});
    defer file.close();

    const bytes_read = try file.readAll(&memory8);
    _ = bytes_read;

    // Initialize pointers to memory views
    registers_unsigned = @ptrCast(&registers);
    memory16 = @ptrCast(&memory8);
    memory32 = @ptrCast(&memory8);

    // Initialize registers to zero
    for (&registers) |*reg| {
        reg.* = 0;
    }

    try stdout.print("running\n", .{});

    const time_start = std.time.milliTimestamp();
    var instruction_count: u32 = 0;

    while (instruction_count < 10_000_000) {
        tick() catch |err| {
            if (err == error.ProgramEnded) {
                try stdout.print("-----\nprogram ended\n", .{});
                break;
            } else {
                const error_msg = switch (err) {
                    error.OutOfBounds => "out of bounds",
                    error.IllegalInstruction => "illegal instruction",
                    error.InvalidBranchCondition => "invalid branch condition",
                    else => "unknown error",
                };
                try stdout.print("-----\nprogram failed: {s}\n", .{error_msg});
                break;
            }
        };
        instruction_count += 1;
    } else {
        try stdout.print("-----\nprogram timed out\n", .{});
    }

    const runtime = std.time.milliTimestamp() - time_start;

    try stdout.print("ran {} instructions in {} ms\n", .{ instruction_count, runtime });
    try stdout.print("execution speed: {} MHz\n", .{@divTrunc(instruction_count, @as(u32, @intCast(runtime))) / 1000});

    try stdout.print("registers:\n", .{});
    for (1..32) |i| {
        try stdout.print("  x{d:<2} = 0x{x:0>8} {d}\n", .{ i, @as(u32, @bitCast(registers[i])), registers[i] });
    }
}

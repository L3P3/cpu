# Expected result: x10 = 2 (both jumps executed correctly)
_start:
	addi x10, zero, 0

	# Test jal (jump and link)
	jal x1, jal_target
	addi x10, x10, 100      # should not reach here

jal_target:
	addi x10, x10, 1

	# Test jalr (jump and link register)
	# Calculate address of jalr_target relative to current PC
	auipc x5, 0             # x5 = current PC
	addi x5, x5, 16         # x5 = PC + 16 (4 instructions ahead)
	jalr x1, x5, 0          # jump to address in x5
	addi x10, x10, 100      # should not reach here

jalr_target:
	addi x10, x10, 1

end:
	jal zero, end           # infinite loop

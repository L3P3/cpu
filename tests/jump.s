# Expected result: x10 = 3 (all jumps executed correctly)
_start:
	li x10, 0
	
	# Test jal (jump and link)
	jal x1, jal_target
	j end                   # should not reach here
	
jal_target:
	addi x10, x10, 1
	
	# Test jalr (jump and link register)
	# Calculate address of jalr_target using auipc + addi
	auipc x5, 0
	addi x5, x5, 16         # jalr_target is 16 bytes ahead
	jalr x1, x5, 0
	j end                   # should not reach here
	
jalr_target:
	addi x10, x10, 1
	
	# Test forward jump
	jal x1, forward_target
	j end                   # should not reach here
	
forward_target:
	addi x10, x10, 1
	
end:
	j end

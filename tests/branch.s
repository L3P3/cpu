# Expected result: x10 = 6 (all branches taken correctly)
_start:
	li x10, 0 # counter for successful branches

	# Test beq (branch if equal)
	li x5, 10
	li x6, 10
	beq x5, x6, beq_pass
	j end
beq_pass:
	addi x10, x10, 1

	# Test bne (branch if not equal)
	li x5, 10
	li x6, 20
	bne x5, x6, bne_pass
	j end
bne_pass:
	addi x10, x10, 1

	# Test blt (branch if less than, signed)
	li x5, -5
	li x6, 10
	blt x5, x6, blt_pass
	j end
blt_pass:
	addi x10, x10, 1

	# Test bge (branch if greater or equal, signed)
	li x5, 10
	li x6, -5
	bge x5, x6, bge_pass
	j end
bge_pass:
	addi x10, x10, 1

	# Test bltu (branch if less than, unsigned)
	li x5, 10
	li x6, 20
	bltu x5, x6, bltu_pass
	j end
bltu_pass:
	addi x10, x10, 1

	# Test bgeu (branch if greater or equal, unsigned)
	li x5, 20
	li x6, 10
	bgeu x5, x6, bgeu_pass
	j end
bgeu_pass:
	addi x10, x10, 1

end:
	j end

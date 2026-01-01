# Expected result: x10 = 1, x11 = 0, x12 = 1, x13 = 0, x14 = 1, x15 = 0, x16 = 1, x17 = 0
_start:
	# Test slti (set less than immediate, signed)
	li x10, 5
	slti x10, x10, 10       # x10 = (5 < 10) = 1
	
	li x11, 10
	slti x11, x11, 5        # x11 = (10 < 5) = 0
	
	# Test sltiu (set less than immediate, unsigned)
	li x12, 5
	sltiu x12, x12, 10      # x12 = (5 < 10) = 1
	
	li x13, 10
	sltiu x13, x13, 5       # x13 = (10 < 5) = 0
	
	# Test slt (set less than, signed)
	li x14, 5
	li x7, 10
	slt x14, x14, x7        # x14 = (5 < 10) = 1
	
	li x15, 10
	li x7, 5
	slt x15, x15, x7        # x15 = (10 < 5) = 0
	
	# Test sltu (set less than, unsigned)
	li x16, 5
	li x7, 10
	sltu x16, x16, x7       # x16 = (5 < 10) = 1
	
	li x17, 10
	li x7, 5
	sltu x17, x17, x7       # x17 = (10 < 5) = 0
	
end:
	j end

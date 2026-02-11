# Test DIV and DIVU instructions
# Expected results:
#   x10 = 5 (15 / 3)
#   x11 = -5 (-15 / 3)
#   x12 = -5 (15 / -3)
#   x13 = -1 (15 / 0, special case)
#   x14 = 1431655765 (0xaaaaaaaa / 2, unsigned)
_start:
	# Test 1: positive / positive
	li x5, 15
	li x6, 3
	div x10, x5, x6         # x10 = 15 / 3 = 5

	# Test 2: negative / positive
	li x7, -15
	div x11, x7, x6         # x11 = -15 / 3 = -5

	# Test 3: positive / negative
	li x8, -3
	div x12, x5, x8         # x12 = 15 / -3 = -5

	# Test 4: division by zero (should return -1)
	li x9, 0
	div x13, x5, x9         # x13 = 15 / 0 = -1

	# Test 5: unsigned division
	li x15, 0xaaaaaaaa
	li x16, 2
	divu x14, x15, x16      # x14 = 0xaaaaaaaa / 2 (unsigned)

end:
	j end

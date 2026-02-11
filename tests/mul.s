# Test MUL instruction
# Expected results:
#   x10 = 42 (6 * 7)
#   x11 = -30 (6 * -5)
#   x12 = 0 (0 * 100)
#   x13 = 100 (10 * 10)
_start:
	# Test 1: positive * positive
	li x5, 6
	li x6, 7
	mul x10, x5, x6         # x10 = 6 * 7 = 42

	# Test 2: positive * negative
	li x7, -5
	mul x11, x5, x7         # x11 = 6 * -5 = -30

	# Test 3: multiply by zero
	li x8, 0
	li x9, 100
	mul x12, x8, x9         # x12 = 0 * 100 = 0

	# Test 4: square
	li x14, 10
	mul x13, x14, x14       # x13 = 10 * 10 = 100

end:
	j end

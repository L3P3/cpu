# Test REM and REMU instructions
# Expected results:
# x10 = 1 (16 % 3)
# x11 = -1 (-16 % 3)
# x12 = 1 (16 % -3)
# x13 = 16 (16 % 0, special case)
# x14 = 0 (100 % 10)
_start:
	# Test 1: positive % positive
	li x5, 16
	li x6, 3
	rem x10, x5, x6 # x10 = 16 % 3 = 1

	# Test 2: negative % positive
	li x7, -16
	rem x11, x7, x6 # x11 = -16 % 3 = -1

	# Test 3: positive % negative
	li x8, -3
	rem x12, x5, x8 # x12 = 16 % -3 = 1

	# Test 4: remainder by zero (should return dividend)
	li x9, 0
	rem x13, x5, x9 # x13 = 16 % 0 = 16

	# Test 5: remainder zero
	li x14, 100
	li x15, 10
	rem x14, x14, x15 # x14 = 100 % 10 = 0

end:
	j end

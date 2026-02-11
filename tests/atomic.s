# Test A extension (atomic operations)
# Expected results (all AMO operations return the OLD value in rd):
#   x10 = 100 (original value from LR)
#   x11 = 0 (SC success)
#   x12 = 10 (AMOADD old value)
#   x13 = 15 (AMOSWAP old value)
#   x14 = 20 (AMOXOR old value)
#   x15 = 31 (AMOAND old value)
#   x16 = 5 (AMOOR old value)
#   x17 = 10 (AMOMIN old value)
#   x18 = 5 (AMOMAX old value)
#   x19 = 10 (AMOMINU old value)
#   x20 = 5 (AMOMAXU old value)

_start:
	# Setup: store test value at memory address 0x100
	li x1, 0x100
	li x2, 100
	sw x2, 0(x1)

	# Test 1: LR.W - Load Reserved
	lr.w x10, (x1)          # x10 = 100 (value at 0x100)

	# Test 2: SC.W - Store Conditional (should succeed)
	li x3, 200
	sc.w x11, x3, (x1)      # x11 = 0 (success), memory[0x100] = 200

	# Test 3: AMOADD.W - Atomic Add
	li x4, 10
	sw x4, 0(x1)            # reset memory[0x100] = 10
	li x5, 5
	amoadd.w x12, x5, (x1)  # x12 = 10 (old value), memory[0x100] = 15

	# Test 4: AMOSWAP.W - Atomic Swap
	li x6, 20
	amoswap.w x13, x6, (x1) # x13 = 15 (old value), memory[0x100] = 20

	# Test 5: AMOXOR.W - Atomic XOR
	li x7, 15
	amoxor.w x14, x7, (x1)  # x14 = 20 (old value), memory[0x100] = 20 XOR 15 = 27

	# Test 6: AMOAND.W - Atomic AND
	li x8, 31
	sw x8, 0(x1)            # reset memory[0x100] = 31
	li x9, 7
	amoand.w x15, x9, (x1)  # x15 = 31 (old value), memory[0x100] = 31 AND 7 = 7

	# Test 7: AMOOR.W - Atomic OR
	li x28, 5
	sw x28, 0(x1)           # reset memory[0x100] = 5
	li x29, 26
	amoor.w x16, x29, (x1)  # x16 = 5 (old value), memory[0x100] = 5 OR 26 = 31

	# Test 8: AMOMIN.W - Atomic Min (signed)
	li x28, 10
	sw x28, 0(x1)           # reset memory[0x100] = 10
	li x29, 5
	amomin.w x17, x29, (x1) # x17 = 10 (old value), memory[0x100] = min(10,5) = 5

	# Test 9: AMOMAX.W - Atomic Max (signed)
	li x28, 100
	amomax.w x18, x28, (x1) # x18 = 5 (old value), memory[0x100] = max(5,100) = 100

	# Test 10: AMOMINU.W - Atomic Min (unsigned)
	li x28, 10
	sw x28, 0(x1)           # reset memory[0x100] = 10
	li x29, 5
	amominu.w x19, x29, (x1) # x19 = 10 (old value), memory[0x100] = min(10,5) = 5

	# Test 11: AMOMAXU.W - Atomic Max (unsigned)
	li x28, -6              # 0xFFFFFFFA as unsigned is very large
	amomaxu.w x20, x28, (x1) # x20 = 5 (old value), memory[0x100] = max(5,0xFFFFFFFA) = 0xFFFFFFFA

end:
	j end

# Expected result: x10 = 0x80, x11 = 0x08, x12 = 0x01, x13 = 0x100, x14 = 0x01, x15 = 0xffffffff
_start:
	# Test slli (shift left logical immediate)
	li x10, 0x08
	slli x10, x10, 4        # x10 = 0x08 << 4 = 0x80
	
	# Test srli (shift right logical immediate)
	li x11, 0x80
	srli x11, x11, 4        # x11 = 0x80 >> 4 = 0x08
	
	# Test srli with larger value
	li x12, 0x10
	srli x12, x12, 4        # x12 = 0x10 >> 4 = 0x01
	
	# Test sll (shift left logical register)
	li x13, 0x10
	li x7, 4
	sll x13, x13, x7        # x13 = 0x10 << 4 = 0x100
	
	# Test srl (shift right logical register)
	li x14, 0x10
	li x7, 4
	srl x14, x14, x7        # x14 = 0x10 >> 4 = 0x01
	
	# Test sra (shift right arithmetic register)
	li x15, -2              # 0xfffffffe
	li x7, 1
	sra x15, x15, x7        # x15 = -2 >> 1 = -1 (0xffffffff)
	
end:
	j end

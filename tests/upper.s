# Expected result: x10 = 0x12345000, x11 points to PC + 0x12345000 offset
_start:
	# Test lui (load upper immediate)
	lui x10, 0x12345        # x10 = 0x12345000

	# Test auipc (add upper immediate to PC)
	auipc x11, 0x12345      # x11 = PC + 0x12345000

end:
	j end

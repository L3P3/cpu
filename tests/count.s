_start:
#	li t0, 0
	li t1, 0xffff

loop:
	sb t0, 0x20(t0)
	addi t0, t0, 1
	bne t0, t1, loop

end:
	j end

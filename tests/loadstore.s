_start:
	li t0, 0xDEADBEEF
	li t1, 0x40
loop:
	sb t0, 0(t1)
	sh t0, 0(t1)
	sw t0, 0(t1)
	lb t2, 0x40(zero)
	lh t2, 0x40(zero)
	lw t2, 0x40(zero)
	j loop

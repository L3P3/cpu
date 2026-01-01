# Expected result: x5 = 42 (20 + 22), x6 = 0
_start:
	li x5, 20
	li x6, 22
	add x5, x5, x6
	li x6, 0
end:
	j end

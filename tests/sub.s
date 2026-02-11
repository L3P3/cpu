# Expected result: x10 = 5 (15 - 10)
_start:
	li x10, 15
	li x11, 10
	sub x10, x10, x11       # x10 = 15 - 10 = 5

end:
	j end

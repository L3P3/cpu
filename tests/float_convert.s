# Test conversion and comparison instructions
# Test FCVT, FEQ, FLT, FLE

.text
.globl _start

_start:
	# Test integer to float conversion
	addi x1, x0, 42     # x1 = 42
	fcvt.s.w f0, x1     # f0 = 42.0 (float)
	
	# Test float to integer conversion
	fcvt.w.s x2, f0     # x2 = 42
	
	# Test comparison
	lui x3, 0x42280     # x3 = 0x42280000 (42.0 in float)
	sw x3, 0(x0)
	flw f1, 0(x0)       # f1 = 42.0
	
	# Test FEQ: f0 == f1 (should be 1)
	feq.s x4, f0, f1
	
	# Test with different value
	lui x5, 0x41200     # x5 = 0x41200000 (10.0 in float)
	sw x5, 4(x0)
	flw f2, 4(x0)       # f2 = 10.0
	
	# Test FLT: f2 < f0 (should be 1, 10 < 42)
	flt.s x6, f2, f0
	
	# Test FLE: f2 <= f0 (should be 1, 10 <= 42)
	fle.s x7, f2, f0
	
	# Exit
	jal x0, .

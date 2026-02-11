# Basic double-precision floating-point tests (D extension)
# Test FLD, FSD, FADD.D, FSUB.D, FMUL.D, FDIV.D

.text
.globl _start

_start:
	# Initialize some double-precision values in memory
	# 2.0 in double: 0x4000000000000000
	lui x1, 0x40000
	sw x1, 4(x0)        # store high word at address 4
	sw x0, 0(x0)        # store low word at address 0
	
	# 3.0 in double: 0x4008000000000000
	lui x2, 0x40080
	sw x2, 12(x0)       # store high word at address 12
	sw x0, 8(x0)        # store low word at address 8
	
	# Load double-precision values
	fld f0, 0(x0)       # f0 = 2.0
	fld f1, 8(x0)       # f1 = 3.0
	
	# Test FADD.D: 2.0 + 3.0 = 5.0
	fadd.d f2, f0, f1
	
	# Test FSUB.D: 3.0 - 2.0 = 1.0
	fsub.d f3, f1, f0
	
	# Test FMUL.D: 2.0 * 3.0 = 6.0
	fmul.d f4, f0, f1
	
	# Test FDIV.D: 6.0 / 2.0 = 3.0
	fdiv.d f5, f4, f0
	
	# Store results back to memory
	fsd f2, 16(x0)      # store 5.0 at address 16
	fsd f3, 24(x0)      # store 1.0 at address 24
	fsd f4, 32(x0)      # store 6.0 at address 32
	fsd f5, 40(x0)      # store 3.0 at address 40
	
	# Load results into integer registers for verification
	lw x3, 20(x0)       # x3 should be 0x40140000 (high word of 5.0)
	lw x4, 28(x0)       # x4 should be 0x3ff00000 (high word of 1.0)
	lw x5, 36(x0)       # x5 should be 0x40180000 (high word of 6.0)
	lw x6, 44(x0)       # x6 should be 0x40080000 (high word of 3.0)
	
	# Exit
	jal x0, .

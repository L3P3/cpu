# Basic floating-point tests (F extension)
# Test FLW, FSW, FADD.S, FSUB.S, FMUL.S, FDIV.S

.text
.globl _start

_start:
	# Initialize some floating-point values in memory
	lui x1, 0x40000     # x1 = 0x40000000 (2.0 in float)
	sw x1, 0(x0)        # store at address 0
	
	lui x2, 0x40400     # x2 = 0x40400000 (3.0 in float)
	sw x2, 4(x0)        # store at address 4
	
	# Load floating-point values
	flw f0, 0(x0)       # f0 = 2.0
	flw f1, 4(x0)       # f1 = 3.0
	
	# Test FADD.S: 2.0 + 3.0 = 5.0
	fadd.s f2, f0, f1
	
	# Test FSUB.S: 3.0 - 2.0 = 1.0
	fsub.s f3, f1, f0
	
	# Test FMUL.S: 2.0 * 3.0 = 6.0
	fmul.s f4, f0, f1
	
	# Test FDIV.S: 6.0 / 2.0 = 3.0
	fdiv.s f5, f4, f0
	
	# Store results back to memory
	fsw f2, 8(x0)       # store 5.0 at address 8
	fsw f3, 12(x0)      # store 1.0 at address 12
	fsw f4, 16(x0)      # store 6.0 at address 16
	fsw f5, 20(x0)      # store 3.0 at address 20
	
	# Load results into integer registers for verification
	lw x3, 8(x0)        # x3 should be 0x40a00000 (5.0)
	lw x4, 12(x0)       # x4 should be 0x3f800000 (1.0)
	lw x5, 16(x0)       # x5 should be 0x40c00000 (6.0)
	lw x6, 20(x0)       # x6 should be 0x40400000 (3.0)
	
	# Exit
	jal x0, .

SOURCES := $(wildcard tests/*.s)
OBJECTS := $(SOURCES:.s=.o)
BINARIES := $(SOURCES:.s=.bin)

.PHONY: all tests c rust zig clean

all: tests c rust zig

tests: $(BINARIES)

c: c/main

rust:
	cd rust && cargo build

zig:
	cd zig && zig build

%.o: %.s
	riscv64-unknown-elf-as -march=rv32ima -o $@ $<

%.bin: %.o
	riscv64-unknown-elf-objcopy -O binary $< $@

c/main: c/main.c
	gcc -O2 -fno-strict-aliasing -o c/main c/main.c

clean:
	rm -f $(OBJECTS) $(BINARIES) c/main
	cd rust && cargo clean
	cd zig && rm -rf zig-cache zig-out

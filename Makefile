SOURCES := $(wildcard tests/*.s)
OBJECTS := $(SOURCES:.s=.o)
BINARIES := $(SOURCES:.s=.bin)

.PHONY: all tests c rust clean

all: tests c rust

tests: $(BINARIES)

c: c/main

rust:
	cd rust && cargo build

%.o: %.s
	riscv64-unknown-elf-as -march=rv32imafd -o $@ $<

%.bin: %.o
	riscv64-unknown-elf-objcopy -O binary $< $@

c/main: c/main.c
	gcc -O2 -fno-strict-aliasing -o c/main c/main.c -lm

clean:
	rm -f $(OBJECTS) $(BINARIES) c/main
	cd rust && cargo clean

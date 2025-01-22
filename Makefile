SOURCES := $(wildcard tests/*.s)
OBJECTS := $(SOURCES:.s=.o)
BINARIES := $(SOURCES:.s=.bin)

all: $(BINARIES)

%.o: %.s
	riscv64-unknown-elf-as -march=rv32i -o $@ $<

%.bin: %.o
	riscv64-unknown-elf-objcopy -O binary $< $@

clean:
	rm -f $(OBJECTS) $(BINARIES)

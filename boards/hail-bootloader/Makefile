# Makefile for building the tock kernel for the Hail platform

TARGET=thumbv7em-none-eabi
PLATFORM=hailbootloader

include ../Makefile.common

TOCKLOADER=tockloader

TOCKLOADER_JTAG_FLAGS = --jtag --board hail --arch cortex-m4 --jtag-device ATSAM4LC8C

.PHONY: program
program: target/$(TARGET)/release/$(PLATFORM).bin
	$(TOCKLOADER) $(TOCKLOADER_GENERAL_FLAGS) flash --address 0x10000 $<

# upload kernel over JTAG
.PHONY: flash
flash: target/$(TARGET)/release/$(PLATFORM).bin
	$(TOCKLOADER) flash --address 0x10000 $(TOCKLOADER_JTAG_FLAGS) $<

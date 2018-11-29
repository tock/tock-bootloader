# Makefile for building the bootloader for Imix

TARGET=thumbv7em-none-eabi
PLATFORM=imixbootloader

include ../Common.mk

TOCKLOADER = tockloader
TOCKLOADER_JTAG_FLAGS = --jtag --board imix --arch cortex-m4 --jtag-device ATSAM4LC8C

# upload kernel over JTAG
.PHONY: flash
flash: target/$(TARGET)/release/$(PLATFORM).bin
	$(TOCKLOADER) flash --address 0x0 $(TOCKLOADER_JTAG_FLAGS) $<

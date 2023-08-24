# Makefile for building the Tock bootloader for nRF52 platforms over UART.

TOCK_ARCH=cortex-m4
TARGET=thumbv7em-none-eabi
PLATFORM=wm1110_dev-bootloader

include ../Common.mk

TOCKLOADER=tockloader

OPENOCD=openocd
OPENOCD_OPTIONS=-f openocd.cfg

# Upload the kernel over JTAG
.PHONY: flash
flash: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).bin
	$(OPENOCD) $(OPENOCD_OPTIONS) -c "program $<; verify_image $<; reset; shutdown;"

.PHONY: flash
flash-debug: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/debug/$(PLATFORM).bin
	$(OPENOCD) $(OPENOCD_OPTIONS) -c "program $<; verify_image $<; reset; shutdown;"

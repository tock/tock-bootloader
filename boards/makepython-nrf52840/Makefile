# Makefile for building the Tock bootloader for nRF52 platforms using CDC-ACM
# over USB.

TOCK_ARCH=cortex-m4
TARGET=thumbv7em-none-eabi
PLATFORM=makepython-nrf52840-bootloader

include ../Common.mk

TOCKLOADER=tockloader

# Where in the flash to load the kernel with `tockloader`
KERNEL_ADDRESS=0x00000

ifdef PORT
  TOCKLOADER_GENERAL_FLAGS += --port $(PORT)
endif

# Upload the kernel over JTAG
.PHONY: flash
flash: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).bin
	$(TOCKLOADER) $(TOCKLOADER_GENERAL_FLAGS) flash --address $(KERNEL_ADDRESS) --board nrf52dk --jlink $<

# Upload the kernel over JTAG using OpenOCD
.PHONY: flash-openocd
flash-openocd: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).bin
	$(TOCKLOADER) $(TOCKLOADER_GENERAL_FLAGS) flash --address $(KERNEL_ADDRESS) --board nrf52dk --openocd $<

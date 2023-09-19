BBC:MicroBit v2 Tock Bootloader
===================

This is the implementation of the Tock bootloader for the BBC:MicroBit v2
board. The bootloader runs using the Debugger UART.

Compiling
---------

To compile the bootloader, simply run the `make` command.

```
make
```

Flashing
--------

OpenOCD is needed to flash the bootloader. Running `make flash` will compile it and flash it.

```
make flash
```

Entering
--------

Entering the bootloader is done by holding Button A during reset.

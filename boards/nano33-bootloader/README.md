Arduino Nano 33 BLE Tock Bootloader
===================

This is the implementation of the Tock bootloader for the Arduino Nano 33 BLE
board. The bootloader runs using the CDC-ACM over USB stack.

Compiling
---------

We actually need to compile the bootloader twice, at two different addresses.
The main bootloader will reside at address `0x00000000` in flash. That is the
default address specified in the `layout.ld` linker script. However, we also
need a temporary "helper" bootloader compiled for address `0x10000`. We will use
the helper bootloader to replace the stock Nano 33 bootloader with our own.

Here are the steps:

```
make
cp ../../target/thumbv7em-none-eabi/release/nano33-bootloader.bin ~/nano33-bootloader-0x00000.bin
```


Adafruit CLUE - nRF52840 Express with Bluetooth LE Tock Bootloader
===================

This is the implementation of the Tock bootloader for the Adafruit CLUE - nRF52840 Express with Bluetooth LE
board. The bootloader runs using the CDC-ACM over USB stack.

Compiling
---------

Here are the steps:

```
make
cp ../../target/thumbv7em-none-eabi/release/clue_nrf52840-bootloader.bin ./clue_nrf52840-bootloader.bin
```

Converting to UF2
-----------

Install [uf2conf](https://github.com/microsoft/uf2/blob/master/utils/uf2conv.py)

```
uf2conv clue_nrf52840-bootloader.bin -f 0xADA52840 --base 0x26000 --output clue_nrf52840-bootloader.uf2
```

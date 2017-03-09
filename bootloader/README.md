Tock Bootloader
===============

The Tock bootloader provides a utility for flashing applications onto
a board over USB. It is compatible with the
[tockloader](https://github.com/helena-project/tockloader) utility.

The Tock bootloader is based on the Berkeley SDB Storm bootloader.


Compiling
---------

```bash
make [hail|justjump]
```

and to flash:

```bash
make flash-bootloader
```

`justjump`
----------

In this mode, there is no ability to upload apps or talk to the board. On boot,
the bootloader just jumps directly to address 0x10000.

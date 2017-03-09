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



Protocol
--------

All messages are initiated by the client and responded to by the bootloader.

### Framing

#### Commands

```
                             0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
Message (arbitrary length)  | Escape Char   | Command       |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
```

- `Message`: The command packet as specified by the individual commands.
             Escaped by replacing all `0xFC` with two consecutive `0xFC`.
- `Escape Character`: `0xFC`.
- `Command`: The command byte.


#### Response

```
 0                   1
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
| Escape Char   | Response      | Message (arbitrary length)
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
```
- `Escape Character`: `0xFC`.
- `Response`: The response byte.
- `Message`: The response packet as specified by the individual commands.
             Escaped by replacing all `0xFC` with two consecutive `0xFC`.



### Commands

#### `PING`

Send a ping to the bootloader. If everything is working it will respond with a
pong.

##### Command
- `Command`: `0x01`.
- `Message`: `None`.

##### Response
- `Response`: `0x11`.
- `Message`: `None`.


#### `INFO`

Retrieve an information string from the bootloader.

##### Command
- `Command`: `0x03`.
- `Message`: `None`.

##### Response

```
 0                   1                   2                   3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
| Length        | String...
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
                     192 bytes                                  |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
```
- `Response`: `0x25`
- `Length`: Length of the information string.
- `String`: `Length` bytes of information string and 192-length zeros.


#### `RESET`

Reset the internal buffer pointers in the bootloader. This is typically
called before each command.

##### Command
- `Command`: `0x05`.
- `Message`: `None`.

##### Response
None.


#### `ERASE_PAGE`

Erase a page of internal flash.

##### Command
```
 0                   1                   2                   3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
| Address                                                       |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
```
- `Command`: `0x06`.
- `Address`: The address of the page to erase. Little endian.

##### Response
- `Response`: `0x15`.
- `Message`: `None`.



#### `WRITE_PAGE`

Write a page of internal flash.

##### Command
```
 0                   1                   2                   3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
| Address                                                       |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
| Data...
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
             (512 bytes)                                        |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
```
- `Command`: `0x07`.
- `Address`: The address of the page to write. Little endian.
- `Data`: 512 data bytes to write to the page.

##### Response
- `Response`: `0x15`.
- `Message`: `None`.


#### `READ_RANGE`

Read an arbitrary rage of internal flash.

##### Command
```
 0                   1                   2                   3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
| Address                                                       |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
| Length                        |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
```
- `Command`: `0x06`.
- `Address`: The address of the page to erase. Little endian.
- `Length`: The number of bytes to read.

##### Response
```
 0                   1                   2                   3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
| Data...
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
             (arbitrary length)                                 |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
```
- `Response`: `0x20`.
- `Data`: Bytes read back from flash.



#### `SET_ATTRIBUTE`

Set an attribute at a given index in the internal flash.

##### Command
```
 0                   1                   2                   3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
| Index         | Key
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+

+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
                | Length        | Value
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
             (arbitrary length)                                 |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
```
- `Command`: `0x13`.
- `Index`: The attribute index to set. 0-15.
- `Key`: Eight byte key, zero padded.
- `Length`: Length of the value. 1-55.
- `Value`: `Length` bytes of value to be stored in the attribute.

##### Response
- `Response`: `0x15`.
- `Message`: `None`.


#### `GET_ATTRIBUTE`

Get an attribute at a given index from the internal flash.

##### Command
```
 0
 0 1 2 3 4 5 6 7
+-+-+-+-+-+-+-+-+
| Index         |
+-+-+-+-+-+-+-+-+
```
- `Command`: `0x13`.
- `Index`: The attribute index to get. 0-15.

##### Response
```
 0                   1                   2                   3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
| Key
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
                                                                |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
| Length        | Value
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
             (55 bytes)                                         |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
```
- `Response`: `0x22`.
- `Key`: Eight byte key, zero padded.
- `Length`: Length of the value. 1-55.
- `Value`: 55 bytes of potential value.



#### `CRC_INTERNAL_FLASH`

Get the CRC of a range of internal flash.

##### Command
```
 0                   1                   2                   3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
| Address                                                       |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
| Length                                                        |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
```
- `Command`: `0x13`.
- `Address`: The address to begin the CRC at. Little endian.
- `Length`: The length of the range to calculate the CRC over.

##### Response
```
 0                   1                   2                   3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
| CRC                                                           |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
```
- `Response`: `0x23`.
- `CRC`: The calculated CRC.





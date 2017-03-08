/*
 * This file is part of StormLoader, the Storm Bootloader
 *
 * StormLoader is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * StormLoader is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with StormLoader.  If not, see <http://www.gnu.org/licenses/>.
 *
 * Copyright 2014, Michael Andersen <m.andersen@eecs.berkeley.edu>
 */

#ifndef BL_COMMANDS_H
#define	BL_COMMANDS_H

#include <stdint.h>

#define ESCAPE_CHAR     0xFC //This was chosen as it is infrequent in .bin files

/**
 * Send a PING to the bootloader. It will drop its hp buffer and send
 * back a PONG
 */
#define CMD_PING        0x01

/**
 * Get info about the bootloader. The result is one byte of length, plus
 * length bytes of string, followed by 192-length zeroes
 */
#define CMD_INFO        0x03

/**
 * Get the Unique ID. Result is 8 bytes of unique ID
 */
#define CMD_ID          0x04

/**
 * Reset all TX and RX buffers
 */
#define CMD_RESET       0x05

/**
 * Erase a page. The RX buffer should contain the address of the start
 * of the 512 byte page. Any non-page-aligned addresses will result in
 * RES_BADADDR. This command is not required before writing a page, it is
 * just an optimisation. It is particularly quick for already empty pages
 */
#define CMD_EPAGE       0x06

/**
 * Write a page in internal flash. The RX buffer should contain the 4 byte
 * address of the start of the page, followed by 512 bytes of page.
 */
#define CMD_WPAGE       0x07

/*
 * Erase a block of pages in ex flash. The RX buffer should contain the address
 * of the start of the block. Each block is 8 pages, so 2048 bytes
 */
#define CMD_XEBLOCK     0x08

/**
 * Write a page to ex flash. The RX buffer should contain the address
 * of the start of the 256 byte page, followed by 256 bytes of page.
 */
#define CMD_XWPAGE      0x09

/**
 * Get the length and CRC of the RX buffer. The response is
 * two bytes of little endian length, followed by 4 bytes of crc32
 */
#define CMD_CRCRX       0x10

/**
 * Read a range from internal flash. The RX buffer should contain a 4 byte address
 * followed by 2 bytes of length. The response will be length bytes long.
 */
#define CMD_RRANGE      0x11

/**
 * Read a range from external flash. The RX buffer should contain a 4 byte address
 * followed by 2 bytes of length. The response will be length bytes long.
 */
#define CMD_XRRANGE     0x12

/**
 * Write a payload attribute. The RX buffer should contain a one byte index,
 * 8 bytes of key (null padded), one byte of value length, and valuelength
 * value bytes. valuelength must be less than or equal to 55. The value
 * may contain nulls.
 * The attribute index must be less than 16
 */
#define CMD_SATTR       0x13

/**
 * Get a payload attribute. The RX buffer should contain a 1 byte index.
 * The result is 8 bytes of key, 1 byte of value length, and 55 bytes
 * of potential value. You must discard 55-valuelength bytes from the end
 * yourself
 */
#define CMD_GATTR       0x14

/**
 * Get the CRC of a range of internal flash. The RX buffer should contain
 * a four byte address and a four byte range. The result will be
 * a four byte crc32
 */
#define CMD_CRCIF       0x15

/**
 * Get the CRC of a range of external flash. The RX buffer should contain
 * a four byte address and a four byte range. The result will be
 * a four byte crc32
 */
#define CMD_CRCEF       0x16

/**
 * Erase a page in external flash. The RX buffer should contain a 4 byte
 * address pointing to the start of the 256 byte page.
 */
#define CMD_XEPAGE      0x17

/**
 * Initialise the external flash chip. This sets the page size
 * to 256b
 */
#define CMD_XFINIT      0x18

/**
 * Go into an infinite loop with the 32khz clock present on pin PA19 (GP6)
 * this is used for clock calibration
 */
#define CMD_CLKOUT      0x19

/**
 * Write the flash user pages (first 4 bytes is first page, second 4 bytes
 * is second page, little endian)
 */
#define CMD_WUSER       0x20

#define RES_OVERFLOW    0x10
#define RES_PONG        0x11
#define RES_BADADDR     0x12
#define RES_INTERROR    0x13
#define RES_BADARGS     0x14
#define RES_OK          0x15
#define RES_UNKNOWN     0x16
#define RES_XFTIMEOUT   0x17
#define RES_XFEPE       0x18
#define RES_CRCRX       0x19
#define RES_RRANGE      0x20
#define RES_XRRANGE     0x21
#define RES_GATTR       0x22
#define RES_CRCIF       0x23
#define RES_CRCXF       0x24
#define RES_INFO        0x25

#define ALLOWED_ATTRIBUTE_FLOOR   0xFC00
#define ALLOWED_ATTRIBUTE_CEILING 0x10000

#define ALLOWED_FLASH_FLOOR   65280
#define ALLOWED_FLASH_CEILING 524287

#define ALLOWED_XFLASH_FLOOR   524288
#define ALLOWED_XFLASH_CEILING 67108863

/* Staging RAM for normal TX
 */
#define TXBUFSZ 8192
extern uint8_t tx_stage_ram [TXBUFSZ];
extern uint16_t tx_ptr;
extern uint16_t tx_left;

/* Staging RAM for RX
 */
#define RXBUFSZ 8192
extern uint8_t rx_stage_ram [RXBUFSZ];
extern uint16_t rx_ptr;

void bl_init(void);
void bl_testloop(void);
void bl_txb_r(uint8_t b);
void bl_txb(uint8_t b);
void bl_rxb(uint8_t b);

void bl_c_ping(void);
void bl_c_info(void);
void bl_loop_poll(void);
void bl_cmd(uint8_t b);
void bl_c_reset(void);
void bl_c_id(void);
void bl_c_wpage(void);
void bl_c_epage(void);
void bl_c_crcrx(void);
void bl_c_sattr(void);
void bl_c_gattr(void);
void bl_c_rrange(void);
void bl_c_crcif(void);
void bl_c_unknown(void);
void bl_c_clkout(void);
void bl_c_wuser(void);

uint32_t crc32(uint32_t crc, const void *buf, uint32_t size);


#endif	/* BL_COMMANDS_H */

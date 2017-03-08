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

#include <usart.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

#include <flashcalw.h>

#include "info.h"

#include "bootloader.h"
#include <wdt_sam4l.h>
#include "ASF/common/services/clock/sam4l/sysclk.h"
#include "ASF/common/services/ioport/ioport.h"

uint8_t byte_escape;
uint8_t tx_stage_ram[TXBUFSZ];
uint16_t tx_ptr;
uint16_t tx_left;
uint8_t rx_stage_ram[RXBUFSZ];
uint16_t rx_ptr;

const sam_usart_opt_t bl_settings = {
     115200,
     US_MR_CHRL_8_BIT,
     US_MR_PAR_NO, //TODO change
     US_MR_NBSTOP_1_BIT,
     US_MR_CHMODE_NORMAL
};

void bl_init(void) {
    // Disable the HW bootloader
    struct wdt_dev_inst wdt_inst;
    struct wdt_config   wdt_cfg;
    wdt_get_config_defaults(&wdt_cfg);
    wdt_init(&wdt_inst, WDT, &wdt_cfg);
    wdt_disable(&wdt_inst);

    byte_escape = 0;

    tx_ptr = tx_left = 0;
    tx_ptr = tx_left = 0;
    rx_ptr = 0;

    // Enable BL USART
    ioport_set_pin_mode(PIN_PA12A_USART0_TXD, MUX_PA12A_USART0_TXD);
    ioport_disable_pin(PIN_PA12A_USART0_TXD);
    ioport_set_pin_mode(PIN_PA11A_USART0_RXD, MUX_PA11A_USART0_RXD);
    ioport_disable_pin(PIN_PA11A_USART0_RXD);
    sysclk_enable_peripheral_clock(USART0);
    usart_reset(USART0);
    usart_init_rs232(USART0, &bl_settings, sysclk_get_main_hz());
    usart_enable_tx(USART0);
    usart_enable_rx(USART0);
}

void bl_loop_poll(void) {
    if (usart_is_rx_ready(USART0)) {
        uint32_t ch;
        usart_getchar(USART0, &ch);
        if (rx_ptr >= RXBUFSZ) {
            tx_ptr = 0;
            tx_left = 1;
            tx_stage_ram[0] = RES_OVERFLOW;
        } else {
            bl_rxb(ch);
        }
    }
    if (usart_is_tx_ready(USART0)) {
        if (tx_left > 0) {
            bl_txb(tx_stage_ram[tx_ptr++]);
            tx_left--;
        }
    }
}

void bl_txb(uint8_t b) {
    usart_putchar(USART0, b);
}

void bl_rxb(uint8_t b) {
    if (byte_escape && b == ESCAPE_CHAR) {
        // To escape characters in a row. We actually wanted the literal
        // escape character.
        byte_escape = 0;
        rx_stage_ram[rx_ptr++] = b;
    } else if (byte_escape) {
        // A single escape character ends this message.
        // Process it.
        bl_cmd(b);
        byte_escape = 0;
    } else if (b == ESCAPE_CHAR) {
        // Need to see the next byte to figure out what to do.
        byte_escape = 1;
    } else {
        // Save this byte.
        rx_stage_ram[rx_ptr++] = b;
    }
}

void bl_cmd(uint8_t b) {
    switch(b) {
        case CMD_PING:
            bl_c_ping();
            break;
        case CMD_INFO:
            bl_c_info();
            break;
        case CMD_ID:
            bl_c_id();
            break;
        case CMD_RESET:
            bl_c_reset();
            break;
        case CMD_WPAGE:
            bl_c_wpage();
            break;
        case CMD_EPAGE:
            bl_c_epage();
            break;
        case CMD_CRCRX:
            bl_c_crcrx();
            break;
        case CMD_RRANGE:
            bl_c_rrange();
            break;
        case CMD_SATTR:
            bl_c_sattr();
            break;
        case CMD_GATTR:
            bl_c_gattr();
            break;
        case CMD_CRCIF:
            bl_c_crcif();
            break;
        case CMD_CLKOUT:
            bl_c_clkout();
            break;
        case CMD_WUSER:
            bl_c_wuser();
            break;
        // These all require external flash and are therefore unsupported.
        case CMD_XEBLOCK:
        case CMD_XWPAGE:
        case CMD_XRRANGE:
        case CMD_CRCEF:
        case CMD_XEPAGE:
        case CMD_XFINIT:
        default:
            bl_c_unknown();
            break;
    }
}

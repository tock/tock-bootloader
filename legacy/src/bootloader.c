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

#include "bootloader.h"
#include <wdt_sam4l.h>
#include "ASF/common/services/clock/sam4l/sysclk.h"
#include "ASF/common/services/ioport/ioport.h"

#include "bootloader_board.h"

uint8_t byte_escape;
uint8_t tx_stage_ram[TXBUFSZ];
uint16_t tx_ptr;
uint16_t tx_left;
uint8_t rx_stage_ram[RXBUFSZ];
uint16_t rx_ptr;

change_baud_state_e change_baud_state = CHANGE_BAUD_IDLE;
uint32_t new_baud_rate = 0;
uint32_t old_baud_rate = 0;

sam_usart_opt_t bl_settings = {
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
    ioport_set_pin_mode(BOOTLOADER_UART_TX_PIN, BOOTLOADER_UART_TX_MUX);
    ioport_disable_pin(BOOTLOADER_UART_TX_PIN);
    ioport_set_pin_mode(BOOTLOADER_UART_RX_PIN, BOOTLOADER_UART_RX_MUX);
    ioport_disable_pin(BOOTLOADER_UART_RX_PIN);
    sysclk_enable_peripheral_clock(BOOTLOADER_UART);
    usart_reset(BOOTLOADER_UART);
    usart_init_rs232(BOOTLOADER_UART, &bl_settings, sysclk_get_main_hz());
    usart_enable_tx(BOOTLOADER_UART);
    usart_enable_rx(BOOTLOADER_UART);
}

void bl_change_baud_rate(void) {
    // Save old baud rate in case we need to revert
    old_baud_rate = bl_settings.baudrate;

    // Set new baud rate
    bl_settings.baudrate = new_baud_rate;
    usart_reset(BOOTLOADER_UART);
    usart_init_rs232(BOOTLOADER_UART, &bl_settings, sysclk_get_main_hz());
    usart_enable_tx(BOOTLOADER_UART);
    usart_enable_rx(BOOTLOADER_UART);
}

uint8_t bl_verify_baud_rate(uint32_t baud_rate) {
    return baud_rate == bl_settings.baudrate;
}

void bl_reset_baud_rate(void) {
    bl_settings.baudrate = old_baud_rate;
    old_baud_rate = 0;
    usart_reset(BOOTLOADER_UART);
    usart_init_rs232(BOOTLOADER_UART, &bl_settings, sysclk_get_main_hz());
    usart_enable_tx(BOOTLOADER_UART);
    usart_enable_rx(BOOTLOADER_UART);
}

void bl_loop_poll(void) {
    if (usart_is_rx_ready(BOOTLOADER_UART)) {
        uint32_t ch;
        usart_getchar(BOOTLOADER_UART, &ch);
        if (rx_ptr >= RXBUFSZ) {
            tx_ptr = 0;
            tx_left = 1;
            tx_stage_ram[0] = RES_OVERFLOW;
        } else {
            bl_rxb(ch);
        }
    }
    if (usart_is_tx_ready(BOOTLOADER_UART)) {
        if (tx_left > 0) {
            bl_txb(tx_stage_ram[tx_ptr++]);
            tx_left--;
        } else if (change_baud_state == CHANGE_BAUD_CHANGING) {
            while (!usart_is_tx_empty(BOOTLOADER_UART));
            // Change baud rate here so that the response to the initial
            // change command goes out at the same rate.
            change_baud_state = CHANGE_BAUD_WAITING_CONFIRMATION;
            bl_change_baud_rate();
        } else if (change_baud_state == CHANGE_BAUD_RESETTING) {
            while (!usart_is_tx_empty(BOOTLOADER_UART));
            // Change baud rate here so that the failure response goes out
            // at the same baud rate.
            change_baud_state = CHANGE_BAUD_IDLE;
            bl_reset_baud_rate();
        }
    }
}

void bl_txb(uint8_t b) {
    usart_putchar(BOOTLOADER_UART, b);
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
    } else if (change_baud_state == CHANGE_BAUD_WAITING_CONFIRMATION && rx_ptr > 10) {
        // Something went wrong with changing the baud rate.
        // First clear the receive, then reset the change baud rate state
        // machine.
        while (usart_is_rx_ready(BOOTLOADER_UART)) {
            uint32_t ch;
            usart_getchar(BOOTLOADER_UART, &ch);
        }
        // Now generate error.
        bl_cmd(0);
    } else {
        // Save this byte.
        rx_stage_ram[rx_ptr++] = b;
    }
}

void bl_cmd(uint8_t b) {
    // Check to see if we are in the middle of changing the baud rate.
    // If we are, then the only valid command is another baud rate change
    // command to confirm the new baud rate. If anything else happens, then
    // something went wrong probably and we should go back to the old
    // baud rate.
    if (change_baud_state == CHANGE_BAUD_WAITING_CONFIRMATION &&
        b != CMD_CHANGE_BAUD) {
        change_baud_state = CHANGE_BAUD_RESETTING;

        // Set the return here.
        tx_ptr = 0;
        tx_left = 2;
        tx_stage_ram[0] = ESCAPE_CHAR;
        tx_stage_ram[1] = RES_CHANGE_BAUD_FAIL;
        return;
    }

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
        case CMD_WUSER:
            bl_c_wuser();
            break;
        case CMD_CHANGE_BAUD:
            bl_change_baud();
            break;
        // These all require external flash and are therefore unsupported.
        case CMD_XEBLOCK:
        case CMD_XWPAGE:
        case CMD_XRRANGE:
        case CMD_CRCEF:
        case CMD_XEPAGE:
        case CMD_XFINIT:
        // This we just don't need anymore.
        case CMD_CLKOUT:
        default:
            bl_c_unknown();
            break;
    }
}

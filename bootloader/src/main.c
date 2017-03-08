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

#include <ioport.h>
#include <asf.h>
#include <board.h>
#include <conf_board.h>
#include <wdt_sam4l.h>
#include <sysclk.h>
#include "bootloader.h"
#include "ASF/common/services/ioport/sam/ioport_gpio.h"
#include "ASF/common/services/ioport/ioport.h"


void board_init(void) {
    // Setup GPIO
    ioport_init();

    // This is probably not used by the bootloader.
    // But is here in case we ever want to support the CMD_CLKOUT command.
    ioport_set_pin_dir(PIN_PA10, IOPORT_DIR_OUTPUT);

    // Pin which is pulled low to enter bootloader mode.
    ioport_set_pin_dir(PIN_PA08, IOPORT_DIR_INPUT);
    ioport_set_pin_mode(PIN_PA08, IOPORT_MODE_PULLUP | IOPORT_MODE_GLITCH_FILTER);

    // Setup Clock
    bpm_set_clk32_source(BPM, BPM_CLK32_SOURCE_RC32K);
    sysclk_init();
}

extern void jump_into_user_code(void)  __attribute__((noreturn));

int main (void) {
    board_init();

    // Verify BL policy
    uint32_t active = 0;
    uint32_t inactive = 0;
    uint32_t samples = 10000;
    while (samples) {
        if (ioport_get_pin_level(PIN_PA08) == 0) {
            active++;
        } else {
            inactive++;
        }
        samples--;
    }

    if (active > inactive) {
        // Enter bootloader mode and wait for commands from sload
        bl_init();
        while (1) {
            bl_loop_poll();
        }
    } else {
        // Go to main application code
        jump_into_user_code();
    }
}

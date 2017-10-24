#include <stdint.h>

#include "bootloader_board.h"

/* Specify a section at the beginning of flash to reserve for state
 * about the board.
 *
 * The first page (512 bytes) is for flags. This starts with "TOCKBOOTLOADER"
 * which allows tools to detect that a bootloader is present on the board.
 *
 * The second and third pages (1024 bytes) are for the attributes that the
 * bootloader can access.
 */


__attribute__ ((section(".attributes")))
struct {
    char    flag_bootloader_exists[14];
    char    flag_version_string[8];
    uint8_t flags_reserved[490];
    char    attribute00[ATTRIBUTES_00_LEN];
    uint8_t attribute00_padding[64-ATTRIBUTES_00_LEN];
    char    attribute01[ATTRIBUTES_01_LEN];
    uint8_t attribute01_padding[64-ATTRIBUTES_01_LEN];
    char    attribute02[ATTRIBUTES_02_LEN];
    uint8_t attribute02_padding[64-ATTRIBUTES_02_LEN];
    uint8_t attributes[832];
} attributes = {
    .flag_bootloader_exists = {'T', 'O', 'C', 'K', 'B', 'O', 'O', 'T', 'L', 'O', 'A', 'D', 'E', 'R'},
    .flag_version_string    = {'0', '.', '6', '.', '0', '\0', '\0', '\0'},
    .flags_reserved         = {0x00},
    .attribute00            = ATTRIBUTES_00_DEF,
    .attribute00_padding    = {0x00},
    .attribute01            = ATTRIBUTES_01_DEF,
    .attribute01_padding    = {0x00},
    .attribute02            = ATTRIBUTES_02_DEF,
    .attribute02_padding    = {0x00},
    .attributes             = {0x00}
};

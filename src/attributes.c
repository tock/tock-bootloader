#include <stdint.h>

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
	char flag_bootloader_exists[14];
	char flag_version_string[8];
	uint8_t flags_reserved[490];
	uint8_t attributes[1024];
} attributes = {
	{'T', 'O', 'C', 'K', 'B', 'O', 'O', 'T', 'L', 'O', 'A', 'D', 'E', 'R'},
	{'0', '.', '5', '.', '0', '\0', '\0', '\0'},
    0x00,
    0x00
};

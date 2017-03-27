#pragma once

#if TOCK_BOARD_hail == 1
#define BOOTLOADER_SELECT_PIN PIN_PA08

#define BOOTLOADER_UART_TX_PIN PIN_PA12A_USART0_TXD
#define BOOTLOADER_UART_TX_MUX MUX_PA12A_USART0_TXD
#define BOOTLOADER_UART_RX_PIN PIN_PA11A_USART0_RXD
#define BOOTLOADER_UART_RX_MUX MUX_PA11A_USART0_RXD
#define BOOTLOADER_UART USART0

#define ATTRIBUTES_00_LEN 13
#define ATTRIBUTES_00_DEF {'b','o','a','r','d','\0','\0','\0',4,'h','a','i','l'}
#define ATTRIBUTES_01_LEN 18
#define ATTRIBUTES_01_DEF {'a','r','c','h','\0','\0','\0','\0',9,'c','o','r','t','e','x','-','m','4'}
#define ATTRIBUTES_02_LEN 19
#define ATTRIBUTES_02_DEF {'j','l','d','e','v','i','c','e',10,'A','T','S','A','M','4','L','C','8','C'}

#elif TOCK_BOARD_justjump == 1

// unused, but defined for compilation
#define BOOTLOADER_UART_TX_PIN PIN_PA12A_USART0_TXD
#define BOOTLOADER_UART_TX_MUX MUX_PA12A_USART0_TXD
#define BOOTLOADER_UART_RX_PIN PIN_PA11A_USART0_RXD
#define BOOTLOADER_UART_RX_MUX MUX_PA11A_USART0_RXD
#define BOOTLOADER_UART USART0

#define ATTRIBUTES_00_LEN 1
#define ATTRIBUTES_00_DEF {0x00}
#define ATTRIBUTES_01_LEN 1
#define ATTRIBUTES_01_DEF {0x00}
#define ATTRIBUTES_02_LEN 1
#define ATTRIBUTES_02_DEF {0x00}


#else
#error "No TOCK_BOARD defined!"
#endif


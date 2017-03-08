################################################################################
# Automatically-generated file. Do not edit!
################################################################################

# Add inputs and outputs from these tool invocations to the build variables
C_SRCS +=  \
src/ASF/common/services/clock/sam4l/dfll.c \
src/ASF/common/services/clock/sam4l/osc.c \
src/ASF/common/services/clock/sam4l/pll.c \
src/ASF/common/services/clock/sam4l/sysclk.c \
src/ASF/sam/drivers/bpm/bpm.c \
src/ASF/sam/drivers/flashcalw/flashcalw.c \
src/ASF/sam/drivers/gpio/gpio.c \
src/ASF/sam/drivers/usart/usart.c \
src/ASF/sam/drivers/wdt/wdt_sam4l.c \
src/ASF/common/utils/interrupt/interrupt_sam_nvic.c \
src/ASF/sam/utils/cmsis/sam4l/source/templates/exceptions.c \
src/ASF/sam/utils/cmsis/sam4l/source/templates/gcc/startup_sam4l.c \
src/ASF/sam/utils/cmsis/sam4l/source/templates/system_sam4l.c \
src/ASF/sam/utils/syscalls/gcc/syscalls.c \
src/main.c \
src/bootloader.c \
src/bl_commands.c \
src/crc32.c

OBJS +=  \
build/src/ASF/common/services/clock/sam4l/dfll.o \
build/src/ASF/common/services/clock/sam4l/osc.o \
build/src/ASF/common/services/clock/sam4l/pll.o \
build/src/ASF/common/services/clock/sam4l/sysclk.o \
build/src/ASF/sam/drivers/bpm/bpm.o \
build/src/ASF/sam/drivers/flashcalw/flashcalw.o \
build/src/ASF/sam/drivers/gpio/gpio.o \
build/src/ASF/sam/drivers/usart/usart.o \
build/src/ASF/sam/drivers/wdt/wdt_sam4l.o \
build/src/ASF/common/utils/interrupt/interrupt_sam_nvic.o \
build/src/ASF/sam/utils/cmsis/sam4l/source/templates/exceptions.o \
build/src/ASF/sam/utils/cmsis/sam4l/source/templates/gcc/startup_sam4l.o \
build/src/ASF/sam/utils/cmsis/sam4l/source/templates/system_sam4l.o \
build/src/ASF/sam/utils/syscalls/gcc/syscalls.o \
build/src/main.o \
build/src/bootloader.o \
build/src/bl_commands.o \
build/src/crc32.o \
build/src/jumpfunc.o

BINARY_NAME=hail_bootloader

OUTPUT_FILE_PATH += build/$(BINARY_NAME).bin
CCBIN=arm-none-eabi-gcc
OBJCOPY=arm-none-eabi-objcopy
OBJDUMP=arm-none-eabi-objdump
SIZE=arm-none-eabi-size
JLINK=JLinkExe
CINCLUDES=-I"src" -I"src/ASF/common/boards/user_board" -I"src/ASF/common/utils" -I"src/ASF/sam/utils"
CINCLUDES+=-I"src/ASF/sam/utils/cmsis/sam4l/include" -I"src/ASF/sam/utils/cmsis/sam4l/source/templates"
CINCLUDES+=-I"src/ASF/sam/utils/header_files" -I"src/ASF/sam/utils/preprocessor" -I"src/ASF/thirdparty/CMSIS/Include"
CINCLUDES+=-I"src/ASF/thirdparty/CMSIS/Lib/GCC" -I"src/config" -I"$(ToolchainDir)\..\..\CMSIS_Atmel"
CINCLUDES+=-I"$(ToolchainDir)\..\..\CMSIS_Atmel\CMSIS\Include" -I"$(ToolchainDir)\..\..\CMSIS_Atmel\Device\ATMEL"
CINCLUDES+=-I"$(ToolchainDir)\..\..\CMSIS_Atmel\Device\ATMEL\sam4l\include" -I"src/ASF/sam/drivers/bpm"
CINCLUDES+=-I"src/ASF/sam/drivers/flashcalw" -I"src/ASF/common/services/clock" -I"src/ASF/common/services/ioport"
CINCLUDES+=-I"src/ASF/sam/drivers/wdt" -I"src/ASF/common/boards"
CINCLUDES+=-I"src/ASF/sam/drivers/gpio"
CINCLUDES+=-I"src/ASF/sam/drivers/usart"
CINCLUDES+=-I"src/ASF/sam/drivers/cpu"

CWARNS=-Wchar-subscripts -Wcomment -Wformat=2 -Wimplicit-int -Wmain -Wparentheses
CWARNS+=-Wsequence-point -Wreturn-type -Wswitch -Wtrigraphs -Wunused -Wuninitialized
CWARNS+=-Wunknown-pragmas -Wfloat-equal -Wundef -Wshadow -Wbad-function-cast
CWARNS+=-Wwrite-strings -Wsign-compare -Waggregate-return -Wformat -Wmissing-format-attribute
CWARNS+=-Wno-deprecated-declarations -Wpacked -Wredundant-decls -Wnested-externs -Wlong-long
CWARNS+=-Wunreachable-code -Wcast-align -Wall -Wstrict-prototypes
CWARNS+=-Werror-implicit-function-declaration -Wpointer-arith -Wall
CMISCFLAGS=-mthumb -D__SAM4LC8C__ -D__ATSAM4LC8C__ -DARM_MATH_CM4=true  -O1
CMISCFLAGS+=-fdata-sections -ffunction-sections -mlong-calls -g3 -mcpu=cortex-m4
CMISCFLAGS+=-c -pipe -fno-strict-aliasing -std=gnu99 -ffunction-sections
CMISCFLAGS+=-fdata-sections --param max-inline-insns-single=500

CPROJFLAGS=-DBOARD=USER_BOARD -Dprintf=iprintf -Dscanf=iscanf -DPARC_CALLBACK_MODE=false

CFLAGS=$(CINCLUDES) $(CWARNS) $(CMISCFLAGS) $(CPROJFLAGS)

build/src/ASF/common/services/clock/sam4l/%.o: src/ASF/common/services/clock/sam4l/%.c
	mkdir -p build/src/ASF/common/services/clock/sam4l
	$(CCBIN) $(CFLAGS) -o "$@" "$<"

build/src/ASF/sam/drivers/bpm/%.o: src/ASF/sam/drivers/bpm/%.c
	mkdir -p build/src/ASF/sam/drivers/bpm
	$(CCBIN) $(CFLAGS) -o "$@" "$<"

build/src/ASF/sam/drivers/flashcalw/%.o: src/ASF/sam/drivers/flashcalw/%.c
	mkdir -p build/src/ASF/sam/drivers/flashcalw
	$(CCBIN) $(CFLAGS) -o "$@" "$<"

build/src/ASF/sam/drivers/gpio/%.o: src/ASF/sam/drivers/gpio/%.c
	mkdir -p build/src/ASF/sam/drivers/gpio
	$(CCBIN) $(CFLAGS) -o "$@" "$<"

build/src/ASF/sam/drivers/usart/%.o: src/ASF/sam/drivers/usart/%.c
	mkdir -p build/src/ASF/sam/drivers/usart
	$(CCBIN) $(CFLAGS) -o "$@" "$<"

build/src/ASF/sam/drivers/wdt/%.o: src/ASF/sam/drivers/wdt/%.c
	mkdir -p build/src/ASF/sam/drivers/wdt
	$(CCBIN) $(CFLAGS) -o "$@" "$<"

build/src/ASF/common/utils/interrupt/%.o: src/ASF/common/utils/interrupt/%.c
	mkdir -p build/src/ASF/common/utils/interrupt
	$(CCBIN) $(CFLAGS) -o "$@" "$<"

build/src/ASF/sam/utils/cmsis/sam4l/source/templates/gcc/%.o: src/ASF/sam/utils/cmsis/sam4l/source/templates/gcc/%.c
	mkdir -p build/src/ASF/sam/utils/cmsis/sam4l/source/templates/gcc
	$(CCBIN) $(CFLAGS) -o "$@" "$<"

build/src/ASF/sam/utils/cmsis/sam4l/source/templates/%.o: src/ASF/sam/utils/cmsis/sam4l/source/templates/%.c
	mkdir -p build/src/ASF/sam/utils/cmsis/sam4l/source/templates
	$(CCBIN) $(CFLAGS) -o "$@" "$<"

build/src/ASF/sam/utils/syscalls/gcc/%.o: src/ASF/sam/utils/syscalls/gcc/%.c
	mkdir -p build/src/ASF/sam/utils/syscalls/gcc
	$(CCBIN) $(CFLAGS) -o "$@" "$<"

build/src/%.o: src/%.c
	$(CCBIN) $(CFLAGS) -o "$@" "$<"



ifneq ($(MAKECMDGOALS),clean)
ifneq ($(strip $(C_DEPS)),)
-include $(C_DEPS)
endif
endif


LFILE=src/bootloader.ld

JLINK=JLinkExe
JLINK_OPTIONS+=-device ATSAM4LC8C -if swd -speed 1200 -AutoConnect 1

# All Target
all: $(OUTPUT_FILE_PATH) $(ADDITIONAL_DEPENDENCIES)

build/src/jumpfunc.o: src/jumpfunc.s
	$(CCBIN) -c -o "$@" "$<"

build/hail_bootloader.elf: $(OBJS) $(USER_OBJS)
	$(CCBIN) -o$@ $(OBJS) $(LIBS) -mthumb -Wl,-Map="build/$(BINARY_NAME).map" -Wl,--start-group -lm  -Wl,--end-group -L"../src/ASF/thirdparty/CMSIS/Lib/GCC"  -Wl,--gc-sections -mcpu=cortex-m4 -Wl,--entry=Reset_Handler -Wl,--cref -mthumb -T $(LFILE)
	$(SIZE) $@

build/hail_bootloader.bin: build/hail_bootloader.elf
	$(OBJCOPY) -Obinary $^ $@

flash-bootloader: $(OUTPUT_FILE_PATH)
	$(JLINK) $(JLINK_OPTIONS) flash-bootloader.jlink

calib: all
	$(JLINK) calib.jlink || true
# Other Targets
clean:
	rm -rf build

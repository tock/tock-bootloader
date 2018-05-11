/* Memory Spaces Definitions, 448K flash, 64K ram */
ROM_ORIGIN  = 0x00000000;
ROM_LENGTH  = 0x00010000;
RAM_ORIGIN  = 0x20000000;
RAM_LENGTH  = 0x00020000;

MPU_MIN_ALIGN = 8K;

INCLUDE ../kernel_layout.ld
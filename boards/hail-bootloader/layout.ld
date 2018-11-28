/* Memory Spaces Definitions, 448K flash, 64K ram */
MEMORY
{
  rom (rx)  : ORIGIN = 0x00000000, LENGTH = 0x00010000
  ram (rwx) : ORIGIN = 0x20000000, LENGTH = 0x00020000
}

INCLUDE ../kernel_layout.ld
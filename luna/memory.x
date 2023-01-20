# TODO move linker script to lunasoc-pac
# TODO generate linker script automatically from svdgen

MEMORY {
    bootrom       : ORIGIN = 0x00000000, LENGTH = 0x00004000
    scratchpad    : ORIGIN = 0x00008000, LENGTH = 0x00001000
    internal_sram : ORIGIN = 0x40000000, LENGTH = 0x00008000
}

REGION_ALIAS("REGION_TEXT",   internal_sram);
REGION_ALIAS("REGION_RODATA", internal_sram);
REGION_ALIAS("REGION_DATA",   internal_sram);
REGION_ALIAS("REGION_BSS",    internal_sram);
REGION_ALIAS("REGION_HEAP",   internal_sram);
REGION_ALIAS("REGION_STACK",  internal_sram);

/*
 * Automatically generated by LUNA; edits will be discarded on rebuild.
 * (Most header files phrase this 'Do not edit.'; be warned accordingly.)
 *
 * Generated: 2023-02-09 13:42:23.389034.
 */

MEMORY {
    bootrom : ORIGIN = 0x00000000, LENGTH = 0x00004000
    scratchpad : ORIGIN = 0x00004000, LENGTH = 0x00001000
    internal_sram : ORIGIN = 0x40000000, LENGTH = 0x00008000
}

REGION_ALIAS("REGION_TEXT", internal_sram);
REGION_ALIAS("REGION_RODATA", internal_sram);
REGION_ALIAS("REGION_DATA", internal_sram);
REGION_ALIAS("REGION_BSS", internal_sram);
REGION_ALIAS("REGION_HEAP", internal_sram);
REGION_ALIAS("REGION_STACK", internal_sram);

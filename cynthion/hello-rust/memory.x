/*
 * Automatically generated by LUNA; edits will be discarded on rebuild.
 * (Most header files phrase this 'Do not edit.'; be warned accordingly.)
 *
 * Generated: 2023-05-10 10:13:07.967194.
 */

MEMORY {
    bootrom : ORIGIN = 0x00000000, LENGTH = 0x00004000
    scratchpad : ORIGIN = 0x00004000, LENGTH = 0x00001000
    internal_srom   : ORIGIN = 0x10000000, LENGTH = 0x00002000
    internal_rodata : ORIGIN = 0x10002000, LENGTH = 0x00001000
    internal_sram   : ORIGIN = 0x10003000, LENGTH = 0x00001000
    internal_stack  : ORIGIN = 0x10004000, LENGTH = 0x00006000
}

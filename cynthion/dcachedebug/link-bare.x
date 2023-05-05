OUTPUT_FORMAT("elf32-littleriscv")
OUTPUT_ARCH("riscv")
ENTRY(_start)

SECTIONS
{
    . = ORIGIN(internal_sram);

    /* Start of day code. */
    .init :
    {
        *(.init) *(.init.*)
    } > internal_sram
    .text :
    {
        *(.text) *(.text.*)
    } > internal_sram

    .rodata :
    {
        *(.rodata) *(.rodata.*)
    } > internal_sram

    .sdata :
    {
        PROVIDE(__global_pointer$ = .);
        *(.sdata) *(.sdata.*)
    }
    .data :
    {
        *(.data) *(.data.*)
    } > internal_sram

    .bss :
    {
        *(.bss) *(.bss.*)
    } > internal_sram

}

PROVIDE(__stack_top = ORIGIN(internal_sram) + LENGTH(internal_sram));

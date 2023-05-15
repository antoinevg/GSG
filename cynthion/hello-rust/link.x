OUTPUT_FORMAT("elf32-littleriscv")
OUTPUT_ARCH("riscv")

/* _start */
ENTRY(_start)

SECTIONS
{
    . = ORIGIN(internal_sram);

    /* .init */
    .init :
    {
        *(.init) *(.init.*)
    } > internal_sram

    /* .text */
    .text :
    {
        *(.text) *(.text.*)
    } > internal_sram

    /* .rodata */
    .rodata :
    {
        *(.rodata) *(.rodata.*)
    } > internal_sram

    /* .sdata */
    .sdata :
    {
        PROVIDE(__global_pointer$ = .);
        *(.sdata) *(.sdata.*)
    } > internal_sram

    /* .data */
    .data :
    {
        *(.data) *(.data.*)
    } > internal_sram

    /* .bss */
    .bss :
    {
        *(.bss) *(.bss.*)
    } > internal_sram

}

/* stack */
PROVIDE(__stack_top = ORIGIN(internal_sram) + LENGTH(internal_sram));

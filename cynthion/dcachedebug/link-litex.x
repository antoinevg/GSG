OUTPUT_FORMAT("elf32-littleriscv")
ENTRY(_start)

/*MEMORY
{
    bootrom (rx) : ORIGIN = 0x0, LENGTH = 16384
    scratchpad (rwx) : ORIGIN = 0x4000, LENGTH = 4096
}*/

PROVIDE(_stext = ORIGIN(REGION_TEXT));
PROVIDE(_stack_top = ORIGIN(REGION_STACK) + LENGTH(REGION_STACK));

PROVIDE(_max_hart_id = 0);
PROVIDE(_hart_stack_size = 2K);
PROVIDE(_heap_size = 0);

PROVIDE(MachineExternal = DefaultHandler);
PROVIDE(DefaultHandler = DefaultInterruptHandler);
PROVIDE(ExceptionHandler = DefaultExceptionHandler);

/* A PAC/HAL defined routine that should initialize custom interrupt controller if needed. */
PROVIDE(_setup_interrupts = default_setup_interrupts);

/* # Start trap function override
  By default uses the riscv crates default trap handler
  but by providing the `_start_trap` symbol external crates can override.
*/
PROVIDE(_start_trap = default_start_trap);

SECTIONS
{
    /* .text @ _stext */
    .text _stext:
    {

        /* .init */
        KEEP(*(.init));
        KEEP(*(.init.rust));

        /* .trap */
        . = ALIGN(8);
        *(.trap);
        *(.trap.rust);

        /* .text */
        *(.text .text.*);

        /*_etext = .;*/
    } > REGION_TEXT

    /* .rodata */
    .rodata :
    {
        . = ALIGN(8);
        /*_srodata = .;*/

        *(.srodata .srodata.*);
        *(.rodata .rodata.*);

        FILL(0);
        . = ALIGN(8);

        /*_erodata = .;*/
    } > REGION_RODATA

    /* .data */
    .data : ALIGN(8) /* align section start to 8 byte boundary */
    {
        _sidata = LOADADDR(.data);

        /*. = ALIGN(8); */ /* align symbol below to 8 byte boundary */
        _sdata = .;

        PROVIDE(__global_pointer$ = . + 0x800);
        /*PROVIDE(__global_pointer$ = .);*/
        *(.sdata .sdata.* .sdata2 .sdata2.*);
        *(.data .data.*);

        FILL(0);
        . = ALIGN(8);

        _edata = .;
    } > REGION_DATA AT > REGION_RODATA

    /* .bss */
    .bss (NOLOAD) :
    {
        . = ALIGN(8);
        _sbss = .;

        *(.sbss .sbss.* .bss .bss.*);
        *(COMMON)

        . = ALIGN(8);

        _ebss = .;
    } > REGION_BSS

    /* .heap */
    .heap (NOLOAD) :
    {
        . = ALIGN(8);
        _sheap = .;
        . += _heap_size;
        . = ALIGN(8);
        _eheap = .;
    } > REGION_HEAP

    /* .stack */
    .stack (NOLOAD) :
    {
        . = ALIGN(8);
        _estack = .;
        . = ABSOLUTE(_stack_top);

        . = ALIGN(8);
        _sstack = .;
    } > REGION_STACK

    /DISCARD/ :
    {
        *(.eh_frame)
        *(.comment)
    }
}

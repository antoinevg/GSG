# define STORE    sw
# define LOAD     lw
# define LOG_REGBYTES 2
#define REGBYTES (1 << LOG_REGBYTES)

/**
 * Entry point of all programs (_start).
 *
 * It initializes DWARF call frame information, the stack pointer, the
 * frame pointer (needed for closures to work in start_rust) and the global
 * pointer. Then it calls _start_rust.
 */

.global _start
_start:
    j crt_init
    nop
    nop
    nop
    nop
    nop
    nop
    nop

/* = .trap ===================================================================== */

.section .trap, "ax"

/**
 * Trap entry point (_start_trap)
 *
 * Saves caller saved registers ra, t0..6, a0..7, calls _start_trap_rust,
 * restores caller saved registers and then returns.
 */
.global default_start_trap
default_start_trap:
    sw x1,  - 1*4(sp)
    sw x5,  - 2*4(sp)
    sw x6,  - 3*4(sp)
    sw x7,  - 4*4(sp)
    sw x10, - 5*4(sp)
    sw x11, - 6*4(sp)
    sw x12, - 7*4(sp)
    sw x13, - 8*4(sp)
    sw x14, - 9*4(sp)
    sw x15, -10*4(sp)
    sw x16, -11*4(sp)
    sw x17, -12*4(sp)
    sw x28, -13*4(sp)
    sw x29, -14*4(sp)
    sw x30, -15*4(sp)
    sw x31, -16*4(sp)
    addi sp,sp,-16*4

    jal ra, _start_trap_rust

    lw x1 , 15*4(sp)
    lw x5,  14*4(sp)
    lw x6,  13*4(sp)
    lw x7,  12*4(sp)
    lw x10, 11*4(sp)
    lw x11, 10*4(sp)
    lw x12,  9*4(sp)
    lw x13,  8*4(sp)
    lw x14,  7*4(sp)
    lw x15,  6*4(sp)
    lw x16,  5*4(sp)
    lw x17,  4*4(sp)
    lw x28,  3*4(sp)
    lw x29,  2*4(sp)
    lw x30,  1*4(sp)
    lw x31,  0*4(sp)
    addi sp,sp,16*4

    mret
    .text

/* = .init -=================================================================== */

.section .init /*, "ax"*/

crt_init:
    /*.cfi_startproc
    .cfi_undefined ra*/

    /* Set up our global pointer. */
    /*.option push
    .option norelax
    la gp, __global_pointer$
    .option pop*/

    /* Set up our stack. */
    la sp, _stack_top    // set stack ponter
    add s0, sp, zero     // set frame pointer

    jal zero, _start_rust

    //.cfi_endproc


/* = .init.rust ================================================================ */

    // rt.rs





/* = .trap.rust =============================================================== */

    // From: rt.rs


/* = .text ==================================================================== */


/* = .text.abort ============================================================== */

/* Make sure there is an abort when linking */
.section .text.abort
.globl abort
abort:
    j abort

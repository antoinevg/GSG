## Theory: rodata needs to be aligned to 64

    .rodata  0x10001a10   breaks  % 64 == 16
    .rodata  0x10001a40   works   % 64 == 0     (+48 bytes using 12 nops)
    .rodata  0x10001a80   works   % 64 == 0     (+112 bytes using 28 nops so each nop is 4 bytes)


## Theory: rodata needs to be aligned to 32

    .rodata  0x10001a10   breaks  % 32 == 16
    .rodata  0x10001a20   works   % 32 == 0


* So we can either align rodata with a 2nd writeln or with 16 bytes worth of nops?

Counter evidence:

Three writeln! gives me 0x10001adc which is not aligned (mod 28)

    0x10001aec  breaks  (3 writelns + 4 nops)
    0x10001adc  works   (3 writelns)
    0x10001a80  works   (2 writelns)
    0x10001a80  breaks  (4 writelns)


## Theory: All sections need a certain alignment

    # works: 1 - writefmt + 4 nops
    .rodata  0x10001a20
    .data    0x10001d04
    .stack   0x10001d08

    # works: 2 - writefmt + 1 writeln
    .rodata  0x10001a80
    .data    0x10001d80
    .stack   0x10001d84

    # works: 3 - writefmt + 2 writeln
    .rodata  0x10001adc
    .data    0x10001dec
    .stack   0x10001df0

    ... see sections.numbers

## Theory: stack size needs to be a certain alignment?

    nope


## Observation: There is definitely a 32 byte alignment thing happening

You can see it when you look at 5 - adding extra nops in multiples of
16 bytes alternately break and work.

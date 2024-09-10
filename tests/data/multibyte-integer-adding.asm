.data 0
num1 [0x52, 0xbf, 0x01, 0x00] _
num2 [0x42, 0x4b, 0x1d, 0x00] _
dst [] _

.entry start

.code
start:
    nop
    ld 0 r0
    ld 4 r1
    add r0 r1 r0
    st 8 r0
    ld 1 r0
    ld 5 r1
    cadd r0 r1 r0
    st 9 r0
    ld 2 r0
    ld 6 r1
    cadd r0 r1 r0
    st 10 r0
    ld 3 r0
    ld 7 r1
    cadd r0 r1 r0
    st 11 r0
    halt
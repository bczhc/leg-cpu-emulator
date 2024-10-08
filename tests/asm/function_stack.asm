.data 0
arr [1, 2, 3, 4, 5] length
ss [] _ ; stack memory start

.entry start

.code
start:
    cp ss fss ; start from the address `ss`, it's the memory area for stack uses
    fpush 0
    fpush length
    fpush arr
    call foo ; n starts with 0
    halt


; args:
; - arr: address of a u8 array
; - an: array length
; - n: a recursion counter. This function stops at n=5
foo:
    push r0
    push r1
    push r2
    push r3

    ; assume this function will use some amount of stack memory, say 4 bytes.
    ; so increase `fss` first
    ; at the end of the function, decrease fss back.
    ; this simulates the "stack frame"
    ; note: it's useful to use `anc` and `snc` to manipulate address offsets, if needed,
    ; just to prevent setting the carry register

    cp fss r11 ; take the `fss` start position
    anc fss 4 fss
    ; now the variables allocated on "stack" start from `r11`

    fpop r0 ; arr
    fpop r1 ; an
    fpop r2 ; n
    jamv f1_end
    jpeq r2 5

    ; store all arguments to memory
    st r11 r0
    anc r11 1 r3
    st r3 r1
    anc r11 2 r3
    st r3 r2

    ; increase every items in `arr`
    add r0 r1 r1 ; r1 is the end address now
    for1:
    ld r0 r2 ; r2: *r0
    add r2 1 r2
    st r0 r2
    add r0 1 r0
    jamv for1
    jplt r0 r1

    ; call self with an increased `n`
    ; when calling a function, the order of arguments by `fpush` is reversed
    anc r11 2 r3
    ld r3 r0 ; r0: n
    add r0 1 r0 ; r0: n + 1
    fpush r0
    anc r11 1 r3
    ld r3 r0; r0: an
    fpush r0
    ld r11 r0; r0: arr
    fpush r0
    call foo

    f1_end:
    snc fss 4 fss ; decrease back
    pop r3
    pop r2
    pop r1
    pop r0
    ret
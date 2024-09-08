.consts

.data 0
m_data [1, 2, 3, 4, 5, 6, 7, 8, 9, 10] data_len

.entry start

.code
f_fib:
    push r0 ; back up registers
    push r1
    push r2

    fpop r0 ; arg1: n
    jple r0 1 if_L1 ; if n <= 1, return n
    sub r0 1 r0
    fpush r0
    call f_fib ; fib(n-1)
    fpop r1
    sub r0 1 r0
    fpush r0
    call f_fib ; fib(n-2)
    fpop r2
    add r1 r2 r1
    fpush r1 ; will return fib(n-1) + fib(n-2)
    jp f_fib_ret

    if_L1:
    fpush r0
    jp f_fib_ret
    f_fib_ret:
    pop r2
    pop r1
    pop r0
    ret

start:
    cp 0 r1
    for_L1:
    add m_data r1 r2
    ld r2 r0

    fpush r0
    call f_fib
    fpop r0
    st r2 r0

    add r1 1 r1
    jpeq r1 data_len for_L2
    jp for_L1
    for_L2:
    halt

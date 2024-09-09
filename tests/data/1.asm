.data 0
arr [2, 0, 2, 2, 0, 9, 1, 0] len

.entry start

.code
; args: arr, length, dst
f_foo:
    push r0
    push r1
    push r2
    push r3
    push r4
    push r5

    fpop r0 ; arr
    fpop r1 ; length

    cp 0 r4
    ld r0 r2
    ld r0 r5

    loop1:
    ld r0 r3
    add r4 r3 r4

    jamv if1
    jpge r3 r2
    cp r3 r2
    if1:
    jamv if2
    jple r3 r5
    cp r3 r5
    if2:

    add r0 1 r0
    jamv loop1
    jplt r0 len

    fpop r0
    st r0 r2
    add r0 1 r0
    st r0 r5

    fpush r4
    pop r5
    pop r4
    pop r3
    pop r2
    pop r1
    pop r0
    ret

start:
    add arr len r1
    fpush r1
    fpush len
    fpush arr
    call f_foo
    fpop r0
    add r1 2 r1
    st r1 r0
    halt
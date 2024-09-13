.consts
length 16
a_sum 16

.entry _start

.code
_start:
    jamv start
    jp
f_scan_right:
    push r0
    fpop r0 ; start

    ; for (int i = 1; i <= bars[start]; ++i)
    ld r0 r1 ; r1 <- bars[start]
    cp 1 r2
    loop1_start:
    ; loop1 {
    ; for (int j = start + 1; j <= 15; ++j)
    add 1 r0 r3
    loop2_start:
    ; loop2 {
    ; if (bars[j] >= i) { sum += j - start - 1; break; }
    ld r3 r4 ; r4 <- bars[j]
    jamv if_L1
    jplt r4 r2
    add r0 1 r5
    push r0 ; back up r0
    sub r3 r5 r5
    ld a_sum r0
    add r0 r5 r0
    st a_sum r0
    pop r0 ; restore r0
    jamv loop2_break
    jp

    if_L1:
    ; } loop2
    add 1 r3 r3
    jamv loop2_start
    jple r3 15
    ; } loop1
    loop2_break:
    add 1 r2 r2
    jamv loop1_start
    jple r2 r1
    loop1_end:


    pop r0
    ret
    ; -----------------------------

start:
    ; copy input to memory
    ; -------------------------
    cp 0 r0
    L1:
    st r0 in
    add 1 r0 r0
    jamv L2
    jpeq r0 length
    jamv L1
    jp
    L2:
    ; -------------------------

    st a_sum 0 ; sum <- 0

    ; for start=0 to 15
    cp 0 r0
    L4:
    fpush r0
    call f_scan_right
    add 1 r0 r0
    jamv L4
    jple r0 15
    L3:

    ld a_sum r0
    cp r0 out
    halt

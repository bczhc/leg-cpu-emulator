.consts
a_array 0
length 16
a_sum 16
a_input 17

.entry _start

.code
_start:
    jamv start
    jp

; read input string until an LF
; args: buf
; returns: input string length
f_read_input:
    push r0
    push r1
    push r2

    fpop r2 ; buf
    push r2
    loop3:
    cp in r1
    ; break if a '\n' met
    jamv loop3_out
    jpeq r1 0x0a
    st r2 r1
    add r2 1 r2
    jamv loop3
    jp
    loop3_out:

    pop r0
    ; length = r2 - r0
    sub r2 r0 r0
    fpush r0

    pop r2
    pop r1
    pop r0
    ret

; args: input_addr, length, buf_addr
f_parse_input_line:
    push r0
    push r1
    push r2
    push r3
    push r4

    fpop r0 ; input
    fpop r1 ; length
    fpop r2 ; buf

    cp 0 r3 ; i = 0;
    cp 0 r4 ; j = 0;
    loop4:
    push r0 ; backup input
    add r0 r4 r0
    ld r0 r0 ; r0: input[j]
    ; if r0 == 0x2c (',') { ++i }
    jamv if1
    jpne r0 0x2c
    add r3 1 r3
    if1:

    ; buf[i] = c - 48
    sub r0 48 r0
    push r2
    add r2 r3 r2
    st r2 r0
    pop r2

    pop r0 ; restore input

    add r4 1 r4
    jamv loop4
    jplt r4 r1

    pop r4
    pop r3
    pop r2
    pop r1
    pop r0
    ret

; args: u8 number
; the number must be between 0 and 99
f_print_u8:
    push r0
    push r1
    fpop r0 ; num

    push r0
    div r0 10 r0
    add r0 48 r1 ; digit 1

    pop r0
    mod r0 10 r0
    add r0 48 r0 ; digit 2

    ; if d1 != 48, print d1
    jamv if2
    jpeq r1 48
    cp r1 out
    if2:

    ; print d2 and a newline
    cp r0 out
    cp 0x0a out
    pop r1
    pop r0
    ret

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
    fpush a_input
    call f_read_input
    fpop r0 ; r0: length of input

    fpush a_array
    fpush r0
    fpush a_input
    call f_parse_input_line

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

    ld a_sum r0 ; r0: the result
    fpush r0
    call f_print_u8
    
    halt

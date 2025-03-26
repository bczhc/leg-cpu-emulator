; Print prime numbers 2-500.
;
; © 2025 Zhai Can <bczhc0@126.com>, under MIT.

.data 0x0000
ss [] _

.consts
NEW_LINE 10
; 16-bit numbers
MAX_LE_HI 0x01 ; 0x01f4, 500
MAX_LE_LO 0xf4

.entry main

.code
main:
    cp ss fss

    cp 2 r0 ; counter lo
    cp 0 r1 ; counter hi

    loop:
    fpush r1
    fpush r0
    call is_prime
    fpop r3
    jamv skip_print
    jpeq azr r3
    print_num:
    fpush r1
    fpush r0
    call f_print_int16
    cp NEW_LINE out
    skip_print:

    ; add one
    fpush r1
    fpush r0
    call int16_add_1
    fpop r0
    fpop r1


    fpush MAX_LE_HI
    fpush MAX_LE_LO
    fpush r1
    fpush r0
    call cmp_int16
    fpop r2
    jamv loop
    jpeq azr r2

    halt

; arg: 16-bit number
; return: 0 (false), 1 (true)
is_prime:
    push r0
    push r1
    push r2
    push r3
    push r4
    fpop r0 ; n-lo
    fpop r1 ; n-hi
    cp 0x02 r2 ; i-lo
    cp 0x00 r3 ; i-hi

    ; corner case for 0x0002
    fpush 0x00
    fpush 0x02
    fpush r1
    fpush r0
    call cmp_int16
    fpop r4
    jamv is_prime_prime
    jpeq aor r4

    is_prime_loop:
    fpush r3
    fpush r2
    fpush r1
    fpush r0
    call int16_is_dividable
    fpop r4
    jamv is_prime_not_prime
    jpeq r4 aor

    ; add one
    fpush r3
    fpush r2
    call int16_add_1
    fpop r2
    fpop r3

    fpush r3
    fpush r2
    fpush r1
    fpush r0
    call cmp_int16
    fpop r4
    jamv is_prime_loop
    jpeq azr r4

    is_prime_prime:
    ; prime
    fpush aor
    jamv is_prime_end

    is_prime_end:
    pop r4
    pop r3
    pop r2
    pop r1
    pop r0
    ret
    ; not prime
    is_prime_not_prime:
    fpush azr
    jamv is_prime_end
    jp



; args: num1-LE, num2-LE
; returns: result-LE
add16:
    push r0
    push r1
    push r2
    fpop r0
    fpop r1
    fpop r2
    add r0 r2 r0
    fpop r2
    cadd r1 r2 r1
    fpush r1
    fpush r0
    pop r2
    pop r1
    pop r0
    ret

; args: num-LE
; returns: result-LE
int16_add_1:
    ; one: 0x0001, LE: 01 00
    ; one number has already been f-pushed
    fpush 0x00
    fpush 0x01
    call add16
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
    jamv print_u8_if2
    jpeq r1 48
    cp r1 out
    print_u8_if2:

    ; print d2 and a newline
    cp r0 out
    cp 0x0a out
    pop r1
    pop r0
    ret

; args: num1-LE, num2-LE
; returns: 0 (false), 1 (true)
cmp_int16:
    push r0
    push r1
    push r2
    push r3
    fpop r0
    fpop r1
    fpop r2
    fpop r3

    jamv cmp_int16_ne
    jpne r0 r2
    jamv cmp_int16_ne
    jpne r1 r3
    ; equal
    fpush 0x01
    cmp_int16_end:
    pop r3
    pop r2
    pop r1
    pop r0
    ret
    cmp_int16_ne:
    fpush 0x00
    jamv cmp_int16_end
    jp

; 16-bit number subtraction
; result=num1 + ~num2 + 1
; args: num1-LE num2-LE
; returns: result-LE
sub16:
    push r0
    push r1
    push r2
    push r3
    fpop r0
    fpop r1
    fpop r2
    fpop r3
    ; flip num2 bits
    not r2 azr r2
    not r3 azr r3
    ; add them
    ; [r0 r1 r2 r3]
    fpush r3
    fpush r2
    fpush r1
    fpush r0
    call add16
    ; do not fpop, add more "one"
    fpush 0x00
    fpush 0x01
    call add16
    ; done. do not fpop, return

    pop r3
    pop r2
    pop r1
    pop r0
    ret

; Check if one number divides another
; all in 16 bit
; arg: dividend, divisor
; return: 0 (false), 1 (true)
int16_is_dividable:
    push r0
    push r1
    call int16_div_rem
    fpop r0 ; discard quotient
    fpop r0
    fpop r0 ; rem-lo
    fpop r1 ; rem-hi
    ; test if rem is zero
    fpush r1
    fpush r0
    fpush 0x00
    fpush 0x00
    call cmp_int16
    pop r1
    pop r0
    ret


; Perform division and modulo for a 16-bit number
; arg: dividend, divisor
; result: quotient, modulus
int16_div_rem:
    push r0
    push r1
    push r2
    push r3
    push r4
    push r5 ; stack memory pointer

    ; stack memory:
    ; [0], [1]: current 'dividend'
    ; [2], [3]: quotient

    cp fss r5
    anc r6 2 r6
    anc fss 4 fss

    ; init quotient = 0
    anc r5 2 r5
    st r5 azr
    anc r5 aor r5
    st r5 azr
    snc r5 3 r5

    cp azr r6
    fpop r0 ; dividend-lo
    fpop r1 ; divident-hi
    fpop r2 ; divisor-lo
    fpop r3 ; divisor-hi

    ; if divisor == 0, undefined
    fpush 0x00
    fpush 0x00
    fpush r3
    fpush r2
    call cmp_int16
    fpop r4
    jamv int16_div_rem_undefined
    jpne r4 azr

    ; loop subtraction until zero or negative
    int16_div_rem_loop:
    anc r5 2 r5
    fpush r5
    snc r5 2 r5
    call int16_mem_inc ; inc quotient
    fpush r3
    fpush r2
    fpush r1
    fpush r0
    call sub16
    fpop r0
    fpop r1
    st r5 r0
    anc r5 aor r5
    st r5 r1

    ; zero or negative?
    push r0
    ld r5 r0
    fpush r0
    snc r5 aor r5
    ld r5 r0
    fpush r0
    pop r0
    fpush 0x00
    fpush 0x00
    call cmp_int16
    fpop r4
    jamv int16_div_rem_if2
    jpeq r4 azr
    ; dividable, (q, m) = (r6, 0)
    fpush azr
    fpush azr
    push r0
    anc r5 3 r5
    ld r5 r0
    fpush r0 ; quotient-hi
    snc r5 1 r5
    ld r5 r0
    fpush r0 ; quotient-lo
    snc r5 2 r5 ; reset r5
    pop r0
    jamv int16_div_rem_end
    jp
    int16_div_rem_if2:

    ; or negative, then not dividable
    push r0
    push r1
    ; [r0 r1]
    ld r5 r0
    anc r5 aor r5
    ld r5 r1
    snc r5 aor r5
    and r1 0x80 r1
    cp 1 r4
    jamv int16_div_rem_if1
    jpne azr r1
    cp 0 r4
    int16_div_rem_if1:
    pop r1
    pop r0
    ; negative if r4==1
    ; not dividable
    jamv int16_div_rem_negative
    jpeq 1 r4
    ; or, continue the loop
    jamv int16_div_rem_loop
    jp
    int16_div_rem_negative:
    ; the actual quotient shoule be decreased by one
    ; add back one divisor to get the remainder
    push r2
    push r3
    ld r5 r2
    anc r5 aor r5
    ld r5 r3
    snc r5 aor r5
    fpush r3
    fpush r2
    pop r3
    pop r2
    fpush r3
    fpush r2
    call add16
    ; now the remainder is in the f-stack

    anc r5 2 r5
    fpush r5
    call int16_mem_dec
    push r0
    anc r5 1 r5
    ld r5 r0 ; quotient-hi
    fpush r0
    snc r5 1 r5
    ld r5 r0 ; quotient-lo
    fpush r0
    snc r5 2 r5 ; reset r5
    pop r0
    jamv int16_div_rem_end
    jp

    int16_div_rem_end:
    snc fss 4 fss
    pop r5
    pop r4
    pop r3
    pop r2
    pop r1
    pop r0
    ret
    int16_div_rem_undefined:
    fpush azr
    fpush azr
    fpush azr
    fpush azr
    jamv int16_div_rem_end
    jp

; arg: num-LE addr
int16_mem_inc:
    push r0
    push r1 ; lo
    push r2 ; hi
    fpop r0 ; addr
    ld r0 r1
    anc r0 aor r0
    ld r0 r2
    fpush 0x00
    fpush 0x01
    fpush r2
    fpush r1
    call add16
    fpop r1
    fpop r2
    st r0 r2
    snc r0 aor r0
    st r0 r1
    pop r2
    pop r1
    pop r0
    ret

; arg: num-LE addr
int16_mem_dec:
    push r0
    push r1 ; lo
    push r2 ; hi
    fpop r0 ; addr
    ld r0 r1
    anc r0 aor r0
    ld r0 r2
    fpush 0x00
    fpush 0x01
    fpush r2
    fpush r1
    call sub16
    fpop r1
    fpop r2
    st r0 r2
    snc r0 aor r0
    st r0 r1
    pop r2
    pop r1
    pop r0
    ret

; Perform integer modulus
; arg: dividend, divisor
; return: result
int16_mod:
    push r0
    call int16_div_rem
    fpop r0 ; discard quotient
    fpop r0
    pop r0
    ret

; Perform integer division
; arg: dividend, divisor
; return: result
int16_div:
    push r0
    push r1
    call int16_div_rem
    fpop r0 ; lo
    fpop r1 ; hi
    push r0
    fpop r0 ; discard remainder
    fpop r0
    pop r0
    fpush r1
    fpush r0
    pop r1
    pop r0
    ret

; Print a 16-bit number in decimal.
; arg: num-LE
f_print_int16:
    ; consts:
    ; 10000: 2710H
    ; 1000: 03E8H
    ; 100: 0064H
    ; 10: 000AH
    ; '0': 0x30
    push r0
    push r1
    push r2
    push r3
    push r4 ; leading zero flag
    fpop r2 ; lo
    fpop r3 ; hi

    fpush 0x27
    fpush 0x10
    fpush r3
    fpush r2
    call int16_div
    fpop r0
    fpop r1
    fpush 0x00
    fpush 0x0a
    fpush r1
    fpush r0
    call int16_mod
    call handle_digit
    fpop r0 ; discard remainder-hi - it's always zero

    fpush 0x03
    fpush 0xe8
    fpush r3
    fpush r2
    call int16_div
    fpop r0
    fpop r1
    fpush 0x00
    fpush 0x0a
    fpush r1
    fpush r0
    call int16_mod
    call handle_digit
    fpop r0 ; discard remainder-hi - it's always zero

    fpush 0x00
    fpush 0x64
    fpush r3
    fpush r2
    call int16_div
    fpop r0
    fpop r1
    fpush 0x00
    fpush 0x0a
    fpush r1
    fpush r0
    call int16_mod
    call handle_digit
    fpop r0 ; discard remainder-hi - it's always zero

    fpush 0x00
    fpush 0x0A
    fpush r3
    fpush r2
    call int16_div
    fpop r0
    fpop r1
    fpush 0x00
    fpush 0x0a
    fpush r1
    fpush r0
    call int16_mod
    call handle_digit
    fpop r0 ; discard remainder-hi - it's always zero

    fpush 0x00
    fpush 0x01
    fpush r3
    fpush r2
    call int16_div
    fpop r0
    fpop r1
    fpush 0x00
    fpush 0x0a
    fpush r1
    fpush r0
    call int16_mod
    call handle_digit
    fpop r0 ; discard remainder-hi - it's always zero

    pop r4
    pop r3
    pop r2
    pop r1
    pop r0
    ret

    handle_digit:
    push r0
    fpop r0
    jamv handle_digit_print
    jpne azr r4 ; do not ignore zeros after an initial print
    jamv handle_digit_end
    jpeq azr r0
    handle_digit_print:
    ; convert to number ASCII
    anc r0 0x30 r0
    cp aor r4
    cp r0 out
    handle_digit_end:
    pop r0
    ret

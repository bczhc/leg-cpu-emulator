.data 0
num1 [0x52, 0xbf, 0x01, 0x00] _
num2 [0x42, 0x4b, 0x1d, 0x00] _
result [0x94, 0x0a, 0x1f, 0x00] _
temp [] _

.entry start

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

; args: num1-LE, num2-LE
; returns: result-LE
add32:
    push r0
    push r1
    push r2
    push r3

    fpop r0 ; num1[0]
    fpop r1 ; num1[1]
    fpop r2 ; num1[2]
    fpop r3 ; num1[3]
    fpop r4 ; num2[0]
    add r0 r4 r0 ; r0: res[0]
    fpop r4 ; num2[1]
    cadd r1 r4 r1 ; r1: res[1]
    fpop r4 ; num2[2]
    cadd r2 r4 r2 ; r2: res[2]
    fpop r4 ; num2[3]
    cadd r3 r4 r3 ; r3: res[3]

    fpush r3
    fpush r2
    fpush r1
    fpush r0

    pop r3
    pop r2
    pop r1
    pop r0
    ret

; args: num1-LE, num2-LE
; returns: result-LE
add64:
    push r0
    push r1
    push r2

    ; have to use memory now
    cp 0 r1
    for1:
    add temp r1 r2
    fpop r0
    st r2 r0
    jamv for1_end
    jpeq r1 7
    add r1 1 r1
    for1_end:
    ; now temp: num1[0], num1[1], ..., num1[7]
    fpop r0 ; num2[0]
    ld temp+0 r1
    add r0 r1 r0 ; r0: res[0]
    st temp+0 r0
    fpop r0 ; num2[1]
    ld temp+1 r1
    cadd r0 r1 r0 ; r0: res[1]
    st temp+1 r0
    fpop r0 ; num2[1]
    ld temp+2 r1
    cadd r0 r1 r0 ; r0: res[2]
    st temp+2 r0
    fpop r0 ; num2[1]
    ld temp+3 r1
    cadd r0 r1 r0 ; r0: res[3]
    st temp+3 r0
    fpop r0 ; num2[1]
    ld temp+4 r1
    cadd r0 r1 r0 ; r0: res[4]
    st temp+4 r0
    fpop r0 ; num2[1]
    ld temp+5 r1
    cadd r0 r1 r0 ; r0: res[5]
    st temp+5 r0
    fpop r0 ; num2[1]
    ld temp+6 r1
    cadd r0 r1 r0 ; r0: res[6]
    st temp+6 r0
    fpop r0 ; num2[1]
    ld temp+7 r1
    cadd r0 r1 r0 ; r0: res[7]
    st temp+7 r0
    ; now temp: res[0], res[1], ..., res[7]

    cp 7 r0
    for2:
    add temp r0 r1
    ld r1 r2 ; r2: res[i]
    fpush r2
    jamv for2_end
    jpeq r0 0
    sub r0 1 r0
    for2_end:

    fpop r2
    fpop r1
    fpop r0
    ret

.code
start:
    ; 0xfefe + 0x00ef = 0xffed
    ; LE: fefe ef00 edff
    fpush 0x00
    fpush 0xef
    fpush 0xfe
    fpush 0xfe
    call add16
    fpop r0 ; 0xed
    fpop r1 ; 0xff
    halt
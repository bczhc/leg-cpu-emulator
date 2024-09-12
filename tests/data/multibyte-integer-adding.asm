.data 0
add32_data [0x00, 0x1d, 0x4b, 0x42, 0x00, 0x01, 0xbf, 0x52] _
add64_data [0xbc, 0x65, 0x59, 0x6e, 0x3a, 0xb0, 0xa9, 0xd1, 0xde, 0x15, 0x6a, 0xf8, 0x37, 0x26, 0xf4, 0xdd] _
add64_result [0xae, 0x9e, 0xd7, 0x71, 0x66, 0xc4, 0x7a, 0x9a] _

.entry _start

.code
_start:
    jamv start
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
    push r3
    push r4
    push r5
    push r6
    push r7
    push r8

    fpop r0 ; num1[0]
    fpop r1 ; num1[1]
    fpop r2 ; ...
    fpop r3
    fpop r4
    fpop r5
    fpop r6
    fpop r7
    fpop r8 ; num2[0]

    add r0 r8 r0 ; r0: res[0]
    fpop r8 ; num2[1]
    cadd r1 r8 r1 ; r1: res[1]
    fpop r8
    cadd r2 r8 r2
    fpop r8
    cadd r3 r8 r3
    fpop r8
    cadd r4 r8 r4
    fpop r8
    cadd r5 r8 r5
    fpop r8
    cadd r6 r8 r6
    fpop r8
    cadd r7 r8 r7 ; r7: res[7]

    fpush r7
    fpush r6
    fpush r5
    fpush r4
    fpush r3
    fpush r2
    fpush r1
    fpush r0

    pop r8
    pop r7
    pop r6
    pop r5
    pop r4
    pop r3
    pop r2
    pop r1
    pop r0
    ret

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
    jamv error
    jpne r0 0xed
    jpne r1 0xff

    ; 0x0001bf52 + 0x001d4b42 = 0x001f0a94
    ; args: 52 bf 01 00 42 4b 1d 00
    cp 0 r0
    for1:
    add add32_data r0 r1
    ld r1 r2
    fpush r2
    add r0 1 r0
    jamv for1
    jplt r0 8
    call add32
    jamv error
    fpop r0
    jpne r0 0x94
    fpop r0
    jpne r0 0x0a
    fpop r0
    jpne r0 0x1f
    fpop r0
    jpne r0 0x00

    ; 0xde156af83726f4dd + 0xbc65596e3ab0a9d1 = 0x1_9a7ac46671d79eae
    ; bc 65 59 6e 3a b0 a9 d1 de 15 6a f8 37 26 f4 dd
    ; ae 9e d7 71 66 c4 7a 9a
    cp 0 r0
    for2:
    add add64_data r0 r1
    ld r1 r2
    fpush r2
    add r0 1 r0
    jamv for2
    jplt r0 16
    call add64

    cp 0 r0
    for3:
    add add64_result r0 r1
    ld r1 r2
    fpop r3
    jamv error
    jpne r2 r3
    add r0 1 r0
    jamv for3
    jplt r0 8

    ok:
    halt

    error:
    halt
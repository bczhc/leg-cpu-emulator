.entry _start ; For now the entrypoint address supports only 8bits still.

.code
_start:
jamv start ; set the jump address register. This is an address out of 8-bit range.
jp ; do the jump, to `start` label

nop
nop
nop
nop
nop
nop
nop
nop
nop
nop
nop
nop
nop
nop
nop
nop
nop
nop
nop
nop
nop
nop
nop
nop
nop
nop
nop
nop
nop
nop
nop
nop
nop
nop
nop
nop
nop
nop
nop
nop
nop
nop
nop
nop
nop
nop
nop
nop
nop
nop
nop
nop
nop
nop
nop
nop
nop
nop
nop
nop
nop
nop
nop
nop
nop
nop
nop
nop
nop
nop
nop
nop
nop
nop
nop
nop

; args: num1, num2
; returns: num1 + num2
f_add:
    push r0
    push r1
    fpop r1
    fpop r0
    add r0 r1 r0
    fpush r0
    pop r1
    pop r0
    ret

start:
    cp 2 r0
    cp 3 r1
    fpush r1
    fpush r0
    call f_add ; also the address of f_add is out of 8-bit range
    fpop r0
    cp r0 out
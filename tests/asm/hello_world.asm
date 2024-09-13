.data 0
text 'hello, world' text_len

.entry start

.code
start:
    cp 0 r0 ; r0 <- 0
    for1:
    add r0 text r1
    ld r1 r1 ; r1 <- text[i]
    cp r1 out ; print character out
    add r0 1 r0 ; increase r0
    jamv for1 ; move address of label 'for1' to the jump-address register
    jplt r0 text_len ; do the jump if r0 < text_len
    cp 0x0a out ; print the new-line
    halt

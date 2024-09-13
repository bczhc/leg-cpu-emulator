.data 0
text 'hello, world' text_len

.entry start

.code
start:
    cp 0 r0 ; i = 0
    for_L1:
    add r0 text r1
    ld r1 r1 ; r1 <- text[i]
    cp r1 out
    add r0 1 r0 ; ++i
    jamv for_L1 ; move address of label 'for_L1' to the jump register
    jplt r0 text_len ; do the jump if r0 < text_len
    cp 0x0a out ; the new-line
    halt

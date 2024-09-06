.consts
not_used 0
blabla 0xff

.data 0
text 'hello, world' text_len
array [1, 2, 3, 0b100] array_len
num 12 _

.entry start

.code
start:
    cp 0 r0 ; -> imm1|cp 0 x r0
    for_L1:
    add r0 text r1 ; -> imm2|add r0 text r1
    ld r1 r1 ; -> ld r1 r1 x
    cp r1 out ; -> cp r1 x out
    add r0 1 r0 ; -> imm2|add r0 1 r0
    jplt r0 text_len for_L1 ; -> imm2|add r0 text_len for_L1
    halt ; -> halt x x x

.entry start

.code
start:
    cp in r0
    cp in r1
    cp in r2
    add r0 1 r0
    add r1 1 r1
    add r2 1 r2
    cp r0 out
    cp r1 out
    cp r2 out
    halt
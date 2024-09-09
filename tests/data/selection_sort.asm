.data 0
data [14, 6, 3, 2, 1, 11, 15, 8, 13, 4, 12, 5, 7, 10, 9, 0] length

.entry start

.code
; args: n
f_sort:
    push r0
    fpop r0 ; n

    cp 0 r2 ; i
    loop1:
    ; ---------------- for (i = 0; i < n; ++i)
    cp r2 r4 ; min_idx = i;
    cp r2 r1 ; j = i;
    loop2:
    ; ------------- for (j = i; j < n; ++j)
    ld r1 r3 ; r3 <- arr[j]
    ld r4 r5 ; r5 <- arr[min_idx]
    ; if (arr[j] < arr[min_idx] min_idx = j;
    jpge r3 r5 if1
    cp r1 r4
    if1:
    ; -------------
    add r1 1 r1
    jplt r1 r0 loop2

    ; if (min_idx != i) swap(&arr[min_idx], &arr[i]);
    jpeq r4 r2 if2
    ld r4 r3
    ld r2 r5
    st r2 r3
    st r4 r5
    if2:

    ; ----------------
    add r2 1 r2
    jplt r2 r0 loop1

    pop r0
    ret

start:
    fpush length
    call f_sort
    halt


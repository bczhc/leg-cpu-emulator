.data 0
data [14, 6, 3, 2, 1, 11, 15, 8, 13, 4, 12, 5, 7, 10, 9, 0] length

.entry start

.code
; args: arr, n
f_sort:
    push r0
    push r1
    fpop r1 ; arr
    fpop r0 ; n

    cp 0 r2 ; i
    sub r0 1 r3 ; r3: n - 1
    cp 0 r4 ; min_idx
    loop1:
    ; ---------------- for (i = 0; i < n - 1; ++i)
    fpush r1
    fpush r0
    fpush r2
    fpush r4
    call f_sort_inner
    fpop r4 ; min_idx = sort_inner(...)

    ; if (min_idx != i) swap(&arr[min_idx], &arr[i]);
    jpeq r4 r2 if2
    push r1
    push r1
    add r1 r2 r1
    fpush r1
    pop r1
    add r1 r4 f1
    fpush r1
    call f_swap
    pop r1
    if2:
    ; ----------------
    jplt r2 r3 loop1
    add r2 1 r2

    pop r1
    pop r0
    ret

; args: min_idx, i, n, arr
; returns new_min_idx
f_sort_inner:
    push r0
    push r1
    push r2
    push r3
    fpop r0 ; min_idx
    fpop r1 ; i
    fpop r2 ; n
    fpop r4 ; arr

    add r1 1 r3 ; j
    loop2:
    ; ---------- for (j = i + 1; j < n; ++j)
    push r1 ; back up i
    push r2 ; back up n

    push r4 ; back up arr
    push r4 ; back up arr
    add r4 r3 r4
    ld r4 r1 ; r0 <- arr[j]
    pop r4
    add r4 r0 r4
    ld r4 r2 ; r2 <- arr[min_idx]
    pop r4

    jpge r0 r2 if1
    ; arr[j] < arr[min_idx]
    cp r3 r0 ; min_idx = j;
    if1:
    pop r2
    pop r1

    ; ----------
    jplt r3 r2 loop2
    add r3 1 r3

    fpush r0
    pop r3
    pop r2
    pop r1
    pop r0
    ret

; args: addr1, addr2
f_swap:
    push r1
    push r2
    fpop r1
    fpop r2
    ld r1 r3
    ld r2 r4
    st r2 r3
    st r1 r4
    pop r2
    pop r1
    ret

start:
    fpush length
    fpush data
    call f_sort
    halt


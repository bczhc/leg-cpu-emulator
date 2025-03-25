## Registers

| Name             | Description          |
| ---------------- | -------------------- |
| r0, r1, ..., r11 | Generic registers    |
| in               | Input                |
| out              | Output               |
| aor              | Always-one register  |
| azr              | Always-zero register |
| fss              | Function stack start |
|                  | Arithmetic carry bit |
|                  | Jump target address  |

For input and output, just `cp` from/to `in`/`out`.

## Instruction Categories

- All `src`s and `dst`s can be either immediate or registers.
- `halt` must be called after a program finishes.

### Arithmetic Operations

| Mnemonic | Format             | Description                    |
| -------- | ------------------ | ------------------------------ |
| add      | add src1 src2 dst  | Addition                       |
| sub      | sub src1 src2 dst  | Subtraction                    |
| and      | and src1 src2 dst  | Bitwise AND                    |
| or       | or src1 src2 dst   | Bitwise OR                     |
| not      | not src dst        | Bitwise NOT                    |
| xor      | xor src1 src2 dst  | Bitwise XOR                    |
| div      | div src1 src2 dst  | Division                       |
| mod      | mod src1 src2 dst  | Modulo                         |
| cadd     | cadd src1 src2 dst | Carry addition                 |
| anc      | anc src1 src2 dst  | Add with no carry bit set      |
| snc      | snc src1 src2 dst  | Subtract with no carry bit set |

- By default, `add` and `cadd` will set the carry bit on overflow, whereas `anc` won't.

### Shift Operations

| Mnemonic | Format             | Description           |
| -------- | ------------------ | --------------------- |
| shl      | shl src1 src2 dst  | (Logical) shift left  |
| shr      | shr src1 src2 dst  | (Logical) shift right |
| wshl     | wshl src1 src2 dst | Wrapping shift left   |
| wshr     | wshr src1 src2 dst | Wrapping shift right  |

### Control Flow

| Mnemonic | Format         | Description              |
| -------- | -------------- | ------------------------ |
| jp       | jp             | Unconditional jump       |
| jpeq     | jpeq src1 src2 | Jump if equal            |
| jpne     | jpne src1 src2 | Jump if not equal        |
| jplt     | jplt src1 src2 | Jump if less than        |
| jple     | jple src1 src2 | Jump if less or equal    |
| jpgt     | jpgt src1 src2 | Jump if greater than     |
| jpge     | jpge src1 src2 | Jump if greater or equal |
| jamv     | jamv label     | Jump address move        |

- Before any jump, the jump target address should be set via `jamv`. See example below.

  ```assembly
  infinite_loop:
  cp azr out
  jamv infinite_loop
  jp
  ```

### Memory Operations

| Mnemonic | Format      | Description      |
| -------- | ----------- | ---------------- |
| ld       | ld addr dst | Load from memory |
| st       | st addr src | Store to memory  |

- `addr` can be either immediate or a register.

### Stack Operations

| Mnemonic | Format   | Description    |
| -------- | -------- | -------------- |
| push     | push src | Push to stack  |
| pop      | pop dst  | Pop from stack |

### Function Calls

| Mnemonic | Format     | Description                                         |
| -------- | ---------- | --------------------------------------------------- |
| call     | call label | Call function                                       |
| ret      | ret        | Return from function                                |
| fpush    | fpush src  | Function push, used for parameters and return-value |
| fpop     | fpop dst   | Function pop, used for parameters and return-value  |

#### Examples

```assembly
; compute 5%3
; `fpush` has an reversed order
fpush 3
fpush 5
call f_mod
fpop r0 ; now r0 is 2

f_mod:
  fpop r0
  fpop r1
  mod r0 r1 r0
  fpush r0
  ret
```

### Miscellaneous

| Mnemonic | Format      | Description            |
| -------- | ----------- | ---------------------- |
| halt     | halt        | Halt execution         |
| cp       | cp src, dst | Copy value             |
| mvc      | mvc dst     | Move carry to register |
| nop      | nop         | No operation           |

## Assembly Format

```assembly
.data 0          ; Data section (starts at address 0) (should not indent)
; These data are stored in memory directly.
my_array [1, 2, 3] array_len  ; Define an array
message 'Hello' msg_len       ; Define a string

.consts               ; Constants section (should not indent)
MAX_VALUE 100     ; Define a constant

.entry main           ; Program entry point

.code                 ; Code section
; Code goes here
main:
  nop
  halt                ; must present
```

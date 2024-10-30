LEG-CPU-emulator
===

Emulator and assembler for my "LEG" Architecture CPU in the game "Turing Complete".

## Build

```shell
cargo build -r
```

## Run All Tests

```shell
cargo test
```

## CLI Usage:
<pre><u style="text-decoration-style:solid"><b>Usage:</b></u> <b>leg</b> [OPTIONS] &lt;SOURCE&gt;

<u style="text-decoration-style:solid"><b>Arguments:</b></u>
  &lt;SOURCE&gt;
          Path to the source file.
          
          The source file is of the two filename extensions: .asm/.bin

<u style="text-decoration-style:solid"><b>Options:</b></u>
  <b>-o</b>, <b>--output</b> &lt;OUTPUT&gt;
          Path to the output file.
          
          If no output file is specified, derive from the input file.

  <b>-t</b>, <b>--out-type</b> &lt;OUT_TYPE&gt;
          [possible values: commented-hex, binary]

  <b>-r</b>, <b>--run</b>
          Assemble and run

      <b>--stdout</b>
          Output to stdout

  <b>-i</b>, <b>--input</b> &lt;INPUT&gt;
          Path to the program input

      <b>--stdin</b>
          Read program input from stdin

  <b>-h</b>, <b>--help</b>
          Print help (see a summary with &apos;-h&apos;)</pre>

## Example

`hello_world.asm`:
```asm
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
```

```console
❯ leg hello_world.asm -r
hello, world
```
Or
```console
❯ leg hello_world.asm
❯ leg hello_world.bin
hello, world
```
For the code that can be used in Turing Complete, use:
```console
❯ leg hello_world.asm --stdout -t hex
0x01 0x0c 0x00 0x10 # copystatic
0x68 0x65 0x6c 0x6c 0x6f 0x2c 0x20 0x77 0x6f 0x72 0x6c 0x64 # data
# start:
0x83 0x00 0x00 0x00 # cp 0 r0
# for1:
0x48 0x00 0x00 0x01 # add r0 text r1
0x28 0x01 0x01 0x00 # ld r1 r1
0x03 0x01 0x00 0x0c # cp r1 out
0x48 0x00 0x01 0x00 # add r0 1 r0
0x44 0x00 0x14 0x00 # jamv for1
0x62 0x00 0x0c 0x00 # jplt r0 text_len
0x83 0x0a 0x00 0x0c # cp 0x0a out
0x02 0x00 0x00 0x00 # halt
```

[`water_world.asm`](https://github.com/bczhc/leg-cpu-emulator/blob/master/tests/asm/water_world.asm):
```console
❯ echo '4,6,1,4,6,5,1,4,1,2,6,5,6,1,4,2' | target/debug/leg tests/asm/water_world.asm -r --stdin
28
❯ echo '5,6,2,5,1,3,2,1,1,1,1,1,1,1,1,1' | target/debug/leg tests/asm/water_world.asm -r --stdin
5
```

More examples are under [tests](https://github.com/bczhc/leg-cpu-emulator/tree/master/tests).

## WebUI

### Dev-Run

```bash
cd web-ui
wasm-pack build
cd web && npm run dev
```

### Build

```bash
cd web-ui
wasm-pack build
cd web && npm run build
# now files to serve are under dist/
```

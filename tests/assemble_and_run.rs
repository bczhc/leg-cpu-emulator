#![feature(decl_macro)]

use leg_cpu_emulator::assembler::Assembler;
use leg_cpu_emulator::emulator::Emulator;

macro test_asm($name:literal) {
    include_str!(concat!("../tests/asm/", $name, ".asm"))
}

fn emulator_run(bin: impl Into<Vec<u8>>) -> (Emulator, Vec<u8>) {
    let mut emulator = Emulator::new(bin).unwrap();
    let mut output = Vec::new();
    loop {
        // println!("PC: {}", emulator.pc.usize());
        emulator.tick().unwrap();
        if emulator.halted {
            break;
        }
        if let Some(x) = emulator.output {
            output.push(*x);
        }
    }
    (emulator, output)
}

fn assemble_binary(code: &str) -> Vec<u8> {
    Assembler::new(code).unwrap().assemble().binary.merge()
}

fn assemble_and_run(code: &str) -> (Emulator, Vec<u8>) {
    let target = Assembler::new(code).unwrap().assemble();
    println!("{}", target.commented_binary);
    emulator_run(target.binary.merge())
}

#[test]
fn asm_hello_world() {
    let code = test_asm!("hello_world");
    let assembler = Assembler::new(code).unwrap();
    let target = assembler.assemble();
    println!("{}", target.commented_binary);
    let output = emulator_run(target.binary.merge()).1;
    assert_eq!(&output, b"hello, world\n");
}

#[test]
fn asm_fibonacci() {
    let code = test_asm!("fibonacci");
    let target = Assembler::new(code).unwrap().assemble();
    println!("{}", target.commented_binary);
    let emulator = emulator_run(target.binary.merge()).0;
    assert_eq!(&emulator.ram[..10], &[1, 1, 2, 3, 5, 8, 13, 21, 34, 55]);
}

#[test]
fn asm_selection_sort() {
    let ram = assemble_and_run(test_asm!("selection_sort")).0.ram;
    assert_eq!(&ram[..16], &(0..16).collect::<Vec<u8>>())
}

#[test]
fn sixteen_bits_addressing() {
    let output = assemble_and_run(test_asm!("16bit_addressing")).1[0];
    assert_eq!(output, 5 /* 2 + 3 */);
}

#[test]
fn multibyte_integer_add() {
    let target = Assembler::new(test_asm!("multibyte-integer-adding"))
        .unwrap()
        .assemble();
    let status = emulator_run(target.binary.merge()).1[0];
    assert_eq!(status, 0);
}

#[test]
fn function_stack() {
    let ram = assemble_and_run(test_asm!("function_stack")).0.ram;
    assert_eq!(&ram[0..5], &[6, 7, 8, 9, 10]);
}

#[test]
fn input_output() {
    let input = vec![0, 1, 2];
    let binary = assemble_binary(test_asm!("input_output"));
    let output = Emulator::new(binary)
        .unwrap()
        .set_input(input)
        .run_to_halt()
        .unwrap();
    assert_eq!(&output, &[1, 2, 3]);
}

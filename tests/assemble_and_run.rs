#![feature(decl_macro)]

use leg_cpu_emulator::assembler::Assembler;
use leg_cpu_emulator::emulator::Emulator;

macro test_asm($name:literal) {
    include_str!(concat!("../tests/data/", $name, ".asm"))
}

fn emulator_run(bin: impl Into<Vec<u8>>) -> (Emulator, Vec<u8>) {
    let mut emulator = Emulator::new(bin).unwrap();
    let mut output = Vec::new();
    loop {
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
    let code = test_asm!("selection_sort");
    let assembler = Assembler::new(code).unwrap();
    let target = assembler.assemble();
    println!("{}", target.commented_binary);
}

#[test]
fn sixteen_bits_addressing() {
    let code = test_asm!("16bit_addressing");
    let assembler = Assembler::new(code).unwrap();
    println!("{}", assembler.assemble().commented_binary);
}

fn assemble_to_commented_binary(code: &str) -> String {
    Assembler::new(code).unwrap().assemble().commented_binary
}

#[test]
fn multibyte_integer_add() {
    println!(
        "{}",
        assemble_to_commented_binary(test_asm!("multibyte-integer-adding"))
    );
}

#[test]
fn function_stack() {
    println!(
        "{}",
        assemble_to_commented_binary(test_asm!("function_stack"))
    );
}

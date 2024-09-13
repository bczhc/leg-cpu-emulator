use leg_cpu_emulator::assembler::Assembler;
use leg_cpu_emulator::emulator::Emulator;

#[test]
fn fibonacci() {
    let target = Assembler::new(include_str!("../tests/data/fibonacci.asm"))
        .unwrap()
        .assemble();

    let binary = target.binary.merge();
    let mut emulator = Emulator::new(binary).unwrap();

    loop {
        emulator.tick().unwrap();
        if emulator.halted {
            break;
        }
    }

    assert_eq!(&emulator.ram[..10], &[1, 1, 2, 3, 5, 8, 13, 21, 34, 55]);
}
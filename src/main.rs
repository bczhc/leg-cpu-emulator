use leg_cpu_emulator::assembler::Assembler;
use leg_cpu_emulator::emulator::Emulator;

fn main() {
    let target = Assembler::new(include_str!("../tests/data/fibonacci.asm"))
        .unwrap()
        .assemble();
    println!("{}", target.commented_binary);

    let binary = target.binary.merge();
    let mut emulator = Emulator::new(binary).unwrap();
    
    loop {
        emulator.tick().unwrap();
        if emulator.halted {
            break;
        }
    }

    for x in emulator.ram.iter().take(10) {
        println!("{}", x);
    }
}

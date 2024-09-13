use leg_cpu_emulator::assembler::Assembler;
use leg_cpu_emulator::emulator::Emulator;

fn main() {
    let target = Assembler::new(include_str!("../tests/data/hello_world.asm"))
        .unwrap()
        .assemble();

    let binary = target.binary.merge();
    let mut emulator = Emulator::new(binary).unwrap();
    
    loop {
        emulator.tick().unwrap();
        if let Some(o) = emulator.output {
            print!("{}", char::from(*o));
        }
        if emulator.halted {
            break;
        }
    }
}

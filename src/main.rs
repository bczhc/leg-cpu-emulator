use leg_cpu_emulator::asm::Assembler;

fn main() {
    println!(
        "{}",
        Assembler::new(include_str!("../tests/data/1.asm"))
            .unwrap()
            .assemble()
            .commented_binary
    );
}

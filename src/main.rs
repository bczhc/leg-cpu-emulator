use leg_cpu_emulator::assembler::Assembler;

fn main() {
    println!(
        "{}",
        Assembler::new(include_str!("../tests/data/multibyte-integer-adding.asm"))
            .unwrap()
            .assemble()
            .commented_binary
    );
}

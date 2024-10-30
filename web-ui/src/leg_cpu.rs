use crate::errors::AnyhowExt;
use leg_cpu_emulator as leg;
use wasm_bindgen::prelude::wasm_bindgen;


#[wasm_bindgen]
pub struct LegCpu;

#[derive(Clone)]
#[wasm_bindgen(getter_with_clone)]
pub struct LegAssemblyTarget {
    pub binary: Vec<u8>,
    pub commented_binary: String,
}

#[derive(Clone)]
#[wasm_bindgen(getter_with_clone)]
pub struct LegEmulationResult {
    pub interrupted: bool,
    pub output: Vec<u8>,
    pub output_lossy_string: String,
    pub cpu_cycles: u64,
    pub ram: Vec<u8>,
    pub ram_pretty_hex: String,
}

#[wasm_bindgen]
impl LegCpu {
    pub fn assemble(code: &str) -> crate::Result<LegAssemblyTarget> {
        let result: anyhow::Result<_> = try {
            let assembler = leg::assembler::Assembler::new(code)?;
            let target = assembler.assemble();
            LegAssemblyTarget {
                binary: {
                    let mut joined = target.binary.header;
                    joined.extend_from_slice(&target.binary.code);
                    joined
                },
                commented_binary: target.commented_binary
            }
        };
        result.map_err_string()
    }

    pub fn emulate(binary: &[u8], input: &str, cycles_limit: Option<u64>) -> crate::Result<LegEmulationResult> {
        let cycles_limit = cycles_limit.unwrap_or(u64::MAX);
        let result: anyhow::Result<_> = try {
            let mut emulator = leg::emulator::Emulator::new(binary)?;
            // always append a new line
            let mut input = input.as_bytes().to_vec();
            input.push(b'\n');
            emulator.set_input(input);
            let mut cycle = 0_u64;
            let mut output = Vec::new();
            let mut interrupted = false;
            loop {
                emulator.tick()?;
                cycle += 1;
                if emulator.halted {
                    break;
                }
                if cycle > cycles_limit {
                    interrupted = true;
                    break;
                }
                if let Some(o) = &emulator.output {
                    output.push(**o);
                }
            }
            let output_lossy_string = String::from_utf8_lossy(&output).to_string();
            let ram_pretty_hex = pretty_hex::pretty_hex(&emulator.ram);
            LegEmulationResult {
                output_lossy_string,
                output,
                cpu_cycles: cycle,
                interrupted,
                ram: emulator.ram,
                ram_pretty_hex,
            }
        };
        result.map_err_string()
    }
}

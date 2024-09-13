#![feature(yeet_expr)]

use anyhow::anyhow;
use clap::Parser;
use leg_cpu_emulator::assembler::Assembler;
use leg_cpu_emulator::emulator::Emulator;
use std::fs::File;
use std::io::{stdout, Read, Write};
use std::path::PathBuf;
use yeet_ops::yeet;

#[derive(clap::Parser)]
struct Args {
    /// Path to the input file.
    ///
    /// The input file is of the two filename extensions: .asm/.bin
    input: PathBuf,
    /// Path to the output file.
    ///
    /// If no output file is specified, derive from the input file.
    #[arg(short, long)]
    output: Option<PathBuf>,
    #[arg(short = 't', long)]
    out_type: Option<OutputType>,
    /// Assemble and run.
    #[arg(short, long)]
    run: bool,
    /// Output to stdout
    #[arg(long)]
    stdout: bool,
}

#[derive(clap::ValueEnum, Clone, Debug, Copy)]
enum OutputType {
    #[value(alias = "hex")]
    CommentedHex,
    #[value(alias = "bin")]
    Binary,
}

impl Default for OutputType {
    fn default() -> Self {
        Self::Binary
    }
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let mut input_file = File::open(&args.input)?;

    match args
        .input
        .extension()
        .and_then(|x| x.to_str())
        .map(|x| x.to_lowercase())
        .as_deref()
    {
        Some("asm") => {
            let mut code = String::new();
            input_file.read_to_string(&mut code)?;
            let out_file = match args.output {
                Some(x) => x,
                None => match args.out_type.unwrap_or_default() {
                    OutputType::CommentedHex => {
                        let mut path = args.input.clone();
                        path.set_extension("txt");
                        path
                    }
                    OutputType::Binary => {
                        let mut path = args.input.clone();
                        path.set_extension("bin");
                        path
                    }
                },
            };

            let out: &mut dyn Write = if args.stdout {
                &mut stdout()
            } else {
                &mut File::create(&out_file)?
            };

            let target = Assembler::new(code)?.assemble();
            match args.out_type.unwrap_or_default() {
                OutputType::CommentedHex => {
                    out.write_all(target.commented_binary.as_bytes())?;
                }
                OutputType::Binary => {
                    out.write_all(&target.binary.merge())?;
                }
            }

            if args.run {
                let output = Emulator::new(target.binary.merge())?.run_to_halt()?;
                print_output(&output);
            }
        }
        Some("bin") => {
            // execute the program
            let mut bin = Vec::new();
            input_file.read_to_end(&mut bin)?;
            let output = Emulator::new(bin)?.run_to_halt()?;
            print_output(&output);
        }
        _ => yeet!(anyhow!(
            "Cannot determine input file type from the name extension"
        )),
    };
    Ok(())
}

fn print_output(output: &[u8]) {
    for &x in output {
        stdout().write_all(&[x]).unwrap();
    }
}

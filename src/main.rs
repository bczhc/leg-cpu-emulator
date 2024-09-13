#![feature(yeet_expr)]

use clap::Parser;
use leg_cpu_emulator::assembler::Assembler;
use leg_cpu_emulator::emulator::Emulator;
use std::fs::File;
use std::io;
use std::io::{stdin, stdout, Read, Write};
use std::path::PathBuf;
use yeet_ops::yeet;

#[derive(clap::Parser)]
struct Args {
    /// Path to the source file.
    ///
    /// The source file is of the two filename extensions: .asm/.bin
    source: PathBuf,
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
    /// Path to the program input.
    #[arg(short, long)]
    input: Option<PathBuf>,
    /// Read program input from stdin.
    #[arg(long)]
    stdin: bool,
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
    let mut source_file = File::open(&args.source)?;

    let program_in = if args.stdin {
        read_to_vec(stdin())?
    } else {
        match args.input {
            None => {
                vec![]
            }
            Some(path) => read_to_vec(File::open(path)?)?,
        }
    };

    match args
        .source
        .extension()
        .and_then(|x| x.to_str())
        .map(|x| x.to_lowercase())
        .as_deref()
    {
        Some("asm") => {
            let mut code = String::new();
            source_file.read_to_string(&mut code)?;
            let out_file = match args.output {
                Some(x) => x,
                None => match args.out_type.unwrap_or_default() {
                    OutputType::CommentedHex => {
                        let mut path = args.source.clone();
                        path.set_extension("txt");
                        path
                    }
                    OutputType::Binary => {
                        let mut path = args.source.clone();
                        path.set_extension("bin");
                        path
                    }
                },
            };

            let target = Assembler::new(code)?.assemble();

            if args.run {
                // transparent-run mode. do not write to file
                let output = Emulator::new(target.binary.merge())?
                    .set_input(program_in)
                    .run_to_halt()?;
                print_output(&output);
            } else {
                let out: &mut dyn Write = if args.stdout {
                    &mut stdout()
                } else {
                    &mut File::create(&out_file)?
                };
                match args.out_type.unwrap_or_default() {
                    OutputType::CommentedHex => {
                        out.write_all(target.commented_binary.as_bytes())?;
                    }
                    OutputType::Binary => {
                        out.write_all(&target.binary.merge())?;
                    }
                }
            }
        }
        Some("bin") => {
            // execute the program
            let mut bin = Vec::new();
            source_file.read_to_end(&mut bin)?;
            let output = Emulator::new(bin)?.set_input(program_in).run_to_halt()?;
            print_output(&output);
        }
        _ => yeet!(anyhow::anyhow!(
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

fn read_to_vec(mut reader: impl Read) -> io::Result<Vec<u8>> {
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf)?;
    Ok(buf)
}

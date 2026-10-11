use std::io::{self, BufWriter, Write};
use std::process::ExitCode;

use apg::{generate, help_text, parse_args, Command};

fn main() -> ExitCode {
    match parse_args(std::env::args().skip(1)) {
        Ok(Command::Help) => {
            println!("{}", help_text());
            ExitCode::SUCCESS
        }
        Ok(Command::Version) => {
            println!("apg {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        Ok(Command::Generate(cfg)) => {
            let stdout = io::stdout();
            let mut out = BufWriter::new(stdout.lock());
            for _ in 0..cfg.count {
                if writeln!(out, "{}", generate(cfg.length, &cfg.groups)).is_err() {
                    return ExitCode::FAILURE;
                }
            }
            let _ = out.flush();
            ExitCode::SUCCESS
        }
        Err(msg) => {
            eprintln!("Error: {msg}");
            eprintln!("Run 'apg --help' for usage.");
            ExitCode::from(2)
        }
    }
}

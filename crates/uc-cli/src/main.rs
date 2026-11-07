//! uc: command-line tool that shows the contents of Unreal Engine files.

mod dump;

use std::path::Path;
use std::process::ExitCode;

const USAGE: &str = "Usage: uc --version\n       uc dump <file>";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    match args.as_slice() {
        ["--version"] => {
            println!("uc {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        ["dump", file] => match dump::run(Path::new(file)) {
            Ok(text) => {
                print!("{text}");
                ExitCode::SUCCESS
            }
            Err(message) => {
                eprintln!("error: {message}");
                ExitCode::FAILURE
            }
        },
        _ => {
            eprintln!("{USAGE}");
            ExitCode::FAILURE
        }
    }
}

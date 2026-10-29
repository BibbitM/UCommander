//! Build automation for UCommander. The commands come in plan point P3.

use std::process::ExitCode;

fn main() -> ExitCode {
    eprintln!("xtask: no commands yet");
    ExitCode::FAILURE
}

#[cfg(test)]
mod tests {
    #[test]
    fn package_name_is_xtask() {
        assert_eq!(env!("CARGO_PKG_NAME"), "xtask");
    }
}

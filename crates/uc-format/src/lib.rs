//! Parser for Unreal Engine package and container files.

mod error;
pub mod package;
mod reader;
mod types;

pub use error::Error;
pub use types::{Guid, IoHash};

#[cfg(test)]
mod tests {
    #[test]
    fn package_name_is_uc_format() {
        assert_eq!(env!("CARGO_PKG_NAME"), "uc-format");
    }
}

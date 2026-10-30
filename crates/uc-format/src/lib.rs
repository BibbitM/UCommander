//! Parser for Unreal Engine package and container files.

pub mod package;

#[cfg(test)]
mod tests {
    #[test]
    fn package_name_is_uc_format() {
        assert_eq!(env!("CARGO_PKG_NAME"), "uc-format");
    }
}

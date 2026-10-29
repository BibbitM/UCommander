//! Types and safe FFI helpers for the Total Commander plugin API.

pub mod buffers;
pub mod content;

#[cfg(test)]
mod tests {
    #[test]
    fn package_name_is_uc_tc() {
        assert_eq!(env!("CARGO_PKG_NAME"), "uc-tc");
    }
}

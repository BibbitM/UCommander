//! UCommanderPacker: Total Commander packer plugin (WCX) for Unreal Engine files.

#[cfg(test)]
mod tests {
    #[test]
    fn package_name_is_uc_packer() {
        assert_eq!(env!("CARGO_PKG_NAME"), "uc-packer");
    }
}

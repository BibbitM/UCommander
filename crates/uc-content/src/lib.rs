//! UCommanderContent: Total Commander content plugin (WDX) for Unreal Engine files.

#[cfg(test)]
mod tests {
    #[test]
    fn package_name_is_uc_content() {
        assert_eq!(env!("CARGO_PKG_NAME"), "uc-content");
    }
}

//! UCommanderLister: Total Commander lister plugin (WLX) for Unreal Engine files.

#[cfg(test)]
mod tests {
    #[test]
    fn package_name_is_uc_lister() {
        assert_eq!(env!("CARGO_PKG_NAME"), "uc-lister");
    }
}

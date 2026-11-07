//! The main asset of a package and its class. The rules are in
//! `docs/formats/package.md`, "Main asset".

use super::header::{PackageHeader, PackageIndex};
use super::summary::PKG_FILTER_EDITOR_ONLY;

/// Package flag `ContainsNoAsset`.
const PKG_CONTAINS_NO_ASSET: u32 = 0x0000_0400;
/// The asset registry filter skips this class and its subclasses in an
/// editor package.
const EDITOR_SKIPPED_CLASS: &str = "BlueprintGeneratedClass";
/// The asset registry filter skips this class and its subclasses in a
/// cooked package.
const COOKED_SKIPPED_CLASS: &str = "Blueprint";

impl PackageHeader {
    /// Gives the export of the main asset: the first asset with the short
    /// package name, or else the first asset in the export table. Assets
    /// that the asset registry filter skips are ignored. `None` when the
    /// package has no asset.
    pub fn main_asset(&self) -> Option<PackageIndex> {
        if self.summary.package_flags & PKG_CONTAINS_NO_ASSET != 0 {
            return None;
        }
        let package_name = &self.summary.package_name;
        // `rsplit` always gives at least one part.
        let short_name = package_name.rsplit('/').next().unwrap_or(package_name);
        let mut first_asset = None;
        for (position, export) in self.exports.iter().enumerate() {
            // The export count is an `i32`, so the index fits.
            let index = PackageIndex(i32::try_from(position + 1).ok()?);
            if export.is_asset != Some(true) || self.is_skipped(index) {
                continue;
            }
            let has_main_name = export.outer_index == PackageIndex(0)
                && self
                    .object_name(index)
                    .is_some_and(|name| name.eq_ignore_ascii_case(short_name));
            if has_main_name {
                return Some(index);
            }
            first_asset.get_or_insert(index);
        }
        first_asset
    }

    /// Gives the class name of the main asset. `None` when the package has
    /// no asset.
    pub fn asset_class(&self) -> Option<String> {
        self.class_name(self.main_asset()?)
    }

    /// The asset registry filter. The file has only the class name, so a
    /// class name that ends with the skipped class name counts as a
    /// subclass.
    fn is_skipped(&self, index: PackageIndex) -> bool {
        let skipped = if self.summary.package_flags & PKG_FILTER_EDITOR_ONLY == 0 {
            EDITOR_SKIPPED_CLASS
        } else {
            COOKED_SKIPPED_CLASS
        };
        self.class_name(index)
            .is_some_and(|class| class.ends_with(skipped))
    }
}

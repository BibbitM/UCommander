//! Unreal Engine packages (`.uasset`, `.umap`).

mod header;
mod objects;
mod summary;
mod version;

pub use header::{
    ExportDependencies, HeaderRead, NameEntry, NameRef, ObjectExport, ObjectImport, PackageHeader,
    PackageIndex, read_header,
};
pub use objects::{IndexTarget, MAX_OBJECT_PATH_LENGTH};
pub use summary::{
    CustomVersion, EngineVersion, Generation, PackageSummary, SummaryRead, Table, Warning,
    read_summary,
};

/// The first 4 bytes of a package file: `0x9E2A83C1`, stored little-endian.
pub const PACKAGE_FILE_TAG: u32 = 0x9E2A83C1;

/// Returns `true` when `bytes` starts with [`PACKAGE_FILE_TAG`].
///
/// The byte-swapped tag (packages from big-endian platforms) gives `false`.
pub fn has_package_file_tag(bytes: &[u8]) -> bool {
    match bytes.first_chunk::<4>() {
        Some(tag) => u32::from_le_bytes(*tag) == PACKAGE_FILE_TAG,
        None => false,
    }
}

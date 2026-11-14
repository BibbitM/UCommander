//! Asset registry data of a package: the assets and their tags. The
//! layout is in `docs/formats/package.md`, "Asset registry data".

use super::summary::{PKG_FILTER_EDITOR_ONLY, PackageSummary};
use super::version::ue4;
use crate::error::Error;
use crate::reader::Reader;

/// Smallest size of one object: two string lengths and the tag count.
const MIN_OBJECT_SIZE: usize = 12;
/// Smallest size of one tag: two string lengths.
const MIN_TAG_SIZE: usize = 8;

/// The asset registry data of a package.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct AssetRegistryData {
    /// UE4 `ASSETREGISTRY_DEPENDENCYFLAGS` and later, without the package
    /// flag `FilterEditorOnly`.
    pub dependency_data_offset: Option<i64>,
    pub objects: Vec<AssetRegistryObject>,
}

/// One object (asset) of the asset registry data.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct AssetRegistryObject {
    /// The path as in the file: relative to the package (for example
    /// `L_Level`), or a full path that starts with `/`.
    pub object_path: String,
    pub class_name: String,
    /// The tags in file order.
    pub tags: Vec<AssetRegistryTag>,
}

/// One tag of an asset registry object.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct AssetRegistryTag {
    pub key: String,
    pub value: String,
}

/// Reads the asset registry data of a package from the first bytes of
/// the package file.
///
/// The section starts at the asset registry data offset of `summary`. An
/// offset of 0 or less gives no data, as in UE. The function does no
/// I/O. If `bytes` ends before the section ends, the error is
/// [`Error::Truncated`].
pub fn read_asset_registry(
    bytes: &[u8],
    summary: &PackageSummary,
) -> Result<AssetRegistryData, Error> {
    let Ok(offset) = usize::try_from(summary.asset_registry_data_offset) else {
        return Ok(AssetRegistryData::default());
    };
    if offset == 0 {
        return Ok(AssetRegistryData::default());
    }
    let mut r = Reader::at(bytes, offset);
    let has_dependency_offset = summary.file_version_ue4 >= ue4::ASSETREGISTRY_DEPENDENCYFLAGS
        && summary.package_flags & PKG_FILTER_EDITOR_ONLY == 0;
    let dependency_data_offset = has_dependency_offset.then(|| r.i64()).transpose()?;
    let count = r.array_count(MIN_OBJECT_SIZE, "asset registry object count")?;
    // No capacity from the counts: an object or a tag in memory is larger
    // than its smallest size in the file.
    let mut objects = Vec::new();
    for _ in 0..count {
        let object_path = r.string("asset registry object path")?;
        let class_name = r.string("asset registry class name")?;
        let tag_count = r.array_count(MIN_TAG_SIZE, "asset registry tag count")?;
        let mut tags = Vec::new();
        for _ in 0..tag_count {
            let key = r.string("asset registry tag key")?;
            let value = r.string("asset registry tag value")?;
            tags.push(AssetRegistryTag { key, value });
        }
        objects.push(AssetRegistryObject {
            object_path,
            class_name,
            tags,
        });
    }
    Ok(AssetRegistryData {
        dependency_data_offset,
        objects,
    })
}

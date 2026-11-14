use super::summary::PKG_FILTER_EDITOR_ONLY;
use super::{PackageHeader, read_summary};
use crate::Error;
use crate::reader::Reader;

impl PackageHeader {
    /// Gives the immediate super class of a top-level generated class.
    ///
    /// Call this on the Blueprint package with the actor's class path.
    pub fn blueprint_parent_class(&self, actor_class_path: &str) -> Option<String> {
        let (package, class) = actor_class_path.split_once('.')?;
        if !self.long_package_name()?.eq_ignore_ascii_case(package) {
            return None;
        }
        let export = self.exports.iter().find(|export| {
            export.outer_index.0 == 0
                && self
                    .name_text(export.object_name)
                    .is_some_and(|name| name.eq_ignore_ascii_case(class))
        })?;
        self.object_path(export.super_index)
    }
}

/// The actor asset registry record and its encoded descriptor.
#[derive(Debug, PartialEq, Eq)]
pub struct ActorMetadata {
    pub object_path: String,
    pub class_path: String,
    pub native_class_path: String,
    pub descriptor_base64: String,
}

/// Finds actor metadata in a standard external actor package.
///
/// Pass the package bytes from offset zero through the asset registry
/// records. Dependency data and object data can be omitted.
pub fn read_actor_metadata(bytes: &[u8]) -> Result<Option<ActorMetadata>, Error> {
    let read = read_summary(bytes)?;
    let summary = read.summary;
    let folder = summary
        .package_name
        .strip_prefix('/')
        .and_then(|path| path.split('/').nth(1));
    if !folder.is_some_and(|name| name.eq_ignore_ascii_case("__ExternalActors__"))
        || summary.package_flags & PKG_FILTER_EDITOR_ONLY != 0
        || summary.asset_registry_data_offset == 0
    {
        return Ok(None);
    }

    let header_end = usize::try_from(summary.total_header_size).unwrap_or_default();
    let start = usize::try_from(summary.asset_registry_data_offset)
        .ok()
        .filter(|&offset| offset >= read.size && offset + 12 <= header_end)
        .ok_or(Error::BadValue {
            offset: read.size,
            field: "asset registry data offset",
        })?;
    let end = usize::try_from(Reader::at(bytes, start).i64()?)
        .ok()
        .filter(|&offset| (start + 12..=header_end).contains(&offset))
        .ok_or(Error::BadValue {
            offset: start,
            field: "asset registry dependency data offset",
        })?;
    let data = bytes.get(..end).ok_or(Error::Truncated { offset: start })?;
    let mut reader = Reader::at(data, start + 8);
    let count = reader.array_count(12, "asset registry record count")?;
    let mut actor = None;
    for _ in 0..count {
        let object_path = reader.string("asset registry object path")?;
        let class_path = reader.string("asset registry class path")?;
        let tag_count = reader.array_count(8, "asset registry tag count")?;
        let mut native_class_path = None;
        let mut descriptor_base64 = None;
        for _ in 0..tag_count {
            let key = reader.string("asset registry tag key")?;
            let value = reader.string("asset registry tag value")?;
            if key.eq_ignore_ascii_case("ActorMetaDataClass") {
                native_class_path = Some(value);
            } else if key.eq_ignore_ascii_case("ActorMetaData") {
                descriptor_base64 = Some(value);
            }
        }
        if let (Some(native_class_path), Some(descriptor_base64)) =
            (native_class_path, descriptor_base64)
        {
            if actor.is_some() {
                return Err(Error::BadValue {
                    offset: reader.offset(),
                    field: "multiple external actor records",
                });
            }
            actor = Some(ActorMetadata {
                object_path,
                class_path,
                native_class_path,
                descriptor_base64,
            });
        }
    }
    Ok(actor)
}

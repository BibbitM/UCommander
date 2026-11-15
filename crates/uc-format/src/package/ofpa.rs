use super::summary::PKG_FILTER_EDITOR_ONLY;
use super::{PackageHeader, Warning, read_summary};
use crate::Error;
use crate::reader::Reader;
use crate::types::Guid;

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
    pub label: Option<String>,
    pub warnings: Vec<Warning>,
    object_path_offset: usize,
    descriptor_offset: usize,
}

/// A condition that does not prevent reading the asset registry fields.
#[derive(Debug, PartialEq, Eq)]
pub enum OfpaWarning {
    Package(Warning),
    UnknownDescriptorVersion(Guid),
    UnknownDescriptorLayout,
}

/// The data for one actor in a standard external actor package.
#[derive(Debug, PartialEq, Eq)]
pub struct OfpaActor {
    pub label: Option<String>,
    pub class_path: String,
    pub native_class_path: String,
    pub outer_map_package_path: String,
    /// Absent when the descriptor layout cannot be identified.
    pub guid: Option<Guid>,
    pub warnings: Vec<OfpaWarning>,
}

/// Reads an external actor from the package header and asset registry records.
/// Use the actor class path with [`PackageHeader::blueprint_parent_class`]
/// on the Blueprint package to obtain its immediate parent.
///
/// Errors use package offsets. A damaged descriptor is reported at the
/// start of its tag value.
pub fn read_ofpa_actor(bytes: &[u8]) -> Result<Option<OfpaActor>, Error> {
    let Some(metadata) = read_actor_metadata(bytes)? else {
        return Ok(None);
    };
    let (map, _) = metadata
        .object_path
        .split_once('.')
        .filter(|(map, _)| map.starts_with('/'))
        .ok_or(Error::BadValue {
            offset: metadata.object_path_offset,
            field: "external actor object path",
        })?;
    let descriptor = decode_base64(&metadata.descriptor_base64).ok_or(Error::BadValue {
        offset: metadata.descriptor_offset,
        field: "actor descriptor Base64",
    })?;
    let mut warnings: Vec<_> = metadata
        .warnings
        .into_iter()
        .map(OfpaWarning::Package)
        .collect();
    let guid = descriptor_guid(&descriptor, &mut warnings).map_err(|error| Error::BadValue {
        offset: metadata.descriptor_offset,
        field: match error {
            Error::BadValue { field, .. } => field,
            _ => "actor descriptor",
        },
    })?;
    Ok(Some(OfpaActor {
        label: metadata.label,
        class_path: metadata.class_path,
        native_class_path: metadata.native_class_path,
        outer_map_package_path: map.to_owned(),
        guid,
        warnings,
    }))
}

fn descriptor_guid(bytes: &[u8], warnings: &mut Vec<OfpaWarning>) -> Result<Option<Guid>, Error> {
    // These three streams select the fields before the GUID.
    const PREFIX_VERSIONS: [(Guid, std::ops::RangeInclusive<i32>); 3] = [
        (
            Guid {
                a: 0x697D_D581,
                b: 0xE64F_41AB,
                c: 0xAA4A_51EC,
                d: 0xBEB7_B628,
            },
            88..=123,
        ),
        (
            Guid {
                a: 0x5B4C_06B7,
                b: 0x2463_4AF8,
                c: 0x805B_BF70,
                d: 0xCDF5_D0DD,
            },
            5..=15,
        ),
        (
            Guid {
                a: 0x601D_1886,
                b: 0xAC64_4F84,
                c: 0xAA16_D3DE,
                d: 0x0DEA_C7D6,
            },
            88..=268,
        ),
    ];
    const UE5_RELEASE: Guid = Guid {
        a: 0xD89B_5E42,
        b: 0x24BD_4D46,
        c: 0x8412_ACA8,
        d: 0xDF64_1779,
    };
    let mut reader = Reader::new(bytes);
    let count = reader.array_count(20, "actor descriptor custom version count")?;
    let mut versions = [None; 3];
    for _ in 0..count {
        let id = reader.guid()?;
        let version = reader.i32()?;
        match PREFIX_VERSIONS.iter().position(|(guid, _)| *guid == id) {
            Some(index) => {
                if versions[index].replace(version).is_some() {
                    return Err(Error::BadValue {
                        offset: reader.offset() - 20,
                        field: "duplicate actor descriptor custom version",
                    });
                }
            }
            None if id != UE5_RELEASE => warnings.push(OfpaWarning::UnknownDescriptorVersion(id)),
            None => {}
        }
    }
    if !versions
        .into_iter()
        .zip(PREFIX_VERSIONS)
        .all(|(version, (_, supported))| version.is_some_and(|v| supported.contains(&v)))
    {
        warnings.push(OfpaWarning::UnknownDescriptorLayout);
        return Ok(None);
    }
    let offset = reader.offset();
    if reader.u32()? != 0 {
        return Err(Error::BadValue {
            offset,
            field: "default actor descriptor flag",
        });
    }
    for _ in 0..4 {
        reader.string("actor descriptor class path")?;
    }
    reader.guid().map(Some)
}

fn decode_base64(text: &str) -> Option<Vec<u8>> {
    let (groups, rest) = text.as_bytes().as_chunks::<4>();
    if !rest.is_empty() {
        return None;
    }
    let mut bytes = Vec::with_capacity(groups.len() * 3);
    for (index, &[a, b, c, d]) in groups.iter().enumerate() {
        let digit = |value| match value {
            b'A'..=b'Z' => Some(value - b'A'),
            b'a'..=b'z' => Some(value - b'a' + 26),
            b'0'..=b'9' => Some(value - b'0' + 52),
            b'+' => Some(62),
            b'/' => Some(63),
            _ => None,
        };
        let (a, b) = (digit(a)?, digit(b)?);
        bytes.push(a << 2 | b >> 4);
        if index + 1 == groups.len() && c == b'=' && d == b'=' && b & 15 == 0 {
            break;
        }
        let c = digit(c)?;
        bytes.push(b << 4 | c >> 2);
        if index + 1 == groups.len() && d == b'=' && c & 3 == 0 {
            break;
        }
        bytes.push(c << 6 | digit(d)?);
    }
    Some(bytes)
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
        let object_path_offset = reader.offset();
        let object_path = reader.string("asset registry object path")?;
        let class_path = reader.string("asset registry class path")?;
        let tag_count = reader.array_count(8, "asset registry tag count")?;
        let mut native_class_path = None;
        let mut descriptor_base64 = None;
        let mut label = None;
        for _ in 0..tag_count {
            let key = reader.string("asset registry tag key")?;
            let value_offset = reader.offset();
            let value = reader.string("asset registry tag value")?;
            if key.eq_ignore_ascii_case("ActorMetaDataClass") {
                native_class_path = Some(value);
            } else if key.eq_ignore_ascii_case("ActorMetaData") {
                descriptor_base64 = Some((value, value_offset));
            } else if key.eq_ignore_ascii_case("ActorLabel") {
                label = Some(value);
            }
        }
        if let (Some(native_class_path), Some((descriptor_base64, descriptor_offset))) =
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
                label,
                warnings: Vec::new(),
                object_path_offset,
                descriptor_offset,
            });
        }
    }
    if reader.offset() != end {
        return Err(Error::BadValue {
            offset: start + 8,
            field: "asset registry record count",
        });
    }
    if let Some(actor) = &mut actor {
        actor.warnings = read.warnings;
    }
    Ok(actor)
}

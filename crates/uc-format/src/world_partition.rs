//! World Partition data: the actor descriptor of an OFPA actor. The
//! layout is in `docs/formats/world-partition.md`.

use crate::error::Error;
use crate::package::Warning;
use crate::reader::Reader;
use crate::types::Guid;

/// Size of one custom version entry: GUID and `i32`.
const CUSTOM_VERSION_SIZE: usize = 20;
/// Size of a transform: 10 `f64`.
const TRANSFORM_SIZE: usize = 80;
/// Size of relative bounds: 10 `f64` and a boolean.
const RELATIVE_BOUNDS_SIZE: usize = 84;
/// Size of a GUID.
const GUID_SIZE: usize = 16;
/// Smallest size of a name: the string length.
const MIN_NAME_SIZE: usize = 4;

/// A custom version that controls the descriptor layout.
struct LayoutVersion {
    name: &'static str,
    guid: Guid,
    /// The smallest value of the supported layout (UE 5.6).
    minimum: i32,
    /// The newest value in UE 5.8.3.
    newest: i32,
}

const fn guid(a: u32, b: u32, c: u32, d: u32) -> Guid {
    Guid { a, b, c, d }
}

const LAYOUT_VERSIONS: [LayoutVersion; 4] = [
    LayoutVersion {
        name: "UE5MainStreamObjectVersion",
        guid: guid(0x697DD581, 0xE64F41AB, 0xAA4A51EC, 0xBEB7B628),
        minimum: 88,
        newest: 123,
    },
    LayoutVersion {
        name: "UE5ReleaseStreamObjectVersion",
        guid: guid(0xD89B5E42, 0x24BD4D46, 0x8412ACA8, 0xDF641779),
        minimum: 28,
        newest: 68,
    },
    LayoutVersion {
        name: "FortniteMainBranchObjectVersion",
        guid: guid(0x601D1886, 0xAC644F84, 0xAA16D3DE, 0x0DEAC7D6),
        minimum: 177,
        newest: 268,
    },
    LayoutVersion {
        name: "FortniteSeasonBranchObjectVersion",
        guid: guid(0x5B4C06B7, 0x24634AF8, 0x805BBF70, 0xCDF5D0DD),
        minimum: 11,
        newest: 15,
    },
];

/// The values of an actor descriptor that UCommander reads.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct ActorDesc {
    pub guid: Guid,
    /// The path of the first native class of the actor class, for example
    /// `/Script/Engine.PointLight`.
    pub native_class_path: String,
    /// The path of the Blueprint class of the actor, for example
    /// `/Game/X/BP_Y.BP_Y_C`. `None` for an actor of a native class.
    pub base_class_path: Option<String>,
    /// `None` when the descriptor has no label.
    pub label: Option<String>,
}

/// The result of [`read_actor_desc`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ActorDescRead {
    pub desc: ActorDesc,
    pub warnings: Vec<Warning>,
}

/// Reads an actor descriptor from its bytes (the decoded value of the
/// `ActorMetaData` tag).
///
/// The function reads the custom versions and the fields up to the actor
/// label. It supports the layout of UE 5.6 to UE 5.8. A missing or older
/// layout custom version gives [`Error::UnsupportedVersion`] (value -1
/// for a missing version). A class descriptor gives
/// [`Error::ClassDescriptor`].
pub fn read_actor_desc(bytes: &[u8]) -> Result<ActorDescRead, Error> {
    let mut r = Reader::new(bytes);
    let warnings = read_custom_versions(&mut r)?;
    if r.bool()? {
        return Err(Error::ClassDescriptor);
    }
    let base_class_path = class_path(&mut r)?;
    let native_class_path = class_path(&mut r)?.unwrap_or_default();
    let guid = r.guid()?;
    r.skip(TRANSFORM_SIZE + 2 * RELATIVE_BOUNDS_SIZE)?;
    skip_delta(&mut r, |r| r.skip(4))?; // Is spatially loaded.
    skip_delta(&mut r, |r| name(r).map(drop))?; // Runtime grid.
    skip_delta(&mut r, |r| r.skip(4))?; // Is editor only.
    skip_delta(&mut r, |r| r.skip(4))?; // Is runtime only.
    skip_guids(&mut r, "references")?;
    skip_guids(&mut r, "editor-only references")?;
    skip_delta(&mut r, |r| skip_names(r, "tags"))?;
    skip_delta(&mut r, |r| skip_names(r, "data layers"))?;
    skip_delta(&mut r, |r| r.skip(4))?; // Is using data layer asset.
    let label = if r.u8()? != 0 { name(&mut r)? } else { None };
    Ok(ActorDescRead {
        desc: ActorDesc {
            guid,
            native_class_path,
            base_class_path,
            label,
        },
        warnings,
    })
}

/// Reads the custom version list, and checks the layout versions.
fn read_custom_versions(r: &mut Reader) -> Result<Vec<Warning>, Error> {
    let count = r.array_count(CUSTOM_VERSION_SIZE, "custom version count")?;
    let mut values = [None; LAYOUT_VERSIONS.len()];
    for _ in 0..count {
        let guid = r.guid()?;
        let version = r.i32()?;
        if let Some(index) = LAYOUT_VERSIONS.iter().position(|known| known.guid == guid) {
            values[index] = Some(version);
        }
    }
    let mut warnings = Vec::new();
    for (known, value) in LAYOUT_VERSIONS.iter().zip(values) {
        // UE uses -1 for a custom version that is not in the list.
        let value = value.unwrap_or(-1);
        if value < known.minimum {
            return Err(Error::UnsupportedVersion {
                field: known.name,
                value,
            });
        }
        if value > known.newest {
            warnings.push(Warning::NewerVersion {
                field: known.name,
                value,
                newest: known.newest,
            });
        }
    }
    Ok(warnings)
}

/// Reads a name. The text `None` (ASCII letters in any case) gives `None`.
fn name(r: &mut Reader) -> Result<Option<String>, Error> {
    let text = r.string("name")?;
    Ok((!text.eq_ignore_ascii_case("None")).then_some(text))
}

/// Reads a class path: a package name, then an asset name.
fn class_path(r: &mut Reader) -> Result<Option<String>, Error> {
    let package = name(r)?;
    let asset = name(r)?;
    Ok(package.map(|package| match asset {
        Some(asset) => format!("{package}.{asset}"),
        None => package,
    }))
}

/// Skips a delta field: a `u8` flag, then the value when the flag is not 0.
fn skip_delta(
    r: &mut Reader,
    skip_value: impl FnOnce(&mut Reader) -> Result<(), Error>,
) -> Result<(), Error> {
    if r.u8()? != 0 {
        skip_value(r)?;
    }
    Ok(())
}

fn skip_guids(r: &mut Reader, field: &'static str) -> Result<(), Error> {
    let count = r.array_count(GUID_SIZE, field)?;
    r.skip(count * GUID_SIZE)
}

fn skip_names(r: &mut Reader, field: &'static str) -> Result<(), Error> {
    let count = r.array_count(MIN_NAME_SIZE, field)?;
    for _ in 0..count {
        r.string(field)?;
    }
    Ok(())
}

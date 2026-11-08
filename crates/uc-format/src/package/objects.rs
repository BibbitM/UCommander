//! Object names, class names and object paths from the import and export
//! tables. The rules are in `docs/formats/package.md`, "Object path".

use super::header::{PackageHeader, PackageIndex};
use crate::error::Error;

/// The class name of a package object.
const PACKAGE_CLASS: &str = "Package";
/// The class name of an export with a null class index.
const CLASS_CLASS: &str = "Class";
/// The class path of an export with a null class index.
const CLASS_CLASS_PATH: &str = "/Script/CoreUObject.Class";

/// The longest object path, in bytes, that [`PackageHeader::object_path`]
/// gives. UE has no limit. A limit keeps the time and the memory of a path
/// walk small for a deep outer chain of long names.
pub const MAX_OBJECT_PATH_LENGTH: usize = 1024;

/// The table entry that a [`PackageIndex`] refers to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IndexTarget {
    Null,
    /// Position in the import table.
    Import(usize),
    /// Position in the export table.
    Export(usize),
}

impl PackageIndex {
    /// Gives the table entry of the index. The position is not checked
    /// against the table size.
    pub fn target(self) -> IndexTarget {
        let value = i64::from(self.0);
        // Each result is in 0..=i32::MAX, so the casts do not lose data.
        match value {
            0 => IndexTarget::Null,
            ..0 => IndexTarget::Import((-value - 1) as usize),
            _ => IndexTarget::Export((value - 1) as usize),
        }
    }
}

impl PackageHeader {
    /// Gives the object name of an import or an export, with its number.
    /// `None` for a null index or an index out of the tables.
    pub fn object_name(&self, index: PackageIndex) -> Option<String> {
        let name = match index.target() {
            IndexTarget::Null => return None,
            IndexTarget::Import(i) => self.imports.get(i)?.object_name,
            IndexTarget::Export(i) => self.exports.get(i)?.object_name,
        };
        self.name_text(name)
    }

    /// Gives the class name of an import or an export. For an export, this
    /// is the object name of its class, or `Class` when its class index is
    /// null. `None` for a null index or an index out of the tables.
    pub fn class_name(&self, index: PackageIndex) -> Option<String> {
        match index.target() {
            IndexTarget::Null => None,
            IndexTarget::Import(i) => self.name_text(self.imports.get(i)?.class_name),
            IndexTarget::Export(i) => match self.exports.get(i)?.class_index {
                PackageIndex(0) => Some(CLASS_CLASS.to_owned()),
                class => self.object_name(class),
            },
        }
    }

    /// Gives the class path of an export: the object path of its class, or
    /// `/Script/CoreUObject.Class` when its class index is null. `None` for
    /// an index that is not an export of the table.
    pub(super) fn export_class_path(&self, index: PackageIndex) -> Option<String> {
        let IndexTarget::Export(i) = index.target() else {
            return None;
        };
        match self.exports.get(i)?.class_index {
            PackageIndex(0) => Some(CLASS_CLASS_PATH.to_owned()),
            class => self.object_path(class),
        }
    }

    /// Gives the full path of an import or an export, from its outer chain.
    /// An export path starts with the package name, except when its outer
    /// chain contains an import. `None` for a null index, an index out of
    /// the tables, an outer chain that does not end, or a path longer than
    /// [`MAX_OBJECT_PATH_LENGTH`] bytes.
    pub fn object_path(&self, index: PackageIndex) -> Option<String> {
        // The parts in reverse order: the object name, a separator, the
        // outer name, and so on. Join them one time at the end.
        let mut parts = vec![self.object_name(index)?];
        let mut length = parts[0].len();
        let mut has_import = false;
        let mut current = self.outer_index(index)?;
        // Each step adds at least one byte, so the length limit stops a
        // deep chain or a loop after at most `MAX_OBJECT_PATH_LENGTH` steps.
        while length <= MAX_OBJECT_PATH_LENGTH {
            if current == PackageIndex(0) {
                let is_export = matches!(index.target(), IndexTarget::Export(_));
                if is_export && !has_import {
                    length += 1 + self.summary.package_name.len();
                    parts.push(".".to_owned());
                    parts.push(self.summary.package_name.clone());
                }
                if length > MAX_OBJECT_PATH_LENGTH {
                    return None;
                }
                parts.reverse();
                return Some(parts.concat());
            }
            has_import |= matches!(current.target(), IndexTarget::Import(_));
            let outer = self.outer_index(current)?;
            let separator = if self.is_package(current) || !self.is_package_or_null(outer) {
                "."
            } else {
                ":"
            };
            let name = self.object_name(current)?;
            length += 1 + name.len();
            parts.push(separator.to_owned());
            parts.push(name);
            current = outer;
        }
        None
    }

    fn outer_index(&self, index: PackageIndex) -> Option<PackageIndex> {
        match index.target() {
            IndexTarget::Null => None,
            IndexTarget::Import(i) => Some(self.imports.get(i)?.outer_index),
            IndexTarget::Export(i) => Some(self.exports.get(i)?.outer_index),
        }
    }

    fn is_package(&self, index: PackageIndex) -> bool {
        self.class_name(index).as_deref() == Some(PACKAGE_CLASS)
    }

    fn is_package_or_null(&self, index: PackageIndex) -> bool {
        index == PackageIndex(0) || self.is_package(index)
    }
}

/// Checks that no outer chain of the tables has a loop. The error offset
/// is the start of an entry in the loop.
///
/// Each entry is visited a fixed number of times, so the check time grows
/// linearly with the table sizes.
pub(crate) fn check_outer_loops(
    header: &PackageHeader,
    import_entry_size: usize,
    export_entry_size: usize,
) -> Result<(), Error> {
    #[derive(Clone, Copy, PartialEq)]
    enum State {
        New,
        InChain,
        Done,
    }
    let import_count = header.imports.len();
    let node = |index: PackageIndex| match index.target() {
        IndexTarget::Null => None,
        IndexTarget::Import(i) => Some(i),
        IndexTarget::Export(i) => Some(import_count + i),
    };
    let outer = |node: usize| match node.checked_sub(import_count) {
        None => header.imports[node].outer_index,
        Some(i) => header.exports[i].outer_index,
    };
    let mut states = vec![State::New; import_count + header.exports.len()];
    let mut chain = Vec::new();
    for start in 0..states.len() {
        let mut current = Some(start);
        // The parser checked all package indexes, so each node is in the
        // tables.
        while let Some(n) = current.filter(|&n| states[n] != State::Done) {
            if states[n] == State::InChain {
                let offset = match n.checked_sub(import_count) {
                    None => table_offset(header.summary.imports.offset) + n * import_entry_size,
                    Some(i) => table_offset(header.summary.exports.offset) + i * export_entry_size,
                };
                return Err(Error::BadValue {
                    offset,
                    field: "outer index",
                });
            }
            states[n] = State::InChain;
            chain.push(n);
            current = node(outer(n));
        }
        for n in chain.drain(..) {
            states[n] = State::Done;
        }
    }
    Ok(())
}

/// A table offset that the parser already checked: it is not negative.
fn table_offset(offset: i32) -> usize {
    usize::try_from(offset).unwrap_or(0)
}

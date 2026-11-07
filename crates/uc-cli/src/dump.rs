//! `uc dump <file>`: shows the package summary, the name map, the import
//! table and the export table as text, one field on each line. The output
//! is for people, not for scripts.

use std::fmt::Display;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use uc_format::Error;
use uc_format::package::{
    EngineVersion, HeaderRead, NameRef, PackageHeader, PackageIndex, Table, read_header,
};

/// The first read size. The parser asks for more if the header is larger.
const FIRST_READ_SIZE: usize = 64 * 1024;
/// The largest read size. A real header is much smaller, so a header that
/// needs more bytes has a damaged count, length or offset.
const MAX_READ_SIZE: usize = 16 * 1024 * 1024;

/// Reads the header of the package at `path`, and gives the output text.
pub fn run(path: &Path) -> Result<String, String> {
    let read = read_file(path).map_err(|error| format!("{}: {error}", path.display()))?;
    Ok(format_header(&read))
}

/// Reads the start of the file. If the parser says "truncated", reads two
/// times more bytes, until the file ends or the read limit is reached.
fn read_file(path: &Path) -> Result<HeaderRead, String> {
    let mut file = File::open(path).map_err(|error| error.to_string())?;
    let mut bytes = Vec::new();
    let mut size = FIRST_READ_SIZE;
    loop {
        let missing = (size - bytes.len()) as u64;
        (&mut file)
            .take(missing)
            .read_to_end(&mut bytes)
            .map_err(|error| error.to_string())?;
        match read_header(&bytes) {
            Err(Error::Truncated { .. }) if bytes.len() == size => {
                if size >= MAX_READ_SIZE {
                    return Err("damaged package: the header is larger than 16 MiB".to_string());
                }
                size = (size * 2).min(MAX_READ_SIZE);
            }
            result => return result.map_err(|error| error.to_string()),
        }
    }
}

/// Lines of `Name: value`.
#[derive(Debug, Default)]
struct Lines(String);

impl Lines {
    /// Adds one line. Control characters from the file are escaped, so
    /// that a damaged file cannot change the terminal or add lines.
    fn add(&mut self, name: &str, value: impl Display) {
        let line = format!("{name}: {value}");
        for c in line.chars() {
            if c.is_control() {
                self.0.extend(c.escape_unicode());
            } else {
                self.0.push(c);
            }
        }
        self.0.push('\n');
    }

    fn option<T: Display>(&mut self, name: &str, value: Option<T>) {
        match value {
            Some(value) => self.add(name, value),
            None => self.add(name, "(not in file)"),
        }
    }

    fn table(&mut self, name: &str, table: Option<Table>) {
        self.option(&format!("{name} count"), table.map(|t| t.count));
        self.option(&format!("{name} offset"), table.map(|t| t.offset));
    }

    fn engine_version(&mut self, name: &str, version: Option<&EngineVersion>) {
        let text = version.map(|v| {
            let licensee = if v.is_licensee() { " (licensee)" } else { "" };
            format!("{v}{licensee}")
        });
        self.option(name, text);
    }
}

fn hex(value: u32) -> String {
    format!("0x{value:08X}")
}

fn format_header(read: &HeaderRead) -> String {
    let header = &read.header;
    let s = &header.summary;
    let mut l = Lines::default();
    l.add("Summary size", read.summary_size);
    l.add("Legacy file version", s.legacy_file_version);
    l.add("Legacy UE3 version", s.legacy_ue3_version);
    l.add("File version UE4", s.file_version_ue4);
    l.add("File version UE5", s.file_version_ue5);
    l.add("File version licensee", s.file_version_licensee_ue);
    l.option("Saved hash", s.saved_hash);
    l.add("Total header size", s.total_header_size);
    l.add("Custom version count", s.custom_versions.len());
    for custom in &s.custom_versions {
        l.add(&format!("Custom version {}", custom.guid), custom.version);
    }
    l.add("Package name", &s.package_name);
    l.add("Package flags", hex(s.package_flags));
    l.table("Name", Some(s.names));
    l.table("Soft object paths", s.soft_object_paths);
    l.option("Localization ID", s.localization_id.as_deref());
    l.table("Gatherable text data", s.gatherable_text_data);
    l.table("Export", Some(s.exports));
    l.table("Import", Some(s.imports));
    l.table("Cell export", s.cell_exports);
    l.table("Cell import", s.cell_imports);
    l.option("Metadata offset", s.metadata_offset);
    l.add("Depends offset", s.depends_offset);
    l.table("Soft package references", s.soft_package_references);
    l.option("Searchable names offset", s.searchable_names_offset);
    l.add("Thumbnail table offset", s.thumbnail_table_offset);
    l.table("Import type hierarchies", s.import_type_hierarchies);
    l.option("Package GUID", s.package_guid);
    l.option("Persistent GUID", s.persistent_guid);
    l.option("Owner persistent GUID", s.owner_persistent_guid);
    l.add("Generation count", s.generations.len());
    for (index, generation) in s.generations.iter().enumerate() {
        let value = format!(
            "{} exports, {} names",
            generation.export_count, generation.name_count
        );
        l.add(&format!("Generation {index}"), value);
    }
    l.engine_version(
        "Saved by engine version",
        s.saved_by_engine_version.as_ref(),
    );
    l.option("Engine changelist", s.engine_changelist);
    l.engine_version(
        "Compatible with engine version",
        s.compatible_with_engine_version.as_ref(),
    );
    l.add("Compression flags", hex(s.compression_flags));
    l.add("Package source", hex(s.package_source));
    l.add(
        "Additional packages to cook count",
        s.additional_packages_to_cook.len(),
    );
    for package in &s.additional_packages_to_cook {
        l.add("Additional package to cook", package);
    }
    l.add("Asset registry data offset", s.asset_registry_data_offset);
    l.add("Bulk data start offset", s.bulk_data_start_offset);
    l.option("World tile info data offset", s.world_tile_info_data_offset);
    let chunk_ids: Vec<String> = s.chunk_ids.iter().map(i32::to_string).collect();
    l.add(
        "Chunk IDs",
        if chunk_ids.is_empty() {
            "(none)".to_string()
        } else {
            chunk_ids.join(", ")
        },
    );
    l.table("Preload dependency", s.preload_dependencies);
    l.option(
        "Names referenced from export data count",
        s.names_referenced_from_export_data_count,
    );
    l.option("Payload TOC offset", s.payload_toc_offset);
    l.option("Data resource offset", s.data_resource_offset);
    format_names(&mut l, header);
    format_imports(&mut l, header);
    format_exports(&mut l, header);
    for warning in &read.warnings {
        l.add("Warning", warning);
    }
    l.0
}

fn yes_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}

/// A package index and the path of its object.
fn index_text(header: &PackageHeader, index: PackageIndex) -> String {
    match header.object_path(index) {
        Some(path) => format!("{} ({path})", index.0),
        None => format!("{} (null)", index.0),
    }
}

/// A name reference as text. The parser checked the name index.
fn name_text(header: &PackageHeader, name: NameRef) -> String {
    header.name_text(name).unwrap_or_default()
}

fn format_names(l: &mut Lines, header: &PackageHeader) {
    for (i, name) in header.names.iter().enumerate() {
        l.add(&format!("Name {i}"), &name.text);
    }
}

fn format_imports(l: &mut Lines, header: &PackageHeader) {
    for (i, import) in header.imports.iter().enumerate() {
        let index = PackageIndex(-(i as i32) - 1);
        let p = format!("Import {i}");
        l.add(
            &format!("{p} object name"),
            name_text(header, import.object_name),
        );
        l.option(&format!("{p} path"), header.object_path(index));
        l.add(
            &format!("{p} outer"),
            index_text(header, import.outer_index),
        );
        l.add(
            &format!("{p} class package"),
            name_text(header, import.class_package),
        );
        l.add(
            &format!("{p} class name"),
            name_text(header, import.class_name),
        );
        l.option(
            &format!("{p} package name"),
            import.package_name.map(|name| name_text(header, name)),
        );
        l.option(
            &format!("{p} import optional"),
            import.import_optional.map(yes_no),
        );
    }
}

fn format_exports(l: &mut Lines, header: &PackageHeader) {
    for (i, export) in header.exports.iter().enumerate() {
        let index = PackageIndex(i as i32 + 1);
        let p = format!("Export {i}");
        let index_of = |value| index_text(header, value);
        l.add(
            &format!("{p} object name"),
            name_text(header, export.object_name),
        );
        l.option(&format!("{p} path"), header.object_path(index));
        l.add(&format!("{p} class"), index_of(export.class_index));
        l.add(&format!("{p} super"), index_of(export.super_index));
        l.option(
            &format!("{p} template"),
            export.template_index.map(index_of),
        );
        l.add(&format!("{p} outer"), index_of(export.outer_index));
        l.add(&format!("{p} object flags"), hex(export.object_flags));
        l.add(&format!("{p} serial size"), export.serial_size);
        l.add(&format!("{p} serial offset"), export.serial_offset);
        l.add(&format!("{p} forced export"), yes_no(export.forced_export));
        l.add(
            &format!("{p} not for client"),
            yes_no(export.not_for_client),
        );
        l.add(
            &format!("{p} not for server"),
            yes_no(export.not_for_server),
        );
        l.option(&format!("{p} package GUID"), export.package_guid);
        l.option(
            &format!("{p} is inherited instance"),
            export.is_inherited_instance.map(yes_no),
        );
        l.add(&format!("{p} package flags"), hex(export.package_flags));
        l.option(
            &format!("{p} not always loaded for editor game"),
            export.not_always_loaded_for_editor_game.map(yes_no),
        );
        l.option(&format!("{p} is asset"), export.is_asset.map(yes_no));
        l.option(
            &format!("{p} generate public hash"),
            export.generate_public_hash.map(yes_no),
        );
        l.option(
            &format!("{p} preload dependencies"),
            export.dependencies.map(|d| {
                format!(
                    "first {}, serialization before serialization {}, \
                     create before serialization {}, serialization before create {}, \
                     create before create {}",
                    d.first_export_dependency,
                    d.serialization_before_serialization,
                    d.create_before_serialization,
                    d.serialization_before_create,
                    d.create_before_create
                )
            }),
        );
        let script = export
            .script_serialization_start_offset
            .zip(export.script_serialization_end_offset)
            .map(|(start, end)| format!("{start} to {end}"));
        l.option(&format!("{p} script serialization offsets"), script);
    }
}

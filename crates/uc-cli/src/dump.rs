//! `uc dump <file>`: shows the package summary as text, one field on each
//! line. The output is for people, not for scripts.

use std::fmt::{Display, Write};
use std::fs::File;
use std::io::Read;
use std::path::Path;

use uc_format::Error;
use uc_format::package::{EngineVersion, SummaryRead, Table, read_summary};

/// The first read size. The parser asks for more if the summary is larger.
const FIRST_READ_SIZE: usize = 64 * 1024;

/// Reads the summary of the package at `path`, and gives the output text.
pub fn run(path: &Path) -> Result<String, String> {
    let read = read_file(path).map_err(|error| format!("{}: {error}", path.display()))?;
    Ok(format_summary(&read))
}

/// Reads the start of the file. If the parser says "truncated", reads two
/// times more bytes, until the file ends.
fn read_file(path: &Path) -> Result<SummaryRead, String> {
    let mut file = File::open(path).map_err(|error| error.to_string())?;
    let mut bytes = Vec::new();
    let mut size = FIRST_READ_SIZE;
    loop {
        let missing = (size - bytes.len()) as u64;
        (&mut file)
            .take(missing)
            .read_to_end(&mut bytes)
            .map_err(|error| error.to_string())?;
        match read_summary(&bytes) {
            Err(Error::Truncated { .. }) if bytes.len() == size => size *= 2,
            result => return result.map_err(|error| error.to_string()),
        }
    }
}

/// Lines of `Name: value`.
#[derive(Debug, Default)]
struct Lines(String);

impl Lines {
    fn add(&mut self, name: &str, value: impl Display) {
        // A write to a String cannot fail.
        let _ = writeln!(self.0, "{name}: {value}");
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

fn format_summary(read: &SummaryRead) -> String {
    let s = &read.summary;
    let mut l = Lines::default();
    l.add("Summary size", read.size);
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
    for warning in &read.warnings {
        l.add("Warning", warning);
    }
    l.0
}

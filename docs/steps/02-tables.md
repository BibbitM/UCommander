# 02 – Name map, import table, export table

Branch: `pt/02-tables` (from `main`)

## Goal

Read the three main tables of an editor package: the name map, the import
table and the export table. With these tables, the parser can tell the
objects of a package, their classes and their outers. Point 3 (asset class)
and point 4 (name, path) use them first. Later points (lister tables view,
packer) use them too. Point 2 also extends `uc dump`, so that it shows the
tables as text.

## Acceptance criteria

1. `uc-format` has a public function that reads the summary and the three
   tables from a byte slice. It gives a `PackageHeader` value, or an error.
2. The parser reads all fields of a name map entry, an import entry and an
   export entry of a UE 5.8 editor package, in the order of the file. The
   field lists are in `docs/formats/package.md`, with the source of each
   fact.
3. The parser does not do I/O. The tables are in the package header (before
   the total header size). If the bytes end before a table ends, the error
   is "truncated", with the offset.
4. A name reference (`NameRef`: name index and number) gives the display
   text of the name: the base name, and `_<number - 1>` when the number is
   more than 0. Example: `K2Node_Event_0`.
5. A package index gives "null", an import or an export. The parser can give
   the class name and the full object path of each import and each export
   (example: `/Game/Summary/BP_SummaryActor.BP_SummaryActor:EventGraph`).
6. Damaged input gives an error, never a panic:
   - A table offset or a count that is out of the input.
   - A name index that is out of the name map.
   - A package index that is out of the import or export table.
   - An outer chain with a loop.
   No count from the file causes an allocation larger than the remaining
   input.
7. Fixture tests: F1 and F2 (from point 1) give the names, imports and
   exports of their `.expected.json` files.
8. `uc dump <file>` prints the summary, then the name map, the import table
   and the export table. Snapshot tests with a synthetic package and with
   F1.
9. All checks pass.

## Design

### `uc-format`

| Item | Content |
|---|---|
| `package::NameRef` | Name index (`i32`) and number (`i32`), as in the file. |
| `package::PackageIndex` | Raw `i32`. A method gives `Null`, `Import(i)` or `Export(i)`. In the file, 0 is null, a negative value `-(i + 1)` is import `i`, a positive value `i + 1` is export `i`. Observed in the `PkgInfo` output of F1 (`Outer: ... (-16)`). The format notes give the UE source. |
| `package::NameEntry` | Name text and the two stored hash values (when the version has them). |
| `package::ObjectImport` | One field for each import field, with UE names in `snake_case`. |
| `package::ObjectExport` | One field for each export field, with UE names in `snake_case`. Object flags keep the raw `u32` value. |
| `package::PackageHeader` | Summary, names, imports, exports. Methods: `name_text(NameRef)`, `object_name(PackageIndex)`, `class_name(PackageIndex)`, `object_path(PackageIndex)`. |
| `package::read_header(bytes) -> Result<HeaderRead, Error>` | Calls `read_summary`, then reads the tables at the offsets of the summary. `HeaderRead` has the header and a list of warnings. |

The parser checks all name indexes and package indexes when it reads the
tables. Then the methods of `PackageHeader` cannot fail on a bad index.

Type names may change a little during the work. I write the final names in
the PR.

### Version conditions

The fields of import and export entries change with the UE4 and UE5 file
versions. Examples, to be confirmed in increment 1: the export package GUID
(removed at UE5 `REMOVE_OBJECT_EXPORT_PACKAGE_GUID`), the inherited instance
flag (UE5 `TRACK_OBJECT_EXPORT_IS_INHERITED`), the script serialization
offsets (UE5 `SCRIPT_SERIALIZATION_OFFSET`), the import package name
(editor packages only) and the optional import flag (UE5
`OPTIONAL_RESOURCES`). Increment 1 records the full field lists and their
conditions from the UE 5.8 source, in our own words.

### `uc dump`

`uc dump <file>` reads the start of the file (64 KiB first, then the total
header size if the parser says "truncated"), calls `read_header` and prints
four sections: summary, names, imports, exports. One line for each name.
One block of `Name: value` lines for each import and each export, as in
point 1. The output is for people, not for scripts.

The snapshot files of point 1 change (new sections). Snapshot files are
display text, not expected values (`AGENTS.md` §6).

### Expected values

The `PkgInfo` output of F1 and F2 (`.pkginfo.txt`, point 1) already has the
name map, the import map and the export map. No new fixture is necessary.

I add three new keys to each `.expected.json`: `names`, `imports`,
`exports`. I do not change the existing values. I copy the values from the
`.pkginfo.txt` file only, never from our parser.

| Key | Values from `PkgInfo` |
|---|---|
| `names` | All names, in order. |
| `imports` | For each import: object name, outer (index and path), class, class package, package. |
| `exports` | For each export: object name, class (index and name), parent (super), template, outer (index and path), object flags, serial size, serial offset, not for client, not for server. |

Fields that `PkgInfo` does not show are tested only with synthetic input.

`PkgInfo` loads the package under a temporary name. Its object paths start
with `/Temp/` (example: `/Temp/Summary/BP_SummaryActor.BP_SummaryActor`),
but the package name in the file is `/Game/Summary/BP_SummaryActor`. The
`.pkginfo.txt` files do not change. The `.expected.json` files contain the
correct paths, with `/Game/` (owner decision 1). The fixture tests compare
the values directly, with no text replacement.

The names have 109 (F1) and 73 (F2) entries, and the export maps have about
200 lines each. I extract the values from the `.pkginfo.txt` files with a
one-time script (owner decision 2). The script reads the `PkgInfo` text
only. It does not use our parser. I do not add the script to the
repository. The owner checks the values.

## Fixture request

None. Point 2 uses F1 (`BP_SummaryActor.uasset`) and F2
(`L_SummaryLevel.umap`) from point 1.

## Increments (one commit each)

| # | Commit title | Content |
|---|---|---|
| 0 | Add point 2 step brief | This file, after approval. |
| 1 | Add table format notes | `docs/formats/package.md`: name map entry, name reference, package index, import entry, export entry. Fields in file order, types, version conditions, source. No code. |
| 2 | Read the name map | `read_header` with the name map. `NameEntry`, `NameRef` and name display text. Synthetic tests and truncation tests. |
| 3 | Read the import table | `ObjectImport`. Name index checks. Synthetic tests and truncation tests. |
| 4 | Read the export table | `ObjectExport`. Name index and package index checks. Synthetic tests and truncation tests. |
| 5 | Resolve object names and paths | `PackageIndex`, `object_name`, `class_name`, `object_path`. Error for an outer chain with a loop. Synthetic tests. |
| 6 | Add table fixture values | New keys in F1 and F2 `.expected.json`. Fixture tests. |
| 7 | Test damaged tables | For each length from the summary end to the end of the export table: an error, not a panic. Large counts, bad offsets, bad name indexes and bad package indexes: an error. |
| 8 | Show tables in uc dump | Name, import and export sections. Updated snapshot tests (synthetic and F1). |
| 9 | Update changelog and README for point 2 | `CHANGELOG.md` `[Unreleased]`, README: `uc dump` text, if necessary. |
| 10 | Set point 2 status to done | `docs/PLAN.md`. The last commit. |

Each feature increment follows the TDD sequence in `AGENTS.md` §6. Before
increment 6, the synthetic tests use bytes that the test code builds field
by field.

## New dependencies (gate)

None.

## Owner actions

1. Approve this brief.
2. Answer the open questions. Done, see "Owner decisions".
3. Check the extracted expected values (increment 6).

## Out of scope

- Other tables: soft object paths, gatherable text, depends map, soft
  package references, searchable names, thumbnail table, import type
  hierarchies, preload dependencies, cell imports and exports, asset
  registry data. Later points add them when a feature needs them.
- Export data (object properties).
- A check of the stored name hash values.
- Content plugin fields (point 3 and later).
- Cooked packages and Zen package headers (X2).

## Owner decisions

The owner answered the open questions of the draft:

1. **`/Temp/` in `PkgInfo` paths.** Do not change the `.pkginfo.txt`
   files. Write the correct paths (with `/Game/`) in the `.expected.json`
   files. The expected file shows the expected results. Do not add a text
   replacement to the test logic.
2. **Extraction of expected values.** Extract all values from the
   `.pkginfo.txt` files with a one-time script. The owner checks them.
3. **Damaged index.** A name index or a package index that is out of range
   gives an error, not a warning. Such an index shows that the content of
   the file is not reliable.
4. **`uc dump` output.** `uc dump` always prints all four sections.

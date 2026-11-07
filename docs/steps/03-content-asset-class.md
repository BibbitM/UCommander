# 03 – Content: asset class

Branch: `pt/03-content-asset-class` (from `main`)

Feature: D1. Release after this point: v0.2.0.

## Goal

Add the content plugin field **Asset class**. It shows the class of the
main asset of a package, for example `Blueprint`, `World` or `StaticMesh`.
With it, the user can show the asset type in a TC column, and sort and
search by it. This is the first field that uses the package parser of
points 1 and 2. Point 3 also adds the shared code that reads a package
header from a file. Point 4 adds more fields that use it.

## Acceptance criteria

1. `uc-format` gives the main asset of a package header: the export that
   UE uses as the asset of the package. The rule is in
   `docs/formats/package.md`, with its source.
2. `uc-format` gives the class name of the main asset. It uses
   `PackageHeader::class_name` from point 2.
3. A header with no main asset gives no asset class. This is not an error.
4. The content plugin has a second field, **Asset class**, at field
   index 1. Its type is a UTF-16 string. The field **Is package** stays at
   index 0, with the same name and type.
5. `ContentGetValueW` writes the value in the format of the field type
   (P2 review finding, point 1 owner decision 5): an `i32` for a boolean,
   a NUL-terminated UTF-16 string for a string. A string that is too long
   for `maxlen` is cut, and ends with a NUL. No write goes past `maxlen`
   bytes.
6. The content plugin reads only the package header, not the whole file.
   It reads 64 KiB first, and more only when the parser says "truncated",
   up to a limit of 16 MiB (the same rule as `uc dump`).
7. Return values of the **Asset class** field:

   | Case | Return value |
   |---|---|
   | Package with a main asset. | `ft_stringw`, the class name. |
   | Package with no main asset. | `ft_fieldempty`. |
   | File that is not a package, or a package that the parser does not support (UE4, unversioned, compressed). | `ft_fieldempty`. |
   | Damaged package (parser error). | `ft_fieldempty`. |
   | File that cannot be opened or read. | `ft_fileerror`. |

8. Fixture tests: F1 gives `Blueprint`, F2 gives `World`. The expected
   values come from the asset registry section of the `PkgInfo` output.
9. FFI tests load the DLL and check each row of the table in criterion 7,
   the field list, and a small `maxlen`.
10. Manual test in TC: a custom column with **Asset class** shows the
    correct value for F1 and F2.
11. All checks pass.

## Design

### Main asset rule

The asset of a package, as the UE editor shows it in the Content Browser,
is an object that has these properties:

- It is an export.
- Its outer index is null (it is directly in the package).
- Its "is asset" flag in the export entry is true.
- Its object name is equal to the short package name (the part of the
  package name after the last `/`).

Fixture F1 shows why the name condition is necessary: export 0
`BP_SummaryActor` (class `Blueprint`) and export 1 `BP_SummaryActor_C`
(class `BlueprintGeneratedClass`) both have the "is asset" flag, and both
have a null outer. The `PkgInfo` asset registry section lists both. The
Content Browser shows only the Blueprint.

Increment 1 confirms this rule in the UE 5.8.3 source (the asset data
code and `UObject::IsAsset`), and records it in our own words. If the
source gives a different rule, I stop and tell the owner before I write
code.

Open item: packages with no export that matches all conditions. Examples:
an OFPA external actor package (its actor has a level as outer, and its
package name is a hash), and a package that is renamed outside the editor.
See open question 2.

### `uc-format`

| Item | Content |
|---|---|
| `PackageHeader::main_asset() -> Option<PackageIndex>` | The export of the main asset rule. `None` if no export matches. |
| `PackageHeader::asset_class() -> Option<String>` | `class_name(main_asset()?)`. |
| `package::read_header_from(reader: impl Read) -> Result<HeaderRead, ReadError>` | The read loop of `uc dump`, moved from `uc-cli`: 64 KiB first, then two times more after each "truncated" error, up to 16 MiB. `ReadError` is an I/O error, a parser error, or "header too large". `uc dump` and the content plugin use it. |

`read_header_from` uses only `std::io::Read`. It does not use TC code or
Windows APIs (`AGENTS.md` §5, rule 1). The parser functions (`read_summary`,
`read_header`) still do no I/O.

Type names may change a little during the work. I write the final names in
the PR.

### `uc-tc` and `uc-content`

- `FIELDS` gets the field **Asset class**, type `FieldType::StringW`, at
  index 1.
- `ContentGetValueW` selects the value write by field type: `i32` for
  `Boolean`, `write_utf16` for `StringW`. The value is computed before the
  write. Each field has one function that gives the value from the file.
- No cache in point 3 (see open question 3).

The detect string does not change.

### Expected values

I add one key to each `.expected.json`:

| Fixture | Key | Value | Source in `.pkginfo.txt` |
|---|---|---|---|
| F1 | `asset_class` | `Blueprint` | Asset registry entry 0: `/Script/Engine.Blueprint'…BP_SummaryActor'`. |
| F2 | `asset_class` | `World` | Asset registry entry 0: `/Script/Engine.World'…L_SummaryLevel'`. |

I do not change the existing values.

## Fixture request

None necessary. Point 3 uses F1 (`BP_SummaryActor.uasset`) and F2
(`L_SummaryLevel.umap`).

Optional (open question 4): F3, a small asset with a native class that is
not a Blueprint or a World, to show one more class.

| # | File | Asset | Settings |
|---|---|---|---|
| F3 | `tests/fixtures/editor/M_Summary.uasset` | Material. | New material, no changes. Save. Run `PkgInfo -all`, and give the output. |

## Increments (one commit each)

| # | Commit title | Content |
|---|---|---|
| 0 | Add point 3 step brief | This file, after approval. |
| 1 | Add main asset format notes | `docs/formats/package.md`: the main asset rule and the short package name, with the source. No code. |
| 2 | Find the main asset of a package | `main_asset` and `asset_class`. Synthetic tests: the name rule, a null outer, the "is asset" flag, no main asset. |
| 3 | Add asset class fixture values | `asset_class` keys in F1 and F2 `.expected.json`. Fixture tests. |
| 4 | Move the header read loop to uc-format | Refactor only. `read_header_from` in `uc-format`, `uc dump` uses it. Tests for the loop: a header larger than 64 KiB, a damaged size above 16 MiB, an I/O error. The `uc dump` snapshots do not change. |
| 5 | Write content values by field type | Refactor in `uc-content`: a value type for each field. FFI tests for **Is package** do not change. |
| 6 | Add the Asset class content field | Field index 1. FFI tests: field list, F1, F2, not a package, damaged package, missing file, small `maxlen`. Update `docs/tc-api/content.md` if the guide check changes a fact. |
| 7 | Update changelog and README for point 3 | `CHANGELOG.md` `[Unreleased]`, README: the field list. |
| 8 | Set point 3 status to done | `docs/PLAN.md`. The last commit. |

Each feature increment follows the TDD sequence in `AGENTS.md` §6.
Increment 2 starts with synthetic headers that the test code builds, as in
point 2. Increment 3 adds the fixture tests. Increment 6 adds the
robustness test for the plugin (damaged package).

## New dependencies (gate)

None.

## Owner actions

1. Approve this brief. Done.
2. Answer the open questions. Done, see "Owner decisions".
3. Approve the download of the Content-Plugin Guide again (gate), for open
   question 1: `contentpluginhelp212_chm.zip`, 63,489 bytes, from
   `https://plugins.ghisler.com/content/`. It is not in `target/` now. I
   put it in `target/` (ignored by git). It does not go into the
   repository. Done.
4. Do the manual test in TC (after increment 6): `cargo xtask install`,
   then add the column **Asset class**, and open the fixture folder.
5. After the merge: decide about the release v0.2.0 (version change and
   tag are gates). The release is not part of this branch.

## Out of scope

- The cache of parsed headers (open question 3).
- `ft_delayed` and background reads.
- Asset name, object path and engine version fields (point 4).
- The asset registry data section of a package. The asset class comes from
  the export table.
- The full class path (`/Script/Engine.Blueprint`) and the parent class of
  a Blueprint. Possible later fields.
- OFPA fields (points 5 and 6).
- Cooked packages and Zen package headers (X2).
- Polish field names (point 16).

## Owner decisions

The owner approved this brief, the guide download, and the proposal of
each open question below:

1. Read the guide again for the string field type.
2. A package with no main asset gives `ft_fieldempty`. No fallback.
3. The cache comes in point 4.
4. F3 is optional. Point 3 continues without it.
5. The field shows the short class name.
6. A damaged package gives `ft_fieldempty`.

## Open questions (answered)

1. **String field type in `ContentGetSupportedField`.** `docs/tc-api/content.md`
   says that `ContentGetValueW` returns `ft_stringw` for UTF-16 text. It does
   not say which type `ContentGetSupportedField` must return for that field:
   `ft_string` or `ft_stringw`. I am not sure. I propose to read the guide
   again (owner action 3), and record the answer in
   `docs/tc-api/content.md`. The manual test in TC then confirms it.
2. **No main asset.** For a package with no export that matches all
   conditions, I propose `ft_fieldempty`. Alternative: a fallback to the
   first export with the "is asset" flag and a null outer. The fallback
   shows a value for renamed packages, but it can show a wrong class.
   OFPA packages get their own fields in point 6.
3. **Cache.** `AGENTS.md` §5 rule 4 and X7 ask for a cache. Point 3 has only
   one field that parses the header, so TC reads each header one time, and
   a cache gives no gain. I propose to add the cache in point 4, when more
   fields read the same header. The cache must be thread-safe (P2 review
   finding).
4. **Fixture F3.** Do you want to make F3 (a material)? It is optional.
   Without it, the fixture tests check only `Blueprint` and `World`.
5. **Display form.** I propose the short class name (`Blueprint`), as the
   Content Browser shows it. Alternative: the class path
   (`/Script/Engine.Blueprint`). A later point can add the path as a
   second field.
6. **Error versus empty.** I propose `ft_fieldempty` for a damaged package
   (criterion 7), so that TC shows an empty cell and not an error.
   `ft_fileerror` is only for I/O errors.

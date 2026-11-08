# 04 – Content: name, path, engine version

Branch: `pt/04-content-name-version` (from `main`)

Features: D2, D7. Cross-cutting: X7 (cache). Release after this point:
none (v0.3.0 comes after point 6).

## Goal

Add more content plugin fields that come from the package header: the
class path, the name and the object path of the main asset, and the
package path (D2, and an extension of D1), and the engine version
and the file versions of the package (D7). With them, the user can show
these values in TC columns, and sort and search by them. Point 4 also
adds the cache of package values (X7, point 3 owner decision 3), because
now many fields read the same header.

## Acceptance criteria

1. `uc-format` gives the class path, the object name and the object path
   of the main asset of a package header. It uses `main_asset`,
   `object_name` and `object_path` from points 2 and 3.
2. A header with no main asset gives no asset class path, no asset name
   and no object path. This is not an error.
3. The content plugin has these new fields. Fields 0 and 1 do not change.

   | Index | Name | Type | Value for F1 |
   |---|---|---|---|
   | 2 | Asset class path | `ft_string` (value `ft_stringw`) | `/Script/Engine.Blueprint` |
   | 3 | Asset name | `ft_string` (value `ft_stringw`) | `BP_SummaryActor` |
   | 4 | Object path | `ft_string` (value `ft_stringw`) | `/Game/Summary/BP_SummaryActor.BP_SummaryActor` |
   | 5 | Package path | `ft_string` (value `ft_stringw`) | `/Game/Summary/BP_SummaryActor` |
   | 6 | Engine version | `ft_string` (value `ft_stringw`) | `5.8.3` |
   | 7 | Full engine version | `ft_string` (value `ft_stringw`) | `5.8.3-58210709+++UE5+Release-5.8` |
   | 8 | File version UE5 | `ft_numeric_32` | 1018 |
   | 9 | File version UE4 | `ft_numeric_32` | 522 |
   | 10 | File version licensee | `ft_numeric_32` | 0 |

4. `ContentGetValueW` writes an `i32` for a `ft_numeric_32` field. A
   buffer smaller than 4 bytes gives `ft_fieldempty`, as for a boolean.
5. Return values of the new fields:

   | Case | Return value |
   |---|---|
   | Package with a value for the field. | The field type (`ft_stringw` or `ft_numeric_32`). |
   | Package with no main asset (fields 2 to 4). | `ft_fieldempty`. |
   | Package with no engine version, or an empty engine version (fields 6 and 7). | `ft_fieldempty`. See owner decision 5. |
   | File that is not a package, or a package that the parser does not support. | `ft_fieldempty`. |
   | Damaged package (parser error). | `ft_fieldempty`. |
   | File that cannot be opened or read. | `ft_fileerror`. |

6. The plugin keeps a cache of package values (X7). A second field of the
   same file does not read the file again. A changed file (other size,
   other last write time or other creation time) is read again. The cache has a fixed maximum
   number of entries. It is safe when TC calls from more than one thread.
7. The plugin still reads only the package header (point 3, criterion 6).
8. Fixture tests: F1, F2 and F3 give the values of criterion 3. The
   expected values come from the UE editor (owner action 3) and from the
   `PkgInfo` output.
9. FFI tests load the DLL and check: the field list, each row of the table
   in criterion 5, a small `maxlen` for a string and for a number, a
   changed file, and calls from more than one thread.
10. Manual test in TC: custom columns with the new fields show the correct
    values for F1, F2 and F3.
11. All checks pass.

## Design

### Paths

The field names use the terms of the UE editor (owner decision 2):

| Field | Value | Source |
|---|---|---|
| Package path | `/Game/Summary/BP_SummaryActor` | The package name of the summary. |
| Object path | `/Game/Summary/BP_SummaryActor.BP_SummaryActor` | `object_path(main_asset)` from point 2. For an editor package it is `<package path>.<object name>`. |
| Asset class path | `/Script/Engine.Blueprint` | The path of the class of the main asset: `object_path` of its class index. |

Note: in the UE API, `FAssetData::PackagePath` is the folder
(`/Game/Summary`), and `FAssetData::PackageName` is the value of our
**Package path** field. Increment 1 records this difference in
`docs/formats/package.md`, so that the term is clear. The folder is not
a field.

The class of an asset from a Blueprint class is in another package, so
its class path is for example `/Game/X/BP_Y.BP_Y_C`. Increment 1 checks
which class path UE uses for an export with a null class index.

The package path comes from the package summary. It is not calculated
from the file path, so it does not need the `.uproject` file (X6, owner
decision 1).

The `PkgInfo` output is not a good source for the expected object path:
it shows the package as `/Temp/Summary/BP_SummaryActor`, because the
commandlet loads the file under a temporary mount point. The summary in
the file has `/Game/Summary/BP_SummaryActor` (observed with `uc dump`,
not yet an expected value). So the owner gives the object paths from the
editor (owner action 3).

### Engine version

Two fields show the "saved by" engine version of the summary (owner
decision 3). The user selects the form in TC.

| Field | Form | F1 |
|---|---|---|
| Engine version | `major.minor.patch` | `5.8.3` |
| Full engine version | `major.minor.patch-changelist+branch`, the form that UE writes (`EngineVersion` `Display` from point 1). The `PkgInfo` output shows the same text for "Engine Version". | `5.8.3-58210709+++UE5+Release-5.8` |

The base name is the short form, as for **Asset class**. The "compatible
with" engine version is not a field.

Increment 1 checks in the UE 5.8.3 source how UE handles an empty engine
version (for example a package from a source build with no version
data). It records the result in `docs/formats/package.md`. If the source
gives a rule that does not agree with criterion 5, I stop and tell the
owner.

### File versions

Three numeric fields: UE5, UE4 and licensee file version. Numeric fields
let the user search with `<` and `>` in TC. The legacy file version is
not a field: all supported packages have -8 or -9.

### `uc-format`

| Item | Content |
|---|---|
| `PackageHeader::asset_class_path() -> Option<String>` | The `object_path` of the class index of `main_asset()`. |
| `PackageHeader::asset_name() -> Option<String>` | `object_name(main_asset()?)`. |
| `PackageHeader::asset_object_path() -> Option<String>` | `object_path(main_asset()?)`. |

The package path, the engine version and the file versions are already
in `PackageSummary`. No new parser code is necessary for them.

Type names may change a little during the work. I write the final names
in the PR.

### Cache (`uc-content`)

- The cache keeps the field values of a package, not the parsed header.
  So each entry is small: a few short strings and numbers.
- Key: the file path that TC gives, the file size, the last write time
  and the creation time. One `std::fs::metadata` call gives all of them.
  The creation time detects a file that a tool replaced and gave the old
  last write time (for example an archive extract or a restore). UE saves
  a package to a temporary file and moves it to the package path, so a
  save also gives a new creation time.
- Sequence of one call (owner decision 8):
  1. Read the metadata. An error gives `ft_fileerror`.
  2. Find an entry with the same key. If it exists, use its values. The
     plugin does not read the file.
  3. Else read and parse the header.
  4. Read the metadata again. Put the result into the cache only if the
     metadata did not change. So a result from a file that changed
     during the read (for example during a save in UE) does not go into
     the cache. The call still gives the result.
- An old entry is not removed when its file changes. No call finds it,
  because the key is different. When the plugin keeps the new values of
  the same path, they replace the old entry. Else a new entry replaces
  it later.
- Known limit: a different file with the same path, size, last write
  time and creation time gives the cached values. On NTFS the times have
  a resolution of 100 ns. On FAT32 the last write time has a resolution
  of 2 s. Network drives are not examined.
- The same file with two paths (letter case, 8.3 name, link) gives two
  entries. This costs one more read. The values stay correct.
- The cache also keeps the result "no values" (not a package, not
  supported, damaged). So TC does not parse a damaged file once for each
  field.
- An I/O error is not kept. The next call tries again.
- At most 16 entries. A new entry replaces the oldest entry.
- A `Mutex` protects the cache. If a panic poisoned the mutex, the plugin
  uses the data again: each entry is complete before it goes into the
  cache, so the data stays valid.
- **Is package** does not use the cache. It reads only 4 bytes.
- No new dependency. The cache uses only `std`.

`ft_delayed` and background reads stay out of scope.

### Expected values

I add these keys to each `.expected.json`. I do not change the existing
values.

| Key | F1 | F2 | F3 | Source |
|---|---|---|---|---|
| `asset_class_path` | from the editor | from the editor | from the editor | Owner action 3. |
| `asset_name` | `BP_SummaryActor` | `L_SummaryLevel` | `M_Summary` | The asset registry section of the `.pkginfo.txt` (the object name after the last `.`). |
| `object_path` | from the editor | from the editor | from the editor | Owner action 3. |
| `package_path` | from the editor | from the editor | from the editor | Owner action 3. |

The engine version (`saved_by_engine_version`) and the UE5 file version
(`file_version_ue5`) are already in the files. The UE4 and the licensee
file versions are not in the `PkgInfo` output. Increment 3 adds them to
`summary` in each file (owner decision 6):

| Key | F1, F2, F3 | Source |
|---|---|---|
| `file_version_ue4` | 522 | The newest UE4 file version in UE 5.8.3 (`ObjectVersion.h`). |
| `file_version_licensee` | 0 | The licensee version of a stock engine. |
| `file_versions_source` | Text with the two sources above. | — |

The existing `source` key does not change.

## Fixture request

No new asset. Owner action 3 asks for three values from the editor.

## Increments (one commit each)

| # | Commit title | Content |
|---|---|---|
| 0 | Add point 4 step brief | This file, after approval. |
| 1 | Add asset path and engine version notes | `docs/formats/package.md`: the class path and the object path of the main asset, the package path and the UE API terms, the empty engine version rule. With the source. No code. |
| 1a | Omit an empty branch in engine version text | Fix of the point 1 `Display` for `EngineVersion`: no `+` when the branch is empty, as in UE. Synthetic test. The `uc dump` snapshots do not change. Added after increment 1 (owner decision 5). |
| 2 | Give the class, name and path of the main asset | `asset_class_path`, `asset_name` and `asset_object_path`. Synthetic tests: a top-level asset, a class from another package, no main asset. |
| 3 | Add asset name and path fixture values | `asset_class_path`, `asset_name`, `object_path` and `package_path` keys, and the file version keys, in F1, F2 and F3 `.expected.json`. Fixture tests. |
| 4 | Cache package values in the content plugin | Refactor of **Asset class**: it gets its value from the cache. FFI tests: other content and other size give the new value; the same size and another last write time give the new value; the same size and the same times give the cached value (the known limit); calls from more than one thread. The **Asset class** FFI tests do not change. `docs/tc-api/content.md`: the call order from the guide (owner decision 7). |
| 5 | Add the asset class path field | Field 2. FFI tests: field list, F1–F3, the rows of criterion 5, small `maxlen`. |
| 6 | Add the asset name and path fields | Fields 3 to 5. FFI tests as in increment 5. |
| 7 | Add the engine version fields | Fields 6 and 7. FFI tests as in increment 5. |
| 8 | Add the file version fields | Fields 8 to 10, and the `i32` write for `ft_numeric_32`. FFI tests as in increment 5, and a `maxlen` less than 4. |
| 9 | Update changelog and README for point 4 | `CHANGELOG.md` `[Unreleased]`, README: the field list. |
| 9a | Update the feature scope | `FEATURES.md`: X6 to `Later` (owner decision 1), new feature D15 (owner decision 9). `PLAN.md`: point 6 gets D15. |
| 10 | Set point 4 status to done | `docs/PLAN.md`. The last commit. |

Each feature increment follows the TDD sequence in `AGENTS.md` §6.
Increment 2 starts with synthetic headers that the test code builds.
Increment 3 adds the fixture tests. Increments 5 to 8 each add the
robustness test for the plugin (damaged package).

## New dependencies (gate)

None.

## Owner actions

1. Approve this brief. Done.
2. Answer the open questions. Done, see "Owner decisions".
3. In the UE editor, for F1, F2 and F3: right-click the asset in the
   Content Browser, select "Copy Reference", and give me the text. I take
   the asset class path and the object path from it. Also give the
   package path as the editor shows it.
4. Do the manual test in TC (after increment 8): `cargo xtask install`,
   then add the new columns, and open the fixture folder. Also sort by a
   new column, and save an asset in UE while TC shows its folder. Done:
   the owner reported no problem.

## Out of scope

- The `.uproject` search (X6). See open question 1.
- `ft_delayed` and background reads.
- The reference form of the path (`/Script/Engine.Blueprint'…'`).
- The folder of the package (`/Game/Summary`).
- The "compatible with" engine version, the legacy file version and the
  custom versions as fields.
- OFPA fields (points 5 and 6). An OFPA external actor package gets an
  asset name and an object path from the main asset rule, but point 4 has
  no fixture for it.
- Cooked packages and Zen package headers (X2).
- Polish field names (point 16).
- The sample custom column set (D15, point 6, owner decision 9).

## Owner decisions

1. Use the summary package name. The `.uproject` search (X6) is not
   necessary now. Increment 9a changes X6 to `Later` in `FEATURES.md`.
2. Use the terms of the UE editor. Fields: **Asset class path**
   (`/Script/Engine.World`), **Object path**
   (`/Game/Summary/L_SummaryLevel.L_SummaryLevel`) and **Package path**
   (`/Game/Summary/L_SummaryLevel`). **Asset class** stays the short
   class name. No field for the folder (`/Game/Summary`) and no field for
   the reference form.
3. Two engine version fields: **Engine version** (`5.8.3`) and **Full
   engine version** (`5.8.3-58210709+++UE5+Release-5.8`). The user
   selects the form in TC.
4. Three `ft_numeric_32` fields for the file versions: **File version
   UE5**, **File version UE4** and **File version licensee**.
5. An empty "saved by" engine version gives `ft_fieldempty` in both
   engine version fields. Increment 1 checks the UE rule for "empty". If
   it is different, I stop and tell the owner. **Changed after increment
   1:** in UE, a version is empty when major, minor and patch are 0. The
   changelist and the branch are not part of the rule. The owner decided:
   use the UE rule. Also, the UE text has no `+` when the branch is
   empty, but the point 1 `Display` always writes `+`. The owner decided:
   fix `Display` in increment 1a.
6. The expected UE4 file version is 522, the newest UE4 version in UE
   5.8.3 (`ObjectVersion.h`). The expected licensee version is 0, the
   value of a stock engine. Both are engine facts, not parser output.
7. The cache has 16 entries. The guide (page `contentgetvalue.htm`, the
   part about caching) says that TC requests the values file by file:
   it goes to the next file only after the previous file is complete.
   Other fields can come at the same time from a background thread. The
   note is in the full-text part of the page, so it is possibly only for
   full-text fields. Increment 4 records this fact in
   `docs/tc-api/content.md`, with this limit. The manual test in TC
   checks that a sort by a field column stays fast. No benchmark: on an
   SSD, the manual test possibly shows no difference.
8. Cache key and check: path, size, last write time and creation time.
   The plugin reads the metadata before and after it reads the file, and
   keeps the result only when the metadata did not change. No change
   notifications (`ReadDirectoryChangesW`). I expect that TC asks again
   for the values after it refreshes a changed folder. I did not check
   it. The manual test checks that TC shows the new values after a save
   in UE.
9. A sample TC custom column set for UE files, as a `wincmd.ini`
   fragment (section `[CustomFields]`) that the user can paste, comes in
   point 6, when the MVP content fields are complete. It goes into the
   README, in a part that tells how each plugin works. Increment 9a adds
   it to `FEATURES.md` as D15 (status MVP), and adds D15 to point 6 in
   `PLAN.md`.

## Open questions (answered)

The answers are in "Owner decisions".

1. **Package name source and X6.** `FEATURES.md` says that object paths
   need the `.uproject` search (X6). The summary of a UE 5.8 editor
   package contains the long package name (observed in F1, F2, F3). I
   propose to use the summary package name, as the main asset rule
   already does (`docs/formats/package.md`, "UCommander rules"). It is
   different from the editor value only for a package that was moved or
   renamed outside the editor. X6 then is not necessary for the MVP. I
   propose to change X6 to `Later` in `FEATURES.md` in increment 9a.
   Alternative: implement X6 in point 4. It finds the `.uproject` and the
   `Content` folder above the file, and calculates `/Game/…` from the
   file path. It handles moved files, but needs rules for plugin content
   and engine content, and a second cache.
2. **Path form.** I propose the object path
   (`/Game/Summary/BP_SummaryActor.BP_SummaryActor`). Alternative or
   extra field: the reference form that "Copy Reference" gives
   (`/Script/Engine.Blueprint'/Game/Summary/BP_SummaryActor.BP_SummaryActor'`).
   It needs the class path of the main asset (an import path, available
   from point 2). It is long for a column.
3. **Engine version form.** I propose the full text
   (`5.8.3-58210709+++UE5+Release-5.8`), as `PkgInfo` shows it. It shows
   the changelist and the branch, which identify a custom engine build.
   Alternative: the short form (`5.8.3`), or both as two fields.
4. **File version fields.** I propose three `ft_numeric_32` fields (UE5,
   UE4, licensee), so that TC can compare them as numbers. Alternative:
   one text field, for example `1018 / 522 / 0`.
5. **Empty engine version.** I propose `ft_fieldempty` when the "saved
   by" engine version is empty (all numbers 0 and an empty branch).
   Increment 1 checks the UE rule for "empty". I stop if it is different.
6. **Expected UE4 and licensee file versions.** `PkgInfo` shows only the
   UE5 file version. I propose these sources for F1, F2 and F3: UE4
   version 522 is the newest UE4 version in UE 5.8.3 (`ObjectVersion.h`),
   and the licensee version 0 is the value of a stock engine. Both are
   facts from the engine, not from our parser. Do you agree, or do you
   want to give the values from another tool?
7. **Cache size.** I propose 16 entries. TC asks for all columns of one
   file, then goes to the next file, so a small cache is enough. Each
   entry is less than about 1 KiB.

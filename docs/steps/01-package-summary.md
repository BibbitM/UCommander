# 01 – Package summary parser and `uc dump`

Branch: `pt/01-package-summary` (from `main`)

## Goal

Read the package summary (`PackageFileSummary`) of an editor package. The
summary is the header at the start of each `.uasset` and `.umap` file. It has
the versions, the package name, the flags, and the offsets and counts of all
tables. Points 2 to 10 use it. Point 1 also adds `uc dump <file>`, which
shows the summary as text. With it, the owner and the agent can examine a
real package without a debugger.

## Acceptance criteria

1. `uc-format` has a public function that reads the summary from a byte
   slice and gives a `PackageSummary` value, or an error.
2. The parser reads all summary fields of a UE 5.8 editor package, in the
   order of the file. The field list is in `docs/formats/package.md`, with
   the source of each fact.
3. The parser does not do I/O. The caller gives the first bytes of the file.
   If the bytes end before the summary ends, the error says "truncated", and
   gives the offset. Then the content plugin can read more bytes, and try
   again (later point).
4. Damaged input gives an error, never a panic. A count or a string length
   from the file never causes an allocation larger than the remaining input.
5. A package version that is newer than the newest known version does not
   stop the parser. The parser uses the newest known layout, and reports a
   warning (`AGENTS.md` §5, rule 7).
6. Packages that the parser does not support give a clear error:
   - UE4 packages (legacy file version older than the UE 5 value).
   - Unversioned packages (cooked; Zen headers are after the MVP, X2).
   - Byte-swapped packages (P2 decision).
   - Compressed packages (a non-empty compressed chunk list).
7. Fixture tests: each fixture in "Fixture request" gives the values of its
   `.expected.json` file.
8. `uc dump <file>` prints each summary field on one line, as
   `Name: value`. A snapshot test compares the output with a stored text
   file. A file that is not a package gives an error message and exit
   code 1.
9. The fixture confirms that the package file tag is stored little-endian
   (open item in `docs/formats/package.md`).
10. All checks pass.

## Design

### `uc-format`

| Item | Content |
|---|---|
| `package::PackageSummary` | One field for each summary field, with UE names in `snake_case`. Offsets are `i64` or `i32` as in the file. Flags keep the raw `u32` value. |
| `package::CustomVersion` | GUID and version number. |
| `package::EngineVersion` | Major, minor, patch, changelist, branch name. |
| `package::read_summary(bytes) -> Result<SummaryRead, Error>` | `SummaryRead` has the summary, the summary size in bytes, and a list of warnings. |
| `Error` | Truncated (offset), not a package, unsupported version (value), unversioned, compressed, bad value (offset, field name). |
| Reader (private) | Bounds-checked reads of integers, GUIDs, strings (ANSI and UTF-16) and counted arrays. Before an array allocation, it checks `count × element size ≤ remaining bytes`. Tests use the public API only. |

Type names may change a little during the work. I write the final names in
the PR.

### `uc dump`

`uc dump <file>` reads the start of the file (64 KiB first; more if the
parser says "truncated"), calls `read_summary` and prints the fields. The
output is for people, not for scripts. The format can change before 1.0.

### Expected values

`AGENTS.md` §6 says that expected values come from the UE editor, not from
our parser. I propose the `PkgInfo` commandlet of the engine as the source:

```bat
UnrealEditor-Cmd.exe <project>.uproject -run=PkgInfo <package file> -all
```

The owner runs it on each fixture, and gives the output. I write the
`.expected.json` from that output only, by hand, and I add the output file
next to the fixture (`<name>.pkginfo.txt`). A reviewer can then compare the
two files. Fields that `PkgInfo` does not show are tested only with
synthetic input.

`.expected.json` layout (example, values not real):

```json
{
  "source": "PkgInfo commandlet, UE 5.8.0",
  "summary": {
    "legacy_file_version": -8,
    "file_version_ue4": 0,
    "file_version_ue5": 0,
    "file_version_licensee_ue": 0,
    "package_name": "/Game/Fixtures/Summary/BP_SummaryActor",
    "package_flags": "0x00000000",
    "name_count": 0,
    "import_count": 0,
    "export_count": 0,
    "saved_by_engine_version": "5.8.0-0+++UE5+Release-5.8",
    "custom_versions": [
      { "guid": "00000000-00000000-00000000-00000000", "version": 0 }
    ]
  }
}
```

The fixture test reads this file with `serde_json` (see "New
dependencies").

## Fixture request

The owner makes these assets in the fixtures project with the stock UE 5.8
engine (not the custom build), and gives the files and the `PkgInfo` output.
Keep the assets small. No Starter Content, no Epic samples.

| # | File | Asset | Settings |
|---|---|---|---|
| F1 | `tests/fixtures/editor/BP_SummaryActor.uasset` | Blueprint class, parent `Actor`. | One component (`StaticMeshComponent`, no mesh). Compile and save. Keep the thumbnail. |
| F2 | `tests/fixtures/editor/L_SummaryLevel.umap` | Level, **not** World Partition (legacy map). | Empty level with one `PointLight`. Save. |

The OFPA and World Partition fixtures come in point 5. Point 1 needs no
container fixture.

Please also tell me:

- The engine version text from the editor "About" window.

## Increments (one commit each)

| # | Commit title | Content |
|---|---|---|
| 0 | Add point 1 step brief | This file, after approval. |
| 1 | Add package summary format notes | `docs/formats/package.md`: the summary fields in file order, their types, and the version conditions, in our own words, with the source. No code. |
| 2 | Read package versions | `read_summary` up to the custom versions: tag, legacy file version, UE4, UE5 and licensee versions, custom version list. Errors for UE4, unversioned and byte-swapped packages. Warning for an unknown newer version. Synthetic tests and truncation tests. |
| 3 | Read package name, flags and offsets | Package name, flags, and the count and offset of each table, up to the thumbnail table and the version-dependent tables after it. Synthetic tests and truncation tests. |
| 4 | Read engine versions and generations | The rest of the summary: GUIDs or saved hash, generations, engine versions, compression data, asset registry and bulk data offsets, chunk IDs, preload dependencies, payload and data resource offsets. Synthetic tests and truncation tests. |
| 5 | Add summary fixtures | F1 and F2 (Git LFS), their `.pkginfo.txt` and `.expected.json`. Fixture tests. `serde_json` dev-dependency. Confirm the tag byte order in `docs/formats/package.md`. |
| 6 | Test each truncation of the fixtures | For each length from 0 to the summary size: the result is an error, not a panic. One test changes each count in the summary to a very large value: the result is an error. |
| 7 | Add uc dump | CLI command, snapshot test with F1, test for a file that is not a package. |
| 8 | Update changelog and README for point 1 | `CHANGELOG.md` `[Unreleased]`, README: `uc dump`. |
| 9 | Set point 1 status to done | `docs/PLAN.md`. The last commit. |

Each feature increment follows the TDD sequence in `AGENTS.md` §6. Before
increment 5, the synthetic tests use summary bytes that the test code
builds field by field.

## New dependencies (gate)

| Crate | Where | Reason | License | Alternatives |
|---|---|---|---|---|
| `serde_json` (with `serde`, `derive`) | `uc-format` dev-dependencies only | Read `.expected.json` in fixture tests. All later fixture tests use it too. | `MIT OR Apache-2.0` | A small JSON reader in test code: more code to maintain and to test. A `key = value` text format: then the file name is not `.expected.json`, as `AGENTS.md` §6 says. |

The plugins and `uc` do not get a new dependency. The snapshot test uses a
stored text file and `assert_eq!`, not a snapshot crate.

## Owner actions

1. Approve this brief. Done.
2. Approve the `serde_json` dev-dependency. Done.
3. Make F1 and F2, run `PkgInfo`, and give me the files and the output
   (before increment 5). Increments 1 to 4 do not need them.
4. Answer the open questions. Done, see "Owner decisions".

## Out of scope

- Name map, import table, export table (point 2).
- Content plugin fields (point 3 and later).
- Cooked packages and Zen package headers (X2).
- Packages with a byte-swapped tag, UE4 packages, compressed packages:
  error only.
- A machine-readable `uc dump` output (JSON).

## Owner decisions

The owner approved this brief and the `serde_json` dev-dependency (with
`derive`). The owner answered the open questions of the draft:

1. **Version numbers of UE 5.8.** Read the UE 5.8 source on GitHub. Write
   the facts in our own words in `docs/formats/package.md`, with the source.
   Do not copy the source.
2. **Custom engine.** The owner's project uses a custom engine build with
   changes to the package format. Make F1 and F2 with the stock UE 5.8
   engine. Point 1 does not support the custom changes.
3. **Fixtures repository.** Copy the fixture files into `tests/fixtures/` for
   this point. The separate fixtures repository comes later.
4. **Older UE 5 versions.** Accept each UE 5 version (best effort, X1). Use
   the version conditions to read the fields. No fixtures for versions before
   5.8 in this point.
5. **P2 review finding.** Move "`ContentGetValueW` writes an `i32` for each
   field. Select the value type by field type." to point 3. Point 3 adds the
   first string field (asset class).
6. **`PkgInfo` as the source.** Use the `PkgInfo` commandlet output as the
   source of expected values.

# P2 – Content plugin skeleton

Status: **In review**

Branch: `pt/p2-content-skeleton` (from `main`)

## Goal

Make the smallest content plugin that TC can load. It has one field: **Is
package**. The field is "Yes" when the file starts with the UE package file
tag. P2 also makes the test harness that loads the real DLL and calls its
exports, as TC does. All later plugin tests use this harness.

## Acceptance criteria

1. `cargo build --workspace --release` makes `target/release/uc_content.dll`.
2. The DLL exports the functions in "API scope" with the names that TC
   expects (checked with a test that loads the DLL).
3. FFI tests load the built DLL and check:
   - Field 0 is "Is package", type boolean.
   - Field 1 does not exist (`ft_nomorefields`).
   - A file that starts with the package file tag gives "Yes".
   - A file with other bytes gives "No".
   - A file with less than 4 bytes gives "No".
   - A file that does not exist gives `ft_fileerror`.
   - An unknown field index gives `ft_nosuchfield`.
   - A buffer that is too small for a string does not overflow
     (tested on `ContentGetSupportedField` with a small `maxlen`).
   - The detect string limits the plugin to `.uasset` and `.umap` files.
4. A panic inside an export returns an error code. It does not cross the FFI
   boundary (tested with a test-only export or a unit test of the guard).
5. All checks pass in CI, including the FFI tests. CI builds the DLL before
   the FFI tests (P1 review finding).
6. Manual test in TC: the field shows in a custom column (see "Manual test").

## API scope

From the Content-Plugin Guide 2.12. Checked against the guide in increment 1;
see `docs/tc-api/content.md`.

| Export | P2 | Notes |
|---|---|---|
| `ContentGetSupportedField` | Yes | Field names, types, units. |
| `ContentGetValue` (ANSI) | No | Changed after the guide check: `64_bit_support.htm` says that a Unicode-only plugin is sufficient, but `overview.htm` lists this function as a minimum function. The manual test on TC 11.x confirmed that TC loads the plugin without it. |
| `ContentGetValueW` | Yes | TC 64-bit uses this form when it exists. |
| `ContentGetDetectString` | Yes | `EXT="UASSET" \| EXT="UMAP"` |
| `ContentSetDefaultParams` | Yes | Accepts the parameters, but does not store them yet: nothing uses them in P2. |
| `ContentPluginUnloading` | Yes | Empty. |
| `ContentGetSupportedFieldFlags`, `ContentStopGetValue`, `ContentGetDefaultSortOrder`, `ContentSetValue`, `ContentEditValue`, `ContentSendStateInformation`, `ContentCompareFiles` | No | Later plan points. |

Calling convention: `extern "system"` (on x64 this is the same as the C
convention; TC 32-bit `__stdcall` is not in scope).

## Architecture

| Crate | Adds |
|---|---|
| `uc-format` | `has_package_file_tag(bytes) -> bool`. Pure function, no I/O. |
| `uc-tc` | Content API constants (field types, return codes) as Rust types. Safe helpers: write an ANSI or UTF-16 string into a TC buffer with truncation and a NUL; read a TC path (`*const u16`). A panic guard that calls `catch_unwind` and returns an error value. |
| `uc-content` | The exports. Each export: guard → convert pointers → call safe Rust code. |
| `uc-test-host` (new, `publish = false`) | Test helper. Loads a plugin DLL with `LoadLibraryW` and finds exports with `GetProcAddress`. Kernel32 declarations only, **no new crates.io dependency**. Later the lister and packer tests use it too. |

## Increments (one commit each)

| # | Commit title | Content |
|---|---|---|
| 0 | Add P2 step brief | This file, after approval. |
| 1 | Add content plugin API notes | `docs/tc-api/content.md`: the exports, types and return codes used in P2, checked against guide 2.12, in our own words, with the source. |
| 2 | Detect the package file tag | `uc-format`: `has_package_file_tag`. Tests: tag → true, other bytes → false, short input → false, empty → false. `docs/formats/package.md`: the tag value and byte order, with the source. |
| 3 | Add content API types | `uc-tc`: field type and return code types. Unit tests for the numeric values. |
| 4 | Add safe TC buffer helpers | `uc-tc`: string write and path read helpers. Unit tests: fits, truncates, `maxlen` 0 and 1, NUL at the end, non-ASCII path. |
| 5 | Add panic guard for exports | `uc-tc`: guard. Unit test: a panic gives the error value. |
| 6 | Add plugin test host | `crates/uc-test-host`: load DLL, get export, unload. Test: load `kernel32.dll` and find `GetTickCount`. |
| 7 | Export field list from content plugin | `ContentGetSupportedField`, `ContentGetDetectString`, `ContentSetDefaultParams`, `ContentPluginUnloading`. FFI tests for field 0, field 1, small buffer, detect string. |
| 8 | Report package file tag in content plugin | `ContentGetValueW`. FFI tests with synthetic files in a temporary folder: tag, other bytes, short file, missing file, unknown field. |
| 9 | Build plugin crates as rlib for tests | Increment 7 showed that the FFI tests loaded an old DLL from `target/<profile>/`. The `rlib` crate type makes `cargo test` build the DLL, because each integration test then depends on the library target. **Correction:** the DLL goes to `target/<profile>/deps/`, and only `cargo build` copies it to `target/<profile>/`. So the test host must load it from `deps/`. Commits `Detect missing and stale plugin DLLs in tests`, `Build plugins before tests in CI`, `Build before tests in RustRover` and `Add build order and plugin crate rules` came from a wrong conclusion ("`cargo test` does not build the DLL"). The second review found this. Commits `Load plugin DLLs from the deps folder`, `Remove the build step before CI tests`, `Remove the RustRover build task` and `Correct the plugin DLL rules` remove them. The message of the rlib commit is also partly wrong. History stays. |
| 10 | Update changelog, plan and README for P2 | `CHANGELOG.md`, `docs/PLAN.md`, README "Manual installation (development)". |

Each feature increment follows the TDD sequence in `AGENTS.md` §6. P2 has no
real fixture yet: fixtures start in point 1. The synthetic tests are enough
for a 4-byte check.

## Manual test in TC

`cargo xtask install` comes in P3, so P2 uses a manual installation:

1. Run `cargo build --workspace --release`.
2. Copy `target\release\uc_content.dll` to a test folder, with the name
   `UCommanderContent.wdx64`.
3. In TC, open Configuration → Options → Plugins → Content plugins (.WDX) →
   Configure → Add.
4. Select `UCommanderContent.wdx64`.
5. Make a custom column with the field `UCommanderContent.Is package`.
6. Open a folder with `.uasset` files. Make sure that the column shows "Yes".
7. Rename a copy of a text file to `test.uasset`. Make sure that the column
   shows "No".

## Owner actions

1. **Approve the download of the Content-Plugin Guide** (gate):
   `contentpluginhelp212_chm.zip`, 63,489 bytes, from
   `https://plugins.ghisler.com/content/`. I put it in `target/` (ignored by
   git) and read it. The guide does not go into the repository.
2. Do the manual test in TC (above).

## Notes from the work

- The FFI tests write files to `%TEMP%\uc-content-tests-<process id>\`. They do
  not delete the folder. The files are small (less than 100 bytes each).
- Manual test on TC 11.x: the plugin loads, and the "Is package" column works.
- `ContentGetValueW` writes the boolean with an unaligned write, because the
  guide does not promise an aligned `FieldValue`.

## Review findings for later plan points

| Point | Item |
|---|---|
| 1 | `ContentGetValueW` writes an `i32` for each field. Select the value type by field type. |
| Cache | Make the cache thread-safe before a field returns `ft_delayed`. |
| P3 | Packaging must rename `uc_content.dll` to `UCommanderContent.wdx64`. |

## Out of scope

- Real package parsing (point 1).
- Field caching and `ft_delayed` (later, with slow fields).
- `pluginst.inf`, ZIPs and `cargo xtask install` (P3).

## Decisions

The owner approved these decisions with the brief.

1. **Field name style.** Users save column sets and search rules with field
   names, so names must stay the same after 1.0. I propose sentence case with
   spaces: "Is package", "Asset class", "Actor label".
2. **Byte-swapped tag.** UE also has a byte-swapped tag for packages from
   big-endian platforms. I propose "No" for it in P2, and a note in
   `docs/formats/package.md`. Editor packages on Windows always use the
   normal tag.
3. **`uc-test-host`.** A new workspace crate without dependencies, not a
   module in `uc-tc`, because only tests use it.

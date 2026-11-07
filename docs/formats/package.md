# Package format – notes

Our own notes about the Unreal Engine package format (`.uasset`, `.umap`).
Each fact has a source.

## Sources

| Key | Source |
|---|---|
| CUE4Parse | CUE4Parse (Apache-2.0), commit `21e2231`. |
| UE58 | Unreal Engine source, GitHub `EpicGames/UnrealEngine`, branch `5.8`, commit `8f7af10`. Read only. No code is copied. |

The UE58 files, with paths relative to `Engine/Source/Runtime/`:

| Short name | Path |
|---|---|
| `Summary.cpp` | `CoreUObject/Private/UObject/PackageFileSummary.cpp` |
| `Summary.h` | `CoreUObject/Public/UObject/PackageFileSummary.h` |
| `ObjectVersion.h` | `Core/Public/UObject/ObjectVersion.h` |
| `ObjectMacros.h` | `CoreUObject/Public/UObject/ObjectMacros.h` |
| `Linker.cpp` | `CoreUObject/Private/UObject/Linker.cpp` |
| `EngineVersion.cpp` | `Core/Private/Misc/EngineVersion.cpp` |
| `EngineVersionBase.h` | `Core/Public/Misc/EngineVersionBase.h` |
| `CustomVersion.cpp` | `Core/Private/Serialization/CustomVersion.cpp` |
| `String.cpp.inl` | `Core/Private/Containers/String.cpp.inl` |
| `Guid.cpp` | `Core/Private/Misc/Guid.cpp` |
| `IoHash.h` | `Core/Public/IO/IoHash.h` |

## Package file tag

| Fact | Value | Source |
|---|---|---|
| A package file starts with a 4-byte tag. | `0x9E2A83C1` | CUE4Parse, `CUE4Parse/UE4/Objects/UObject/FPackageFileSummary.cs`: constant `PACKAGE_FILE_TAG`. |
| The tag is stored little-endian on Windows. | Bytes `C1 83 2A 9E` | Observed in fixtures `BP_SummaryActor.uasset` and `L_SummaryLevel.umap` (point 1, stock UE 5.8.3). |
| Packages from big-endian platforms use the byte-swapped tag. | `0xC1832A9E` | CUE4Parse, same file: constant `PACKAGE_FILE_TAG_SWAPPED`. UE58 `Summary.cpp` also accepts this tag, and then swaps the bytes of all later values. UCommander does not support these packages (P2 decision). |
| UE does not read the summary of a file that is smaller than 32 bytes. | 32 | UE58 `Summary.cpp`, start of the read function. |

## Basic value types

All integers are little-endian.

| Type | Layout | Source |
|---|---|---|
| `i32`, `u32`, `i64`, `u16` | 4, 4, 8 and 2 bytes. | — |
| GUID | Four `u32` values (A, B, C, D): 16 bytes. | UE58 `Guid.cpp`. |
| Hash (`IoHash`) | 20 raw bytes. | UE58 `IoHash.h`: the hash is a 20-byte array. |
| String (`FString`) | `i32` length, then the characters. See below. | UE58 `String.cpp.inl`, string read function. |
| Array | `i32` count, then the elements. | Usual UE array layout. Used in UE58 `Summary.cpp`. |

String rules:

- Length 0: an empty string. No character data.
- Length greater than 0: `length` bytes of 8-bit characters. The last byte is
  the null terminator.
- Length less than 0: `-length` 16-bit (UTF-16) characters. The last
  character is the null terminator.
- The length `i32::MIN` is an error (damaged archive).
- A length of 1 gives an empty string (the terminator only).
- UE writes a null terminator at the last position, also when the file has a
  different value there.

## Summary fields

The table shows the fields in file order, for an editor package that UE 5.8
reads. Source for all rows: UE58 `Summary.cpp`, read function; types from
UE58 `Summary.h`.

"UE4 ≥ N" means: the UE4 file version is N or more. "UE5 ≥ N" means: the UE5
file version is N or more. The version names are in the next section.

| # | Field | Type | Condition |
|---|---|---|---|
| 1 | Tag | `u32` | Always. |
| 2 | Legacy file version | `i32` | Always. |
| 3 | Legacy UE3 version | `i32` | Legacy file version ≠ -4. UE ignores the value. UE writes 864. |
| 4 | File version UE4 | `i32` | Always. |
| 5 | File version UE5 | `i32` | Legacy file version ≤ -8. |
| 6 | File version licensee | `i32` | Always. |
| 7 | Saved hash | Hash | UE5 ≥ `PACKAGE_SAVED_HASH`. |
| 8 | Total header size | `i32` | UE5 ≥ `PACKAGE_SAVED_HASH`. |
| 9 | Custom versions | Array of custom versions | Legacy file version ≤ -2. Layout: see "Custom versions". |
| 10 | Total header size | `i32` | UE5 < `PACKAGE_SAVED_HASH`. (The same field as row 8, at the old position.) |
| 11 | Package name | String | Always. |
| 12 | Package flags | `u32` | Always. |
| 13 | Name count | `i32` | Always. |
| 14 | Name offset | `i32` | Always. |
| 15 | Soft object paths count | `i32` | UE5 ≥ `ADD_SOFTOBJECTPATH_LIST`. |
| 16 | Soft object paths offset | `i32` | UE5 ≥ `ADD_SOFTOBJECTPATH_LIST`. |
| 17 | Localization ID | String | Flag `FilterEditorOnly` not set, and UE4 ≥ `ADDED_PACKAGE_SUMMARY_LOCALIZATION_ID`. |
| 18 | Gatherable text data count | `i32` | UE4 ≥ `SERIALIZE_TEXT_IN_PACKAGES`. |
| 19 | Gatherable text data offset | `i32` | UE4 ≥ `SERIALIZE_TEXT_IN_PACKAGES`. |
| 20 | Export count | `i32` | Always. |
| 21 | Export offset | `i32` | Always. |
| 22 | Import count | `i32` | Always. |
| 23 | Import offset | `i32` | Always. |
| 24 | Cell export count | `i32` | UE5 ≥ `VERSE_CELLS`. |
| 25 | Cell export offset | `i32` | UE5 ≥ `VERSE_CELLS`. |
| 26 | Cell import count | `i32` | UE5 ≥ `VERSE_CELLS`. |
| 27 | Cell import offset | `i32` | UE5 ≥ `VERSE_CELLS`. |
| 28 | Metadata offset | `i32` | UE5 ≥ `METADATA_SERIALIZATION_OFFSET`. |
| 29 | Depends offset | `i32` | Always. |
| 30 | Soft package references count | `i32` | UE4 ≥ `ADD_STRING_ASSET_REFERENCES_MAP`. |
| 31 | Soft package references offset | `i32` | UE4 ≥ `ADD_STRING_ASSET_REFERENCES_MAP`. |
| 32 | Searchable names offset | `i32` | UE4 ≥ `ADDED_SEARCHABLE_NAMES`. |
| 33 | Thumbnail table offset | `i32` | Always. |
| 34 | Import type hierarchies count | `i32` | UE5 ≥ `IMPORT_TYPE_HIERARCHIES`. |
| 35 | Import type hierarchies offset | `i32` | UE5 ≥ `IMPORT_TYPE_HIERARCHIES`. |
| 36 | Package GUID (legacy) | GUID | UE5 < `PACKAGE_SAVED_HASH`. UE copies its 16 bytes into the first 16 bytes of the saved hash. |
| 37 | Persistent GUID | GUID | Flag `FilterEditorOnly` not set, and UE4 ≥ `ADDED_PACKAGE_OWNER`. |
| 38 | Owner persistent GUID | GUID | Flag `FilterEditorOnly` not set, and UE4 ≥ `ADDED_PACKAGE_OWNER`, and UE4 < `NON_OUTER_PACKAGE_IMPORT`. UE ignores the value. |
| 39 | Generations | Array of generations | Always. Layout: see "Generations". |
| 40 | Saved by engine version | Engine version | UE4 ≥ `ENGINE_VERSION_OBJECT`. Layout: see "Engine version". |
| 41 | Engine changelist | `i32` | UE4 < `ENGINE_VERSION_OBJECT`. Replaces row 40. |
| 42 | Compatible with engine version | Engine version | UE4 ≥ `PACKAGE_SUMMARY_HAS_COMPATIBLE_ENGINE_VERSION`. Without the field, UE uses the value of row 40. |
| 43 | Compression flags | `u32` | Always. |
| 44 | Compressed chunks | Array of compressed chunks | Always. Each chunk is four `i32`: uncompressed offset, uncompressed size, compressed offset, compressed size (16 bytes). Source: UE58 `Linker.cpp`. |
| 45 | Package source | `u32` | Always. |
| 46 | Additional packages to cook | Array of strings | Always. UE does not use the value now. |
| 47 | Texture allocations count | `i32` | Legacy file version > -7. Must be 0. |
| 48 | Asset registry data offset | `i32` | Always. |
| 49 | Bulk data start offset | `i64` | Always. |
| 50 | World tile info data offset | `i32` | UE4 ≥ `WORLD_LEVEL_INFO`. |
| 51 | Chunk IDs | Array of `i32` | UE4 ≥ `CHANGED_CHUNKID_TO_BE_AN_ARRAY_OF_CHUNKIDS`. |
| 52 | Chunk ID | `i32` | UE4 ≥ `ADDED_CHUNKID_TO_ASSETDATA_AND_UPACKAGE`, and UE4 < `CHANGED_CHUNKID_TO_BE_AN_ARRAY_OF_CHUNKIDS`. A value less than 0 means no chunk. Replaces row 51. |
| 53 | Preload dependency count | `i32` | UE4 ≥ `PRELOAD_DEPENDENCIES_IN_COOKED_EXPORTS`. Without the field, UE uses -1. |
| 54 | Preload dependency offset | `i32` | UE4 ≥ `PRELOAD_DEPENDENCIES_IN_COOKED_EXPORTS`. Without the field, UE uses 0. |
| 55 | Names referenced from export data count | `i32` | UE5 ≥ `NAMES_REFERENCED_FROM_EXPORT_DATA`. Without the field, UE uses the name count. |
| 56 | Payload TOC offset | `i64` | UE5 ≥ `PAYLOAD_TOC`. Without the field, UE uses -1. |
| 57 | Data resource offset | `i32` | UE5 ≥ `DATA_RESOURCES`. Without the field, UE uses -1. |

The summary ends after row 57.

Notes:

- The version conditions use the UE4 file version for UE4 version names and
  the UE5 file version for UE5 version names. Source: UE58 `ObjectVersion.h`,
  comparison operators of `FPackageFileVersion`.
- The condition "flag `FilterEditorOnly` not set" uses the package flags of
  row 12. Rows 37 and 38 are read only by an editor build of UE. The
  editor writes them for editor packages. Source: UE58 `Summary.cpp`.
- When UE reads the package flags, it clears the transient flags
  (`NewlyCreated`, `IsSaving`, `ReloadingForCooker`). The file can contain
  them. Source: UE58 `Summary.cpp` and `ObjectMacros.h`.

### Custom versions

| Legacy file version | Layout | Source |
|---|---|---|
| -2 | Enum-based (old). | UE58 `Summary.cpp`, custom version format selection. |
| -3 to -5 | GUID-based (old). | Same. |
| -6 and lower | Optimized: array of entries. Each entry is a GUID (key) and an `i32` version: 20 bytes. | Same, and UE58 `CustomVersion.cpp`. |

All UE 5 packages have a legacy file version of -8 or lower, so only the
optimized layout applies to UCommander.

### Generations

Each generation is two `i32` values: export count, then name count
(8 bytes). Source: UE58 `Linker.cpp`, generation read function.

### Engine version

| Field | Type |
|---|---|
| Major | `u16` |
| Minor | `u16` |
| Patch | `u16` |
| Changelist | `u32` |
| Branch | String |

Bit 31 of the changelist is the licensee flag. The lower 31 bits are the
changelist number. Source: UE58 `EngineVersion.cpp` and
`EngineVersionBase.h`.

UE corrects one known error in old packages: for UE4 < `CORRECT_LICENSEE_FLAG`
and engine version 4.26.0 with changelist 12740027 or more, UE clears the
licensee flag. Source: UE58 `Summary.cpp`. This does not apply to UE 5
packages.

## Package flags

| Flag | Value | Source |
|---|---|---|
| `NewlyCreated` | `0x00000001` | UE58 `ObjectMacros.h`. |
| `Cooked` | `0x00000200` | Same. |
| `UnversionedProperties` | `0x00002000` | Same. |
| `IsSaving` | `0x00008000` | Same. |
| `ReloadingForCooker` | `0x40000000` | Same. |
| `FilterEditorOnly` | `0x80000000` | Same. |

## Version numbers

Source for all values: UE58 `ObjectVersion.h`. The values come from the enum
order. We calculated them from the enum, and they agree with the values that
CUE4Parse uses for the same names.

### Legacy file version

| Value | Meaning |
|---|---|
| -2 | Enum-based custom versions. |
| -3 | GUID-based custom versions. |
| -4 | The legacy UE3 version is not in the file. |
| -5 | The legacy UE3 version is in the file again, so that old engines can stop cleanly. |
| -6 | Optimized custom versions. |
| -7 | The texture allocations count is not in the file. |
| -8 | The UE5 file version is in the file. |
| -9 | Newest value in UE 5.8. After the licensee version, the layout can change with the versions read before that point. A reader that finds a file version newer than it knows must stop after the licensee version. |

Source: UE58 `Summary.cpp`, comment and constant at the legacy file version.

UE 5.8 behavior:

- Legacy file version 0 or more: a UE3 package. UE does not read it.
- Legacy file version less than -9 (newer than UE 5.8): UE stops after this
  field, because the layout is not known.
- UE4, UE5 and licensee versions all 0: an unversioned package. The editor
  does not read it.
- UE4 version less than `OLDEST_LOADABLE_PACKAGE` (214): UE stops.
- UE4 or UE5 version newer than the newest known version: UE can stop (if
  the compatibility check is on).
- Compression flags with unknown bits, or a non-empty compressed chunk list:
  UE stops.

### UE5 file versions

| Name | Value |
|---|---|
| `INITIAL_VERSION` | 1000 |
| `NAMES_REFERENCED_FROM_EXPORT_DATA` | 1001 |
| `PAYLOAD_TOC` | 1002 |
| `OPTIONAL_RESOURCES` | 1003 |
| `LARGE_WORLD_COORDINATES` | 1004 |
| `REMOVE_OBJECT_EXPORT_PACKAGE_GUID` | 1005 |
| `TRACK_OBJECT_EXPORT_IS_INHERITED` | 1006 |
| `FSOFTOBJECTPATH_REMOVE_ASSET_PATH_FNAMES` | 1007 |
| `ADD_SOFTOBJECTPATH_LIST` | 1008 |
| `DATA_RESOURCES` | 1009 |
| `SCRIPT_SERIALIZATION_OFFSET` | 1010 |
| `PROPERTY_TAG_EXTENSION_AND_OVERRIDABLE_SERIALIZATION` | 1011 |
| `PROPERTY_TAG_COMPLETE_TYPE_NAME` | 1012 |
| `ASSETREGISTRY_PACKAGEBUILDDEPENDENCIES` | 1013 |
| `METADATA_SERIALIZATION_OFFSET` | 1014 |
| `VERSE_CELLS` | 1015 |
| `PACKAGE_SAVED_HASH` | 1016 |
| `OS_SUB_OBJECT_SHADOW_SERIALIZATION` | 1017 |
| `IMPORT_TYPE_HIERARCHIES` | 1018 (newest in UE 5.8) |

### UE4 file versions

Only the versions that the summary uses. The newest UE4 version in UE 5.8 is
522 (`CORRECT_LICENSEE_FLAG`).

| Name | Value |
|---|---|
| `OLDEST_LOADABLE_PACKAGE` | 214 |
| `WORLD_LEVEL_INFO` | 224 |
| `ADDED_CHUNKID_TO_ASSETDATA_AND_UPACKAGE` | 278 |
| `CHANGED_CHUNKID_TO_BE_AN_ARRAY_OF_CHUNKIDS` | 326 |
| `ENGINE_VERSION_OBJECT` | 336 |
| `ADD_STRING_ASSET_REFERENCES_MAP` | 384 |
| `PACKAGE_SUMMARY_HAS_COMPATIBLE_ENGINE_VERSION` | 444 |
| `SERIALIZE_TEXT_IN_PACKAGES` | 459 |
| `PRELOAD_DEPENDENCIES_IN_COOKED_EXPORTS` | 507 |
| `ADDED_SEARCHABLE_NAMES` | 510 |
| `ADDED_PACKAGE_SUMMARY_LOCALIZATION_ID` | 516 |
| `ADDED_PACKAGE_OWNER` | 518 |
| `NON_OUTER_PACKAGE_IMPORT` | 520 |
| `CORRECT_LICENSEE_FLAG` | 522 |

## UCommander rules

These are our decisions, not UE facts. They follow from the facts above and
the point 1 brief.

| Case | Result |
|---|---|
| Tag is not the package file tag or the swapped tag. | Error: not a package. |
| Swapped tag. | Error: byte-swapped package. |
| Legacy file version 0 or more, or greater than -8. | Error: unsupported version (UE3 or UE4 package). |
| Legacy file version less than -9. | Error: unsupported version. The layout is not known. |
| UE4, UE5 and licensee versions all 0. | Error: unversioned package. |
| UE5 version less than `INITIAL_VERSION` (1000). | Error: unsupported version (UE4 package). |
| UE4 version greater than 522, or UE5 version greater than 1018. | Warning. Read with the newest known layout. |
| Non-empty compressed chunk list. | Error: compressed package. |
| String with 8-bit characters. | Read each byte as one Latin-1 character. We did not examine how UE converts bytes 128 to 255. |
| String with UTF-16 characters that are not valid UTF-16. | Use U+FFFD for each bad character. |
| A count or a length that is larger than the remaining input. | Error: truncated. No allocation. |
| A count that fits in the remaining input. | Allocate capacity from the count only when one element in memory is not larger than its bytes in the file. Other lists grow with the elements that the parser reads. |
| Size of a string in memory. | At most 2 times its bytes in the file: Latin-1 bytes 128 to 255 are 2 UTF-8 bytes, and U+FFFD for 2 bad UTF-16 bytes is 3 UTF-8 bytes. |

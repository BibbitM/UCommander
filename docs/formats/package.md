# Package format – notes

Our own notes about the Unreal Engine package format (`.uasset`, `.umap`).
Each fact has a source.

## Sources

| Key | Source |
|---|---|
| CUE4Parse | CUE4Parse (Apache-2.0), commit `21e2231`. |
| UE58 | Unreal Engine source, GitHub `EpicGames/UnrealEngine`, branch `5.8`, commit `8f7af10`. Read only. No code is copied. |
| UE583 | Unreal Engine 5.8.3 source, from the Epic Games Launcher installation (`Build.version`: changelist 58210709, branch `++UE5+Release-5.8`). This engine made the point 1 fixtures. Read only. No code is copied. |

The UE58 and UE583 files, with paths relative to `Engine/Source/Runtime/`:

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
| `Linker.h` | `CoreUObject/Public/UObject/Linker.h` |
| `LinkerLoad.cpp` | `CoreUObject/Private/UObject/LinkerLoad.cpp` |
| `LinkerLoad.h` | `CoreUObject/Public/UObject/LinkerLoad.h` |
| `ObjectResource.cpp` | `CoreUObject/Private/UObject/ObjectResource.cpp` |
| `ObjectResource.h` | `CoreUObject/Public/UObject/ObjectResource.h` |
| `UnrealNames.cpp` | `Core/Private/UObject/UnrealNames.cpp` |
| `NameTypes.h` | `Core/Public/UObject/NameTypes.h` |
| `Archive.cpp` | `Core/Private/Serialization/Archive.cpp` |
| `Package.cpp` | `CoreUObject/Private/UObject/Package.cpp` |
| `Obj.cpp` | `CoreUObject/Private/UObject/Obj.cpp` |
| `Class.h` | `CoreUObject/Public/UObject/Class.h` |
| `AssetData.h` | `CoreUObject/Public/AssetRegistry/AssetData.h` |
| `AssetData.cpp` | `CoreUObject/Private/AssetRegistry/AssetData.cpp` |
| `AssetRegistryInterface.cpp` | `CoreUObject/Private/Misc/AssetRegistryInterface.cpp` |
| `BlueprintGeneratedClass.cpp` | `Engine/Private/BlueprintGeneratedClass.cpp` |
| `Actor.cpp` | `Engine/Private/Actor.cpp` |
| `Char.h` | `Core/Public/Misc/Char.h` |
| `GenericPlatformStricmp.cpp` | `Core/Private/GenericPlatform/GenericPlatformStricmp.cpp` |

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
| Boolean | `u32`: 0 is false, 1 is true. UE reads a value greater than 1 as true, and marks the archive as damaged. | UE583 `Archive.cpp`, boolean read function. |

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

## Tables

The name map, the import table and the export table are in the package
header, before the total header size. UE reads them from the offsets and
counts of the summary. They are not adjacent: other tables can be between
them. Source: UE583 `LinkerLoad.cpp`, name map, import map and export map
read functions. Observed in fixture `BP_SummaryActor.uasset`: name offset
607, soft object paths offset 3108, import offset 5309, export offset 5949.

### Name map

The name map has "name count" entries. Each entry:

| # | Field | Type | Condition |
|---|---|---|---|
| 1 | Length | `i32` | Always. |
| 2 | Characters | `length` bytes (8-bit), or `-length` UTF-16 characters | Always. The same rules as a string. |
| 3 | Non-case-preserving hash | `u16` | UE4 ≥ `NAME_HASHES_SERIALIZED`. |
| 4 | Case-preserving hash | `u16` | UE4 ≥ `NAME_HASHES_SERIALIZED`. |

Source: UE583 `UnrealNames.cpp`, name entry read function.

- UE stops with a critical error when the absolute value of the length is
  more than 1024 (`NAME_SIZE`, UE583 `NameTypes.h`).
- UE ignores the two hash values when it reads. They are only kept for the
  file layout.
- The name text ends at the first null character.
- UE does not split a number suffix from a name map entry. The text
  `K2Node_Event` stays one entry.
- Observed in fixture `BP_SummaryActor.uasset`: entry 0 has length 30
  (`/Game/Summary/BP_SummaryActor` and the terminator), then 2 hash values.

### Name reference

A name in the tables (`FName`) is two `i32` values: name index, then
number. Source: UE583 `LinkerLoad.h`, name read operator of the load
linker.

- A name index that is not in the name map gives a critical error. UE uses
  an empty name.
- Display text: the name map text. If the number is not 0, add `_` and
  `number - 1`. Example: name `K2Node_Event` with number 1 is
  `K2Node_Event_0`. Source: UE583 `NameTypes.h` (number 0 means "no
  number", external number is the internal number minus 1) and
  `UnrealNames.cpp` (name to text functions).

### Package index

A reference to an object (`FPackageIndex`) is one `i32`. Source: UE583
`ObjectResource.h`.

| Value | Meaning |
|---|---|
| 0 | Null (no object). |
| Less than 0 | Import `-value - 1`. Example: -1 is import 0. |
| More than 0 | Export `value - 1`. Example: 1 is export 0. |

UE stops with a fatal error for an import or export index out of the table
when it resolves the object. Source: UE583 `LinkerLoad.cpp`, index to
object function.

### Import entry

Fields in file order. Source: UE583 `ObjectResource.cpp`, import read
operator; types from UE583 `ObjectResource.h`.

| # | Field | Type | Condition |
|---|---|---|---|
| 1 | Class package | Name reference | Always. |
| 2 | Class name | Name reference | Always. |
| 3 | Outer index | Package index | Always. |
| 4 | Object name | Name reference | Always. |
| 5 | Package name | Name reference | UE4 ≥ `NON_OUTER_PACKAGE_IMPORT`. |
| 6 | Import optional | Boolean | UE5 ≥ `OPTIONAL_RESOURCES`. |

Notes:

- The package name is in all packages of these versions, also in editor
  packages. It can be `None`: then the package of the import is found from
  the outer chain.
- For packages with the `FilterEditorOnly` flag, the saver can write the
  object name as the package name. Then UE sets the package name to `None`
  when it reads.
- An entry is 40 bytes for a UE 5.8 package. Observed in fixture
  `BP_SummaryActor.uasset`: 16 imports from offset 5309 end at the export
  offset 5949.
- After the read, UE changes some imports for old packages (import map
  fix-up, UE583 `LinkerLoad.cpp`). UCommander does not do this.

### Export entry

Fields in file order. Source: UE583 `ObjectResource.cpp`, export read
operator; types from UE583 `ObjectResource.h`.

| # | Field | Type | Condition |
|---|---|---|---|
| 1 | Class index | Package index | Always. |
| 2 | Super index | Package index | Always. |
| 3 | Template index | Package index | UE4 ≥ `TEMPLATE_INDEX_IN_COOKED_EXPORTS`. |
| 4 | Outer index | Package index | Always. |
| 5 | Object name | Name reference | Always. |
| 6 | Object flags | `u32` | Always. |
| 7 | Serial size | `i64` | UE4 ≥ `64BIT_EXPORTMAP_SERIALSIZES`. Before: `i32`. |
| 8 | Serial offset | `i64` | UE4 ≥ `64BIT_EXPORTMAP_SERIALSIZES`. Before: `i32`. |
| 9 | Forced export | Boolean | Always. |
| 10 | Not for client | Boolean | Always. |
| 11 | Not for server | Boolean | Always. |
| 12 | Package GUID | GUID | UE5 < `REMOVE_OBJECT_EXPORT_PACKAGE_GUID`. UE ignores the value. |
| 13 | Is inherited instance | Boolean | UE5 ≥ `TRACK_OBJECT_EXPORT_IS_INHERITED`. |
| 14 | Package flags | `u32` | Always. |
| 15 | Not always loaded for editor game | Boolean | UE4 ≥ `LOAD_FOR_EDITOR_GAME`. |
| 16 | Is asset | Boolean | UE4 ≥ `COOKED_ASSETS_IN_EDITOR_SUPPORT`. |
| 17 | Generate public hash | Boolean | UE5 ≥ `OPTIONAL_RESOURCES`. |
| 18 | First export dependency | `i32` | UE4 ≥ `PRELOAD_DEPENDENCIES_IN_COOKED_EXPORTS`. |
| 19 | Serialization before serialization dependencies | `i32` | Same as row 18. |
| 20 | Create before serialization dependencies | `i32` | Same as row 18. |
| 21 | Serialization before create dependencies | `i32` | Same as row 18. |
| 22 | Create before create dependencies | `i32` | Same as row 18. |
| 23 | Script serialization start offset | `i64` | UE5 ≥ `SCRIPT_SERIALIZATION_OFFSET`, and package flag `UnversionedProperties` not set. Without the field, UE uses 0. |
| 24 | Script serialization end offset | `i64` | Same as row 23. |

Notes:

- When UE reads the object flags, it keeps only the load flags (`RF_Load`
  mask, UE583 `ObjectMacros.h`). The file can contain other bits. UE also
  applies the mask when it writes (UE583 `ObjectResource.cpp`, same
  operator), so a package that UE saved contains only load flags. Observed
  in fixtures `BP_SummaryActor.uasset` and `L_SummaryLevel.umap`: the
  stored flags are equal to the `PkgInfo` values.
- The condition of rows 23 and 24 uses the "unversioned properties" state
  of the load linker. The load linker sets this state from the package flag
  `UnversionedProperties`. Source: UE583 `LinkerLoad.cpp`, summary
  processing.
- An entry is 112 bytes for a UE 5.8 editor package. Observed in fixture
  `BP_SummaryActor.uasset`: 14 exports from offset 5949 end at 7517, which
  is the depends offset.

### Object path

UE makes the path name of an import or an export from the outer chain.
Source: UE583 `Linker.cpp`, import and export path name functions, and
`Linker.h`, class name functions.

1. Start with the object name of the entry.
2. Go to the outer. Put the outer name before the text, with a separator.
3. Continue until the outer index is null.

Separator between an outer and its inner object: `:` when the outer is not
a package, and the outer of the outer is null or a package. Else `.`. An
object is a package when its class name is `Package`.

Class name of an entry:

- Import: its class name field.
- Export: the object name of its class index. When the class index is null,
  the class name is `Class`.

For an export, add the package name and `.` at the start, except when the
outer chain contains an import. Example from the `PkgInfo` output of
`BP_SummaryActor.uasset`: export `EventGraph`, with outer export
`BP_SummaryActor` (class `Blueprint`, outer null), gives
`<package>.BP_SummaryActor:EventGraph`.

The forced export flag can change the package part of the path. UCommander
does not use this option.

### Main asset

The main asset is the object that UE uses as the asset of a package, for
example in the Content Browser and in the save code. Its class is the
asset class of the package.

UE finds it in the loaded package, not in the file. Source: UE583
`Package.cpp`, the function that finds the asset in a package. The
function examines each object of the package:

1. It ignores an object that is not an asset (see "Is asset" below).
2. It ignores an object that the asset registry filter skips (see
   "Asset registry filter" below).
3. An object with the "main name" (see below) is the result. The search
   stops.
4. If no object has the main name, the result is the first asset that the
   function found. The order is the order of the objects in memory. The
   file does not record this order.

Validity checks of loaded objects also change the result. They do not
apply to a file.

#### Is asset

The export entry field "is asset" contains the value that UE calculated
for the object when it saved the package. Source: UE583
`ObjectResource.cpp`, export entry constructor.

The calculation (UE583 `Obj.cpp`, the base "is asset" function): an object
is an asset when all of these conditions are true:

- It does not have the object flags `Transient` (`0x00000040`) or
  `ClassDefaultObject` (`0x00000010`). It has the object flag `Public`
  (`0x00000001`). Source of the values: UE583 `ObjectMacros.h`.
- It is a valid object (not marked as garbage).
- Its class does not have the class flag `Optional` (`0x00000010`, UE583
  `ObjectMacros.h`).
- In the editor: it is not a property bag placeholder. UE makes a
  placeholder for an export whose class import was missing at load.
- Its outer is a package, or it is in an external package.
- The package is not transient and is not a play-in-editor package.

Classes can change the calculation:

- A class object (`Class`) is never an asset (UE583 `Class.h`).
- A Blueprint generated class (`BlueprintGeneratedClass`) is an asset
  with the base calculation (UE583 `BlueprintGeneratedClass.cpp`).
- An actor is an asset only when it is in an external package (OFPA),
  and it is the main actor of that package (UE583 `Actor.cpp`).

So a Blueprint package has two assets: the Blueprint and its generated
class. Observed in fixture `BP_SummaryActor.uasset`: exports 0
(`BP_SummaryActor`, class `Blueprint`) and 1 (`BP_SummaryActor_C`, class
`BlueprintGeneratedClass`) have the "is asset" flag. The asset registry
section of the `PkgInfo` output lists both.

#### Main name

An object has the main name when its path name in the package is equal
to the short package name. The comparison ignores the case of ASCII
letters only (UE583 `Char.h` and `GenericPlatformStricmp.cpp`). The short
package name is the text after the last `/` of the package name. A
top-level object has a path name in the package equal to its object
name. An object in an external package never has the main name. Source:
UE583 `AssetData.cpp` and `AssetData.h`, the functions that detect a
"UAsset" by its names.

Example: package `/Game/Summary/BP_SummaryActor`, short name
`BP_SummaryActor`. Export `BP_SummaryActor` has the main name. Export
`BP_SummaryActor_C` does not.

#### Asset registry filter

In the editor, the filter skips (UE583 `AssetRegistryInterface.cpp`):

- Each object of a package with the package flag `ContainsNoAsset`.
- In an editor package (no package flag `FilterEditorOnly`): objects of
  the class `BlueprintGeneratedClass` and of its subclasses that are not
  abstract.
- In a cooked package (package flag `FilterEditorOnly`): objects of the
  class `Blueprint` and of its subclasses that are not abstract.

The filter uses the loaded class. A file gives only the class name, so a
subclass that is defined in another package is not known from the file.

## Package flags

| Flag | Value | Source |
|---|---|---|
| `NewlyCreated` | `0x00000001` | UE58 `ObjectMacros.h`. |
| `Cooked` | `0x00000200` | Same. |
| `ContainsNoAsset` | `0x00000400` | UE583 `ObjectMacros.h`. |
| `UnversionedProperties` | `0x00002000` | Same. |
| `IsSaving` | `0x00008000` | Same. |
| `ReloadingForCooker` | `0x40000000` | Same. |
| `FilterEditorOnly` | `0x80000000` | Same. |

## Version numbers

Source for all values: UE58 `ObjectVersion.h`. The values come from the enum
order. We calculated them from the enum, and they agree with the values that
CUE4Parse uses for the same names.

Point 2 added the UE4 versions 365, 485, 504, 508 and 511. Source: UE583
`ObjectVersion.h`. We calculated them from the enum order with a script.
The same script gave the correct values for the known versions 214, 507,
518, 520 and 522. We did not compare them with CUE4Parse. In the source,
the name of 508 is `VER_UE4_TemplateIndex_IN_COOKED_EXPORTS`.

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

Only the versions that the summary and the tables use. The newest UE4 version in UE 5.8 is
522 (`CORRECT_LICENSEE_FLAG`).

| Name | Value |
|---|---|
| `OLDEST_LOADABLE_PACKAGE` | 214 |
| `WORLD_LEVEL_INFO` | 224 |
| `ADDED_CHUNKID_TO_ASSETDATA_AND_UPACKAGE` | 278 |
| `CHANGED_CHUNKID_TO_BE_AN_ARRAY_OF_CHUNKIDS` | 326 |
| `ENGINE_VERSION_OBJECT` | 336 |
| `LOAD_FOR_EDITOR_GAME` | 365 |
| `ADD_STRING_ASSET_REFERENCES_MAP` | 384 |
| `PACKAGE_SUMMARY_HAS_COMPATIBLE_ENGINE_VERSION` | 444 |
| `SERIALIZE_TEXT_IN_PACKAGES` | 459 |
| `COOKED_ASSETS_IN_EDITOR_SUPPORT` | 485 |
| `NAME_HASHES_SERIALIZED` | 504 |
| `PRELOAD_DEPENDENCIES_IN_COOKED_EXPORTS` | 507 |
| `TEMPLATE_INDEX_IN_COOKED_EXPORTS` | 508 |
| `ADDED_SEARCHABLE_NAMES` | 510 |
| `64BIT_EXPORTMAP_SERIALSIZES` | 511 |
| `ADDED_PACKAGE_SUMMARY_LOCALIZATION_ID` | 516 |
| `ADDED_PACKAGE_OWNER` | 518 |
| `NON_OUTER_PACKAGE_IMPORT` | 520 |
| `CORRECT_LICENSEE_FLAG` | 522 |

## UCommander rules

These are our decisions, not UE facts. They follow from the facts above and
the point 1 and point 2 briefs.

| Case | Result |
|---|---|
| Tag is not the package file tag or the swapped tag. | Error: not a package. |
| Swapped tag. | Error: byte-swapped package. |
| Legacy file version 0 or more, or greater than -8. | Error: unsupported version (UE3 or UE4 package). |
| Legacy file version less than -9. | Error: unsupported version. The layout is not known. |
| UE4, UE5 and licensee versions all 0. | Error: unversioned package. |
| UE5 version less than `INITIAL_VERSION` (1000). | Error: unsupported version (UE4 package). |
| UE4 version greater than 522, or UE5 version greater than 1018. | Warning. Read with the newest known layout. Risk: with legacy file version -9, a newer version can change the layout after the licensee version. Then the values after that point can be wrong without an error. Owner decision (brief, acceptance criterion 5). |
| Compression flags with bits that UE does not know. | Not checked. Keep the raw value. UE stops for these flags, but we did not record the known bits. |
| Non-empty compressed chunk list. | Error: compressed package. |
| String with 8-bit characters. | Read each byte as one Latin-1 character. We did not examine how UE converts bytes 128 to 255. |
| String with UTF-16 characters that are not valid UTF-16. | Use U+FFFD for each bad character. |
| A count or a length that is larger than the remaining input. | Error: truncated. No allocation. |
| A count that fits in the remaining input. | Allocate capacity from the count only when one element in memory is not larger than its bytes in the file. Other lists grow with the elements that the parser reads. |
| Size of a string in memory. | At most 2 times its bytes in the file: Latin-1 bytes 128 to 255 are 2 UTF-8 bytes, and U+FFFD for 2 bad UTF-16 bytes is 3 UTF-8 bytes. |
| Name map entry with a length of more than 1024 characters (absolute value). | Error: bad value. UE stops for this length too. |
| Name index out of the name map. | Error: bad value (point 2 owner decision 3). |
| Package index out of the import or export table. | Error: bad value (point 2 owner decision 3). |
| Outer chain with a loop. | Error: bad value. UE has no check for this case. |
| Object path longer than 1024 bytes. | No path. UE has no limit. The limit keeps the time and the memory of a path walk small for a deep outer chain of long names. |
| Boolean with a value greater than 1. | Read as true. No error. UE also reads it as true. |
| Name map hash values. | Keep the raw values. Do not check them. |
| Object flags of an export. | Keep the raw value. Do not apply the `RF_Load` mask. |
| Import map fix-up of UE. | Not done. UCommander shows the imports as in the file. |
| Main asset: the package name. | Use the package name of the summary. UE uses the name of the loaded package, which comes from the file path. They are different only for a package that was moved outside the editor. |
| Main asset: an export with the main name. | An export with a null outer, and an object name (with its number) equal to the short package name. Objects in an external package have a non-null outer in the file, so they never match. |
| Main asset: no export with the main name. | The first asset in export table order. UE uses the order in memory, which the file does not record. The results are different only when two or more assets are candidates (point 3 owner decision 2). |
| Main asset: the "is asset" flag is not in the file (UE4 before `COOKED_ASSETS_IN_EDITOR_SUPPORT`). | Not an asset. |
| Asset registry filter: subclasses. | A class name that ends with the skipped class name (`BlueprintGeneratedClass`, or `Blueprint` in a cooked package) counts as a subclass. The file does not give the class hierarchy. |

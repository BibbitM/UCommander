# World Partition – notes

Our own notes about the World Partition data in UE packages: the actor
descriptor of an OFPA external actor. Each fact has a source.

## Sources

| Key | Source |
|---|---|
| UE583 | Unreal Engine 5.8.3 source, from the Epic Games Launcher installation. See [package.md](package.md). Read only. No code is copied. |
| UEGH | Unreal Engine source, GitHub `EpicGames/UnrealEngine`, branches `5.0` to `5.7`. Read only. No code is copied. |

The UE583 and UEGH files, with paths relative to `Engine/Source/Runtime/`:

| Short name | Path |
|---|---|
| `ActorDescUtils.cpp` | `Engine/Private/WorldPartition/WorldPartitionActorDescUtils.cpp` |
| `ActorDesc.cpp` | `Engine/Private/WorldPartition/WorldPartitionActorDesc.cpp` |
| `ActorDesc.h` | `Engine/Public/WorldPartition/WorldPartitionActorDesc.h` |
| `ActorDescArchive.cpp` | `Engine/Private/WorldPartition/WorldPartitionActorDescArchive.cpp` |
| `ActorDescArchive.h` | `Engine/Public/WorldPartition/WorldPartitionActorDescArchive.h` |
| `WorldPartition.cpp` | `Engine/Private/WorldPartition/WorldPartition.cpp` |
| `Level.cpp` | `Engine/Private/Level.cpp` |
| `Base64.cpp` | `Core/Private/Misc/Base64.cpp` |
| `Base64.h` | `Core/Public/Misc/Base64.h` |
| `Archive.cpp` | `Core/Private/Serialization/Archive.cpp` |
| `MemoryArchive.h` | `Core/Public/Serialization/MemoryArchive.h` |
| `TopLevelAssetPath.h` | `CoreUObject/Public/UObject/TopLevelAssetPath.h` |
| `TopLevelAssetPath.cpp` | `CoreUObject/Private/UObject/TopLevelAssetPath.cpp` |
| `TransformNonVectorized.h` | `Core/Public/Math/TransformNonVectorized.h` |
| `Vector.h` | `Core/Public/Math/Vector.h` |
| `DevObjectVersion.cpp` | `Core/Private/UObject/DevObjectVersion.cpp` |
| Custom version enums | `Core/Public/UObject/<Name>.h`, and from UE 5.6 also `Core/Public/UObject/<Name>s.inl` |

## Actor descriptor tags

The editor writes the actor descriptor of an actor into the asset
registry data (see [package.md](package.md), "Asset registry data"), as
two tags of the asset of the actor. Source: UE583 `ActorDescUtils.cpp`,
the function that adds the asset registry tags of an actor.

| Tag | Value |
|---|---|
| `ActorMetaDataClass` | The path of the first native class of the actor class, for example `/Script/Engine.PointLight`. |
| `ActorMetaData` | The actor descriptor, as Base64 text. |

UE uses a descriptor only when both tags are present. Source: same file,
the function that checks the asset data.

A level that is not OFPA has the tag `ActorsMetaData` (with `s`) on its
world: the descriptors of the actors of the level. Each entry contains
the actor path, the native class path and a Base64 descriptor, with `,`
between them. Source: UE583 `Level.cpp`, the function that reads the tag.
Observed in fixture F2: one entry for `PointLight_1`, and the text
`,None` after the Base64 text. UCommander does not read this tag now
(point 12).

## Base64

UE encodes the descriptor bytes with `FBase64` in the standard mode: the
alphabet of RFC 4648, table 1 (`A–Z`, `a–z`, `0–9`, `+`, `/`), with `=`
padding. Source: UE583 `ActorDescUtils.cpp` (encode call with the
default mode) and `Base64.h` (the default mode is standard).

The UE decode rules (UE583 `Base64.cpp`):

- UE removes all `=` characters at the end, also more than two.
- The remaining length modulo 4 must be 0, 2 or 3. A remainder of 1 is
  an error.
- A character that is not in the alphabet (also a character of 128 or
  more, and `=` that is not at the end) is an error.
- A remainder of 2 gives 1 byte, a remainder of 3 gives 2 bytes.
- UE does not check the unused bits of the last character.

## Descriptor bytes

The decoded bytes are a custom version list, then the descriptor
fields. Source: UE583 `ActorDesc.cpp`, the function that serializes a
descriptor to bytes, and the function that initializes a descriptor
from bytes.

The custom version list has the optimized layout of the package summary
(see [package.md](package.md), "Custom versions"): an `i32` count, then
for each entry a GUID and an `i32` version (20 bytes).

UE reads the fields with a memory reader (UE583 `MemoryArchive.h`):

| Type | Layout | Source |
|---|---|---|
| Name (`FName`) | A string (see [package.md](package.md), "String rules"), with the number suffix in the text. The text `None` is the empty name. | UE583 `MemoryArchive.h`, name operator. |
| Class path (`FTopLevelAssetPath`) | Two names: package name, then asset name. Text form: `<package>.<asset>`. When the package name is `None`, the path is empty. | UE583 `TopLevelAssetPath.h` and `.cpp`. |
| Boolean | `u32`, as in a package. | UE583 `Archive.cpp`. |
| Vector | Three `f64` (x, y, z). | UE583 `Vector.h`: `f64` when the UE5 file version of the archive is at least `LARGE_WORLD_COORDINATES`. A memory reader has the newest file version. Observed in F2: `f64`. |
| Quaternion | Four `f64` (x, y, z, w). | Same rule. Observed in F2: `0, 0, 0, 1`. |
| Transform | Rotation (quaternion), translation (vector), scale (vector): 80 bytes. | UE583 `TransformNonVectorized.h`. Observed in F2. |
| Relative bounds | Center (vector), rotation (quaternion), extents (vector), is valid (boolean): 84 bytes. | UE583 `ActorDesc.h`, operator of the relative bounds type. Observed in F2. |
| GUID array, name array | `i32` count, then the elements. | Usual UE array layout. |

### Delta fields

Many fields are "delta" fields. Source: UE583 `ActorDescArchive.h`, the
delta serializer.

- When `FortniteMainBranchObjectVersion` is at least
  `WorldPartitionActorClassDescSerialize`, a delta field starts with one
  `u8` flag. Flag 0: the value is not in the bytes. Flag not 0: the value
  follows.
- The writer writes flag 0 when the value is equal to the value of the
  class descriptor. The class descriptor comes from a registry in the
  editor (the class default object), not from the file.
- The class default object has no actor label: UE sets the label of a
  descriptor only for an actor that is not a class default object or an
  archetype. Source: UE583 `ActorDesc.cpp`, the function that
  initializes a descriptor from an actor. So flag 0 for the label means
  "no label". UE then shows the actor name. Source: same file, the
  function that gives the label or the name.

### Fields up to the label

The layout for UE 5.6 to UE 5.8, in file order. Sources: UE583
`ActorDescArchive.cpp` (the init function: rows 1 to 3) and
`ActorDesc.cpp` (the serialize function: rows 4 and later). Observed in
fixture F2 (the `ActorsMetaData` descriptor of `PointLight_1`, 490
bytes).

| # | Field | Type | F2 |
|---|---|---|---|
| 1 | Is default descriptor | Boolean | 0 |
| 2 | Base class (the Blueprint class) | Class path | empty (`None`, `None`) |
| 3 | Native class | Class path | `/Script/Engine.PointLight` |
| 4 | Actor GUID | GUID | `6413279F-42A6D887-DF5B61B2-5DEB3B08` |
| 5 | Actor transform, relative | Transform | identity |
| 6 | Runtime bounds, relative | Relative bounds | extents 1000, valid |
| 7 | Editor bounds, relative | Relative bounds | extents 1000, valid |
| 8 | Is spatially loaded | Delta boolean | flag 0 |
| 9 | Runtime grid | Delta name | flag 0 |
| 10 | Is editor only | Delta boolean | flag 0 |
| 11 | Is runtime only | Delta boolean | flag 0 |
| 12 | References | GUID array | 0 |
| 13 | Editor-only references | GUID array | 0 |
| 14 | Tags | Delta name array | flag 0 |
| 15 | Data layers | Delta name array | flag 0 |
| 16 | Is using data layer asset | Delta boolean | flag 0 |
| 17 | Actor label | Delta name | flag 1, `PointLight` |

Notes:

- The base class is empty for an actor of a native class. For an actor
  of a Blueprint class, it is the path of that class, for example
  `/Game/X/BP_Y.BP_Y_C`. The native class is the first native class in
  the class hierarchy. Source: UE583 `ActorDesc.cpp`, the function that
  initializes a descriptor from an actor.
- A descriptor with "is default descriptor" 1 is a class descriptor. It
  has no GUID, no transform and no bounds, and other rules apply.
  Source: UE583 `ActorDesc.cpp`. An external actor package contains an
  actor, not a class descriptor.
- After the label come the HLOD fields, the actor folder, the parent
  actor, the folder GUID, the property map, the content bundle GUID and
  other fields, then the component descriptors. Then the data of a
  descriptor subclass. UCommander does not read them now.
- Each descriptor subclass in UE583 (partition actor, landscape, level
  instance, packed level actor, HLOD, world data layers, map build data,
  water zone, mesh partition, PCG partition) calls the base serialize
  function first, and writes its own data after it. So the fields up to
  the label have the same position for all subclasses. Source: the
  serialize function of each subclass in UE583 (`Engine/Private/
  WorldPartition/` and the plugins `Water`, `MeshPartition` and `PCG`).
- A subclass can add more custom versions to the list. They do not
  change the fields up to the label.

### Custom versions

These four custom versions control the layout. GUIDs: UE583
`DevObjectVersion.cpp`. Values: UE583 custom version `.inl` files (each
name has an explicit value there).

| Custom version | GUID | Newest in UE 5.8.3 | Minimum for the supported layout |
|---|---|---|---|
| `UE5MainStreamObjectVersion` | `697DD581-E64F41AB-AA4A51EC-BEB7B628` | 123 | 88 (`WorldPartitionActorDescActorAndClassPaths`) |
| `UE5ReleaseStreamObjectVersion` | `D89B5E42-24BD4D46-8412ACA8-DF641779` | 68 | 28 (`LargeWorldCoordinates`) |
| `FortniteMainBranchObjectVersion` | `601D1886-AC644F84-AA16D3DE-0DEAC7D6` | 268 | 177 (`WorldPartitionActorDescSerializeRelativeTransform`) |
| `FortniteSeasonBranchObjectVersion` | `5B4C06B7-24634AF8-805BBF70-CDF5D0DD` | 15 | 11 (`WorldPartitionActorDescActorTransformSerialization`) |

Observed in F2: the list has these four entries, with the newest
values (123, 68, 268, 15).

The version names that the read code checks up to the label, with their
values (UE583) and the first UE release that has the name (UEGH):

| Custom version | Name | Value | First release |
|---|---|---|---|
| UE5 main | `WorldPartitionActorDescSerializeDataLayers` | 8 | 5.0 |
| UE5 main | `WorldPartitionActorDescSerializeActorLabel` | 22 | 5.0 |
| UE5 main | `WorldPartitionActorDescSerializeArchivePersistent` | 23 | 5.0 |
| UE5 main | `WorldPartitionActorDescRemoveBoundsRelevantSerialization` | 75 | 5.1 |
| UE5 main | `WorldPartitionActorDescActorAndClassPaths` | 88 | 5.1 |
| UE5 release | `ConvertedActorGridPlacementToSpatiallyLoadedFlag` | 18 | 5.0 |
| UE5 release | `LargeWorldCoordinates` | 28 | 5.0 |
| Fortnite main | `WorldPartitionActorDescSerializeActorIsRuntimeOnly` | 78 | 5.1 |
| Fortnite main | `WorldPartitionActorClassDescSerialize` | 88 | 5.3 |
| Fortnite main | `WorldPartitionActorDescSerializeInvalidBounds` | 96 | 5.3 |
| Fortnite main | `WorldPartitionActorDescSerializeEditorOnlyReferences` | 98 | 5.3 |
| Fortnite main | `WorldPartitionClasDescGuidTransient` | 100 | 5.3 |
| Fortnite main | `WorldPartitionActorDescSerializeEditorBounds` | 156 | 5.5 |
| Fortnite main | `WorldPartitionActorDescSerializeRelativeTransform` | 177 | 5.6 |
| Fortnite season | `WorldPartitionActorDescSerializeDataLayerAssets` | 3 | 5.3 |
| Fortnite season | `WorldPartitionActorDescNativeBaseClassSerialization` | 5 | 5.3 |
| Fortnite season | `WorldPartitionActorDescTagsSerialization` | 6 | 5.3 |
| Fortnite season | `WorldPartitionActorDescActorTransformSerialization` | 11 | 5.4 |

`FortniteSeasonBranchObjectVersion` is not in the branches `5.0` to
`5.2` (UEGH).

So the last layout change up to the label is in UE 5.6: from
`WorldPartitionActorDescSerializeRelativeTransform`, the descriptor has
a relative transform and relative bounds (rows 5 to 7). Before it, the
descriptor has an absolute transform and absolute bounds. The read code
of UEGH branch `5.6` has the same version checks and fields up to the
label as UE583. The only differences are in the component descriptors,
after the label. The UE 5.7.4 installation (Epic Games Launcher) has
the same version checks and serialized fields as UE583 in these files.
The other differences are log calls and a path conversion.

A custom version that is not in the list has the value -1 when UE reads
(UE583 `Archive.cpp`). UE 5.8 stops with a fatal error when a custom
version of the list is not known to the engine, is not valid, or is
newer than the engine. Sources: UE583 `ActorDesc.cpp`, the function
that initializes a descriptor from bytes, and
`Core/Private/Serialization/CustomVersion.cpp`, the compare function.

## Expected values

The console command `wp.Editor.DumpActorDescs <file>` writes the actor
descriptors of the open World Partition map to a CSV file. Source: UE583
`WorldPartition.cpp`. The point 5 fixtures use it as the source of the
expected values.

## UCommander rules

| Case | Result |
|---|---|
| Base64 text. | The UE decode rules above. |
| A custom version of the four is missing, or less than the minimum. | Error: unsupported descriptor (UE 5.5 or earlier). Point 5 owner decision 3. |
| A custom version of the four is newer than the newest UE 5.8.3 value. | Warning. Read with the UE 5.8.3 layout. |
| Other custom versions in the list. | Ignore them, with no warning. They are for the data of descriptor subclasses, which UCommander does not read. UE stops for a version that it does not know, but UCommander does not need these versions. |
| "Is default descriptor" is not 0. | Error: not an actor descriptor. |
| Delta label with flag 0. | No label. |
| Label, class path or name with the text `None`. | No value. UE compares names without case for ASCII letters, so `none` is also no value. |
| Bytes after the label. | Not read. |
| Count of an array larger than the remaining bytes. | Error: truncated. No allocation. |

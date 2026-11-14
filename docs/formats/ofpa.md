# OFPA actor data

## Sources

- UE 5.8.3 Launcher source, changelist 58210709.
- UE 5.7.4 Launcher source, changelist 51494982. Compared the asset
  registry records in `AssetRegistry/Private/PackageReader.cpp`.
- `tests/fixtures/editor/BP_SummaryActor.pkginfo.txt`.

Engine paths below are relative to `Engine/Source/Runtime/`.

## Locate the asset registry records

Use the summary's asset registry data offset. In a UE 5.8 or 5.7 editor
package, the section begins with an absolute `i64` offset to the dependency
data. The dependency data follows the asset records. Read through the
start of the dependency data to obtain the complete asset records. The
records have this layout.

| Field | Type |
|---|---|
| Asset record count | `i32` |
| Each asset record | Object path FString, class name FString, tag count `i32` |
| Each tag | Key FString, value FString |

Read each asset's tags before the next asset record. The engine omits the
dependency offset for old package versions before
`VER_UE4_ASSETREGISTRY_DEPENDENCYFLAGS` and for packages with
`PKG_FilterEditorOnly`. Source for both versions:
`AssetRegistry/Private/PackageReader.cpp`,
`FDeserializePackageData::DoSerialize`,
`FDeserializeObjectPackageData::DoSerialize` and
`FDeserializeTagData::DoSerialize`.

## Identify the external actor package

UE builds the standard external actor folder from the map package's mount
point, its folder path and its short name. It inserts `__ExternalActors__`
immediately after the mount point. Use that package path component to
select standard external actor packages. Confirm the selection with the
actor tags on one asset record. The `BP_SummaryActor` fixture shows that
the two tags also occur on Blueprint assets.

Sources: UE 5.8.3
`Engine/Private/Level.cpp`, `ULevel::GetExternalActorsPath`, and
`CoreUObject/Private/Misc/PackagePath.cpp`,
`FPackagePath::GetExternalActorsFolderName`. The two packages listed in
`tests/fixtures/ofpa/actors.expected.json` confirm this layout.

## Read the descriptor

`ActorMetaDataClass` gives the first native class in the actor class
hierarchy. `ActorMetaData` gives a Base64 string. After decoding, the bytes
start with a custom version container, followed by the actor descriptor
archive. The descriptor has different fields for a class default object
and an actor instance. The instance has an actor GUID. The descriptor's
`BaseClass` is the actor's own class path when that class is non-native.
The Blueprint package's `ParentClassPath` tag gives the immediate
Blueprint parent. The generated class export also stores that parent as
its super index. Resolve this index through the Blueprint package's
import or export table. The fixture's `ParentClass` tag gives
`/Script/Engine.Actor` for `BP_SummaryActor_C`.

Sources: UE 5.8.3
`Engine/Private/WorldPartition/WorldPartitionActorDescUtils.cpp`,
`AppendAssetDataTagsFromActor` and `GetAssetDataFromActorDescriptor`;
`Engine/Private/WorldPartition/WorldPartitionActorDesc.cpp`, `Init`,
`SerializeTo` and `Serialize`;
`Engine/Private/WorldPartition/WorldPartitionActorDescArchive.cpp`,
`FActorDescArchive::Init`;
`Engine/Private/Blueprint.cpp`, the asset registry tag writer;
`CoreUObject/Private/UObject/ObjectResource.cpp`, the export serialization
described in [Package format](package.md);
`BP_SummaryActor.pkginfo.txt`.

Both fixtures store the full actor object path in the asset registry
record. Take the package path before the first dot to obtain the outer
map path. The same record gives the actor class and the `ActorLabel`
tag. UE adds that tag for an external actor with a label when it is
not a child actor. Source: UE 5.8.3 `Engine/Private/Actor.cpp`,
`AActor::GetAssetRegistryTags`.

The descriptor's custom version count is an `i32`. Each entry has a
GUID followed by an `i32` version. The three gates for the supported
prefix are UE5 Main 88, Fortnite Season 5, and Fortnite Main 88.
The fixtures use 123, 15, and 268, respectively. The payload begins with
a `u32` default descriptor flag. Two class paths follow, each with
two FStrings for the package and class name. For an actor instance,
the GUID follows these paths. Stop at this GUID for point 5.

Sources: UE 5.7.4 and 5.8.3
`Core/Public/UObject/*ObjectVersions.inl` for the three version gates,
`Core/Private/UObject/DevObjectVersion.cpp` for their GUIDs, and
`WorldPartitionActorDescArchive.cpp` and `WorldPartitionActorDesc.cpp`
for the prefix. Both fixture GUIDs match the editor evidence.

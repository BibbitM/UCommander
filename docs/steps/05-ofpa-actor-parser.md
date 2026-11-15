# 05 – OFPA actor data parser

Branch: `pt/05-ofpa-actor-parser` (from `main`)

Features: none directly. Point 5 makes the parser for D3–D5 (point 6)
and B7 (point 13). It also moves D6 to the MVP (owner decision 7).
Release after this point: none (v0.3.0 comes after point 6, owner
decision 11).

## Goal

`uc-format` reads the actor data of an OFPA external actor package
(one file per actor, in `__ExternalActors__`): the actor label, the
actor GUID, the native class, the Blueprint class and the map of the
actor. The data comes from the package header only, so the content
plugin can use it in point 6 without a slow read. `uc dump` shows the
data, so that the owner and the agent can examine a package.

## Facts found before the brief

These facts come from the UE 5.8 source in the Epic Games Launcher
installation (`UE583`) and from fixture F2. Increment 1 records them in
`docs/formats/`, with the exact source.

1. The editor writes the actor data of an external actor into the
   asset registry data of its package: tag `ActorMetaDataClass` (the
   path of the native class) and tag `ActorMetaData` (the actor
   descriptor). Source: UE583
   `Engine/Private/WorldPartition/WorldPartitionActorDescUtils.cpp`.
2. The `ActorMetaData` value is Base64 text. The decoded bytes are a
   custom version list, then the descriptor fields. Source: same file,
   and `WorldPartitionActorDesc.cpp`, the function that serializes the
   descriptor to bytes.
3. The descriptor starts with: a "default descriptor" boolean, the
   Blueprint class path (`BaseClass`, empty for a native actor class),
   the native class path, the actor GUID. The label comes later, after
   the transform, the bounds, the references, the tags and the data
   layers. Names are stored as strings. Source:
   `WorldPartitionActorDescArchive.cpp` and `WorldPartitionActorDesc.cpp`.
4. Many fields use "delta serialization": a flag byte, then the value
   only when the value is different from the class default. The class
   default comes from a registry in the editor, not from the file. For
   the label, the class default is empty: UE sets the label only for an
   actor that is not a class default object. Source: same files.
5. The asset registry data is the last part of the package header.
   Observed in fixture F2 (`L_SummaryLevel.umap`): asset registry data
   offset 4878, size 1052 (`PkgInfo`), total header size 5930.
6. A level that is not OFPA also has actor descriptors: tag
   `ActorsMetaData` of the world, one entry for each actor. Observed in
   F2 (one entry, `PointLight_1`). Source of the use: UE583
   `Engine/Private/Level.cpp`. This is for point 12, not for point 5.
7. The console command `wp.Editor.DumpActorDescs <file>` writes the
   actor descriptors of the open World Partition map to a CSV file.
   Source: UE583 `Engine/Private/WorldPartition/WorldPartition.cpp`.
   I propose this output as the source of the expected values (owner
   action 3).
8. The descriptor fields up to the label have the same layout in UE
   5.6, 5.7 and 5.8. The last layout change before the label is the
   version `WorldPartitionActorDescSerializeRelativeTransform` of
   `FortniteMainBranchObjectVersion`: from this version, the descriptor
   has a relative transform and relative bounds, not an absolute
   transform and absolute bounds. The version name first exists in UE
   5.6. Each other version name that the read code uses up to the label
   exists in UE 5.5 or earlier. Sources: the custom version headers
   (`.h`, and `.inl` from 5.6) of GitHub `EpicGames/UnrealEngine`,
   branches `5.0` to `5.7`, and UE583. The read code of branch `5.6`
   (`WorldPartitionActorDesc.cpp`, `WorldPartitionActorDescArchive.cpp`
   and `.h`, the relative bounds type in `WorldPartitionActorDesc.h`)
   has the same version checks and fields as UE583 up to the label. The
   only differences are after the label (component descriptors).

## Acceptance criteria

1. `uc-format` reads the asset registry data of a package: for each
   object, the object path, the class name and the tags (key and value).
   A package with no asset registry data gives an empty list.
2. `uc-format` decodes the `ActorMetaData` tag and reads these values of
   the actor descriptor:

   | Value | Example (proposal for F5) |
   |---|---|
   | Actor GUID | from `DumpActorDescs` |
   | Native class path | `/Script/Engine.PointLight` |
   | Blueprint class path | none for a native actor, `/Game/Summary/BP_SummaryActor.BP_SummaryActor_C` for F6 |
   | Actor label | `Lampa żółta` |

3. `uc-format` gives the OFPA actor data of a package header in one
   call: the values of criterion 2, the actor path and the map path
   (see "Design"). A package that is not an external actor package gives
   no actor data. This is not an error.
4. An actor with no serialized label (delta flag 0) gives no label. This
   is not an error.
5. A damaged asset registry section, damaged Base64 text, or a damaged
   or truncated descriptor gives an error, never a panic and never an
   unbounded allocation. The package header itself stays valid: the
   asset class and the other point 3 and 4 values do not depend on the
   asset registry data.
6. The parser supports the descriptor layout of UE 5.6 to UE 5.8 (fact
   8, owner decision 3). A descriptor custom version that is newer than
   the newest UE 5.8.3 version gives a warning. The parser reads it with
   the newest known layout (as the summary rule of point 1). A
   descriptor with an older layout (UE 5.5 and earlier: a custom version
   that is less than the newest version name that the read code uses up
   to the label, or a missing custom version) gives an "unsupported"
   error.
7. Fixture tests: F5, F6 and the other owner actors give the expected
   values of criterion 2 and the map path. F7 (UE 5.7) gives its
   expected values. F1 and F3 give no actor data. F2 gives its asset
   registry tags and no actor data (it is not an external actor
   package).
8. `uc dump` shows the asset registry data and the OFPA actor data.
   New snapshots for F5 and F6. The F1 snapshot gets the new section.
9. All checks pass.

## Design

### Asset registry data (`uc-format`)

A new module `package/registry.rs`. The layout comes from UE583
`AssetRegistry/Private/PackageReader.cpp`, the function that reads the
main package data, and from `CoreUObject/Private/UObject/SavePackage/
SavePackageUtilities.cpp`, the function that writes it. Increment 1
records the layout and the version conditions.

| Item | Content |
|---|---|
| `AssetRegistryData` | The list of objects. |
| `AssetRegistryObject` | Object path, class name, tags (a list of key and value pairs, in file order). |
| `read_asset_registry(bytes, &PackageHeader)` | Reads the section from the asset registry data offset of the summary. |

The section is separate from `read_header`, so that a damaged asset
registry section does not stop the point 3 and point 4 fields
(criterion 5). `read_header_from` stops when the tables are complete,
and it does not give its bytes. A new function `read_header_bytes_from`
reads up to the total header size and gives the bytes (see "Changes
during the work").

### Base64

UE uses `FBase64` in the standard mode (alphabet `A–Z a–z 0–9 + /`,
with `=` padding). Increment 1 checks this. I propose a small decoder in
`uc-format` (about 50 lines), with the test vectors of RFC 4648 §10.
The decoded size is at most 3/4 of the text size, so the allocation has
a known limit. Alternative: the `base64` crate (gate, open question 2).

### Actor descriptor (`uc-format`)

A new module, for example `world_partition/actor_desc.rs`. Increment 1
records the layout in a new file `docs/formats/world-partition.md`.

- The custom version list at the start gives the layout. These custom
  versions control the layout: `UE5MainStreamObjectVersion`,
  `UE5ReleaseStreamObjectVersion`, `FortniteSeasonBranchObjectVersion`,
  `FortniteMainBranchObjectVersion`. Increment 1 records their GUIDs and
  the values of the version names that the descriptor uses, from the
  UE 5.8.3 enum order, as for `ObjectVersion.h` in point 1.
- The parser reads the base descriptor fields up to and including the
  label. It does not read the fields after the label (HLOD, folder,
  parent, properties, components) or the extra data of descriptor
  subclasses (level instance, landscape, HLOD). Increment 1 checks that
  each subclass writes its extra data after the base fields.
- The parser keeps only the values of criterion 2 (owner decision 1).
  It reads and skips the other fields on the way (transform, bounds,
  grid, data layers). They are D6, which comes in point 6 (owner
  decisions 7 and 11).
- Supported layouts: one layout for UE 5.6, 5.7 and 5.8 (fact 8). The
  parser implements only the branches of the UE 5.8.3 read code that
  apply to this layout. Increment 1 records the minimum value of each of
  the four custom versions. UE 5.6 has no fixture: the owner has no UE
  5.6 installation. F7 (UE 5.7) and F5, F6 (UE 5.8) test the layout. The
  UE 5.9 layout comes when UE 5.9 is released (owner decision 3).
- A delta field with flag 0 has the class default value. The file does
  not give it. For the label, the result is "no label" (fact 4). For the
  skipped fields, the value is not necessary.
- A descriptor with the "default descriptor" flag set is a class
  descriptor, not an actor. `read_actor_desc` gives an error for it.
  A Blueprint package of an actor class contains one (observed in F1).

### OFPA actor data of a package

| Value | Source |
|---|---|
| Actor path | The object path of the asset registry object that has the `ActorMetaData` tag. Example: `/Game/Ofpa/L_OfpaMap.L_OfpaMap:PersistentLevel.PointLight_UAID_…`. |
| Map path | The package part of the actor path (text before the first `.`): `/Game/Ofpa/L_OfpaMap`. The same form as the **Package path** field of point 4 (owner decision 4). |
| Actor class | Not new. The **Asset class** of point 3 gives the class of the main actor (`PointLight`, `BP_SummaryActor_C`). |
| Values of criterion 2 | The decoded descriptor. |

A package is an external actor package when its asset registry data has
an object with these properties (owner decision 8):

- Its object path is a full path: it starts with `/`.
- It has the `ActorMetaData` tag.
- Its descriptor is not a class descriptor.

F1 (a Blueprint of an actor class) also has the `ActorMetaData` tag, on
an object with a relative path, and its descriptor is a class
descriptor. So the tag alone does not identify an external actor
package. A class descriptor gives no actor data. This is not an error.

If more than one object agrees with the rule, the parser uses the first
one and gives a warning. The `PkgInfo` output of F5 and F6 shows if UE
writes one object only.

Type names may change a little during the work. I write the final names
in the PR.

### `uc dump`

New sections after the export table:

```
Asset registry dependency data offset: 3041
Asset registry object count: 1
Asset registry object 0 path: /Game/Ofpa/L_OfpaMap.L_OfpaMap:PersistentLevel.PointLight_UAID_…
Asset registry object 0 class: /Script/Engine.PointLight
Asset registry object 0 tag count: 3
Asset registry object 0 tag ActorLabel: Lampa żółta
Asset registry object 0 tag ActorMetaData: (672 characters)
Asset registry object 0 tag ActorMetaDataClass: /Script/Engine.PointLight
Actor path: /Game/Ofpa/L_OfpaMap.L_OfpaMap:PersistentLevel.PointLight_UAID_…
Actor map: /Game/Ofpa/L_OfpaMap
Actor GUID: 00B22501-45F611EC-B769568D-0F1275DB
Actor label: Lampa żółta
Actor native class: /Script/Engine.PointLight
Actor Blueprint class: (none)
```

A package with no external actor gives the line `OFPA actor: (none)`.
A damaged section gives the line `Asset registry error: …` or
`Actor error: …`, and the header lines stay.

The `ActorMetaData` value is long Base64 text, so `uc dump` shows its
length only. The other tag values are shown in full. An empty value is
`(empty)`. The existing F1
snapshot gets the asset registry section. This is display text, not an
expected value (`AGENTS.md` §6).

## Fixture request

The owner makes these assets in the fixtures project with stock UE
5.8.3.

1. A new level `/Game/Ofpa/L_OfpaMap`: File → New Level → "Empty Open
   World". World Partition and OFPA are on. Save it.
2. In this level, add two actors and save:
   - F5: a **Point Light**. Set the label to `Lampa żółta` (Polish
     letters, to test UTF-16 text). Default settings.
   - F6: an instance of the existing Blueprint `BP_SummaryActor`. Set
     the label to `Summary Actor`. Put it in the actor folder
     `Group/Sub` (for later points).
   - Other actors that the owner adds (owner decision 6). Tell me the
     kind and the label of each. Useful cases: an actor in a data layer,
     an actor that is not spatially loaded, a non-default runtime grid,
     and a Blueprint actor whose Blueprint has a Blueprint parent. They
     also help D6 (point 6, owner decision 11).
3. Copy into the repository (Git LFS), with the folder structure of
   `Content`:
   - `tests/fixtures/editor/Ofpa/L_OfpaMap.umap` (F4, for point 13),
   - the full folder `Content/__ExternalActors__/Ofpa/L_OfpaMap/` to
     `tests/fixtures/editor/__ExternalActors__/Ofpa/L_OfpaMap/`. It
     contains F5, F6, the other actors and the actors of the template
     (for example `WorldDataLayers`). Tell me which file is which actor.
   - If a data layer is used: its data layer asset and the
     `__ExternalObjects__` folder of the map, with the same structure.
4. The `PkgInfo` output of F4, F5, F6 and the other actor packages, as
   for F1–F3.
5. With the level open: run the console command
   `wp.Editor.DumpActorDescs <path>.csv`, and give me the CSV file. I
   add it to the fixtures folder as the source of the expected values.
6. UE 5.7 (owner decision 3), in a UE 5.7 project: a level
   `/Game/Ofpa/L_OfpaMap57` from "Empty Open World", with one Point
   Light with the label `Lampa żółta` (F7). Copy its external actor
   package to `tests/fixtures/editor-5.7/__ExternalActors__/Ofpa/
   L_OfpaMap57/`, with its `PkgInfo` output and the
   `wp.Editor.DumpActorDescs` CSV of UE 5.7. The map file is not
   necessary.

Expected values (`.expected.json` of each actor package, new keys
only):

| Key | Source |
|---|---|
| `actor_guid` | `DumpActorDescs` CSV. Also visible in the Details panel (Actor → Actor Guid). |
| `actor_label` | `DumpActorDescs` CSV, and the Outliner. |
| `actor_native_class_path` | `DumpActorDescs` CSV. |
| `actor_blueprint_class_path` | `DumpActorDescs` CSV (base class column). `null` for F5. |
| `actor_path` | The asset registry section of the `PkgInfo` output (it shows `/Temp/…` for the package, see point 4; I replace `/Temp` with `/Game` and write this in the `source` key, owner decision 5). |
| `map_path` | `/Game/Ofpa/L_OfpaMap` (`/Game/Ofpa/L_OfpaMap57` for F7), from the editor. |
| `asset_class`, `asset_class_path`, `package_path` | As for F1–F3 (point 3 and 4 keys). |

### Result

The owner made the fixtures (UE 5.8.3 and UE 5.7.4). Each folder has
an `EditorEvidence.json` file with the editor values of each actor.

| Fixture | Package (`__ExternalActors__/Ofpa/…`) | Actor |
|---|---|---|
| F5 | `L_OfpaMap/9/UY/KPHXLC180ULM321RTVNKBI` | Point Light `Lampa żółta` |
| F6 | `L_OfpaMap/6/VM/S206VJ57URX2IZEGIAFTJ8` | `BP_SummaryActorRenamed_C`, `Summary Actor`, folder `Group/Sub` |
| F8 | `L_OfpaMap/1/5X/LDR80LGXHNUA4U4CBHWYNV` | Target Point `Always Loaded Target`, not spatially loaded |
| F9 | `L_OfpaMap/4/TV/YHL37N60V5A37O1U3ASTQ7` | Text Render Actor `Custom Grid Text`, runtime grid `FixtureGrid` |
| F10 | `L_OfpaMap/7/GM/CD2EGZM9LI8RXFXKWX2IC8` | `BP_FixtureChild_C` (parent `BP_FixtureParent_C`), `Child Blueprint Actor` |
| F11 | `L_OfpaMap/C/LG/7A5579ZXJXD32CJYP3WICE` | Spot Light `Data Layer Spot`, data layer `DL_OfpaFixture` |
| F12 | `L_OfpaMap/C/N1/EXGB2JTMGHKCGRQDEZBUFA` | `WorldDataLayers` (template) |
| F13 | `L_OfpaMap/B/DH/DPUOQ7MHWJ6AFJ48NJZJ24` | `WorldPartitionMiniMap` (template) |
| F7 | `L_OfpaMap57/B/I4/4LDIAYPOR4RGIIJ2QNMPSY` (UE 5.7) | Point Light `Lampa żółta` |
| F14 | `L_OfpaMap57/3/5F/UY7CAHIHPCHXPYHP6UAZC4` (UE 5.7) | `WorldDataLayers` (template) |
| F15 | `L_OfpaMap57/5/2D/65F9D6M5A1KPQ0QFCEMT5H` (UE 5.7) | `WorldPartitionMiniMap` (template) |

Differences from the request:

1. `/Game/Summary/BP_SummaryActor` is a redirector to
   `BP_SummaryActorRenamed` in the fixtures project. So the Blueprint
   class of F6 is
   `/Game/Summary/BP_SummaryActorRenamed.BP_SummaryActorRenamed_C`.
2. UE writes the CSV with the ANSI code page, so the label of F5 and F7
   is `Lampa ???ta` in it. The expected label comes from the
   `EditorEvidence.json` file (`GetActorDescs` in the editor). The
   `ActorLabel` tag in the `PkgInfo` output gives the same label.
3. The asset registry object path of an actor in the `PkgInfo` output
   starts with `/Game`, not `/Temp`. The `/Temp` replacement of owner
   decision 5 is not necessary.
4. The HLOD layer assets of the template, the map of F7 and the actor
   Blueprints of F10 are not in the repository. No test uses them.

A script compared the three sources (CSV, `EditorEvidence.json`,
`PkgInfo`) for each actor, and wrote the `.expected.json` files. All
values agree.

## Increments (one commit each)

| # | Commit title | Content |
|---|---|---|
| 0 | Add point 5 step brief | This file, after approval. |
| 1 | Add asset registry and actor desc notes | `docs/formats/package.md`: the asset registry data layout and its version conditions. New `docs/formats/world-partition.md`: the tags, Base64, the descriptor layout up to the label, the custom versions and their values, the delta flag, the subclass rule, the UE 5.7 comparison. With the source. No code. |
| 1a | Update the feature scope | `FEATURES.md`: D6 to `MVP`. `PLAN.md`: new point 6a with D6 and the release v0.3.0; point 6 has no release (owner decision 7). |
| 2 | Add OFPA fixtures | The files of the fixture request, `.expected.json` of F4–F7 and the other actors, the CSV files. No code. |
| 3 | Read the asset registry data of a package | `read_asset_registry`. Synthetic tests: no objects, one object with tags, 8-bit and UTF-16 text. Robustness: truncated section, bad offset, bad count. |
| 4 | Add asset registry fixture tests | F2 tags from its `PkgInfo` output (new key `asset_registry` in its `.expected.json`; existing keys do not change). F5 and F6 tags. |
| 5 | Decode Base64 text | Decoder and RFC 4648 vectors. Robustness: bad character, bad padding, bad length. |
| 6 | Read the actor descriptor | Synthetic descriptors: native actor, Blueprint actor, no label (flag 0), UTF-16 label, newer custom version (warning), UE 5.5 layout and missing custom version (error), "default descriptor" flag (error). Robustness: truncated. |
| 7 | Give the OFPA actor data of a package | The combined call, map path. Fixture tests: F5, F6, the other actors and F7 values, F1–F3 no actor data. |
| 8 | Show asset registry and actor data in dump | `uc dump` sections. Snapshots: F1 changed, F5 and F6 new. |
| 9 | Update changelog for point 5 | `CHANGELOG.md` `[Unreleased]`: the `uc dump` sections. README does not change: no new plugin feature. |
| 10 | Set point 5 status to done | `docs/PLAN.md`. The last commit, after the review. |

Each feature increment follows the TDD sequence in `AGENTS.md` §6.

### Changes during the work

| Commit | Change |
|---|---|
| Increment 4 | The test reads the expected objects and tags from the `PkgInfo` output directly. No new `asset_registry` key in `.expected.json`: the `FiBData` value of F1 has control characters that a hand copy cannot keep. |
| Record the external actor package rule | Owner decision 8, after the F1 finding. |
| Add OFPA asset registry fixture tests | The asset registry tests of F5, F6 and the other new packages (part of increment 4, after the fixtures came). |
| Share descriptor and registry test builders | Refactor of the test helpers, for increment 7. |
| Add OFPA actor fixture tests | The fixture part of increment 7, in its own commit. |
| Read the header bytes for later sections | New `read_header_bytes_from`, for `uc dump` (increment 8) and the content plugin (point 6). |
| Test descriptor fields with flag 1, Test a damaged total header size, Record class redirects and actor error rules | Findings of the branch review. |
| Move D15 to point 6a | Owner decision 10, after the PR was opened. |
| Merge the points 6 and 6a | Owner decision 11, after the PR was opened. |

Open item for point 6: `read_asset_registry` has no limit on the object
and tag count, other than the header size. A damaged 16 MiB header with
many empty tags can use about 190 MB of memory (branch review). Point 6
decides a limit before the content plugin uses the function.

## New dependencies (gate)

None. The Base64 decoder is our own code (owner decision 2).

## Owner actions

1. Approve this brief. Done.
2. Answer the open questions. Done, see "Owner decisions".
3. Make the fixtures of the fixture request (before increment 2). I can
   do increments 1, 1a, 3, 5 and 6 with synthetic data before the
   fixtures come.

## Out of scope

- Content plugin fields (point 6).
- The packer view of the external actors of a map (point 13). Point 5
  adds the map fixture F4 only.
- The `ActorsMetaData` tag of a level that is not OFPA (point 12).
- D6 values (data layers, runtime grid, spatially loaded flag, bounds).
  Point 6 adds them to the parser and to the content plugin (owner
  decisions 7 and 11).
- The actor folder, the parent actor and the descriptor fields after
  the label.
- Descriptor subclass data (level instance, landscape, HLOD).
- The actor label from the export data (the `ActorLabel` property). The
  parser does not read export data.
- Cooked packages: the cook removes the editor-only asset registry tags.
- Descriptor layouts of UE 5.5 and earlier, and the UE 5.9 layout
  (owner decision 3).

## Owner decisions

1. Read the base descriptor up to the label. Keep only the values of
   criterion 2.
2. Use our own Base64 decoder, with tests. No new dependency.
3. UE 5.8 has priority. Fixtures from UE 5.7 are welcome. The parser
   supports each version back to the last layout change, and stops at
   the version before the change. The last change is in UE 5.6 (fact 8),
   so the parser supports UE 5.6 to 5.8, and UE 5.5 is not supported.
   UE 5.9 (now a preview) comes later: the whole plugin gets an update
   when UE 5.9 is released.
4. The map path is the package path of the map
   (`/Game/Ofpa/L_OfpaMap`). It is shorter.
5. The expected actor path comes from `PkgInfo`, with `/Temp` replaced
   by `/Game`.
6. The owner adds more actors to the fixture level.
7. D6 (OFPA data layers, runtime grid, spatially loaded flag, bounds)
   changes from `Later` to `MVP`. It is a new plan point 6a after point
   6. The release v0.3.0 moves from point 6 to point 6a.
8. An external actor package has an asset registry object with a full
   object path, the `ActorMetaData` tag and a descriptor that is not a
   class descriptor (see "OFPA actor data of a package"). The owner
   accepted this rule after the F1 finding during the work.
9. The descriptor reader ignores the custom versions that are not one of
   the four layout versions, with no warning. This is an exception to
   `AGENTS.md` §5 rule 7. These versions control only data that
   UCommander does not read (subclass data, fields after the label), so
   a warning gives no useful information. The owner accepted this after
   the branch review.
10. D15 (sample TC custom column set) moves from point 6 to point 6a.
    The column set then shows all OFPA fields, also the D6 fields. The
    owner decided this after the PR was opened.
11. The points 6 and 6a become one point 6, "Content: OFPA actor
    fields", with D3 to D6, D15 and the release v0.3.0. The D6 fields
    come from the same descriptor, before the label, and the fixtures
    and the editor evidence already have their values. This replaces
    the new point 6a of decision 7 and the move of decision 10. The
    owner decided this after the PR was opened.

## Open questions (answered)

The answers are in "Owner decisions".

1. **Scope of the descriptor.** I propose to read the base descriptor up
   to the label, and to keep only the values of criterion 2. Alternative:
   also keep the D6 values that the parser reads on the way (grid,
   spatially loaded, data layers, bounds), or read up to the actor
   folder path (for the features F2 and B8, later). Each extra value needs its own
   tests and expected values.
2. **Base64.** I propose an own decoder (about 50 lines, no dependency).
   Alternative: the `base64` crate (MIT OR Apache-2.0), a new
   dependency.
3. **Supported descriptor layouts.** I propose only the layout of UE
   5.8.3 (all four custom versions at least at the newest value that
   changes the fields up to the label). Older descriptors give an
   "unsupported" error. UE 5.8 is required, and earlier versions are
   best effort (X1). Alternative: all the old branches of the UE 5.8.3
   read code, with synthetic tests only. Or: also the layouts of UE 5.5
   and 5.7, with fixtures from those engines (both are installed on the
   owner's computer).
4. **Map path form.** I propose the package path of the map
   (`/Game/Ofpa/L_OfpaMap`), the same form as the **Package path**
   field. Alternative: the object path of the world
   (`/Game/Ofpa/L_OfpaMap.L_OfpaMap`).
5. **Expected actor path.** `PkgInfo` shows `/Temp/…` paths (point 4).
   I propose to take the actor path from `PkgInfo` and replace `/Temp`
   with `/Game`, and to record this in the `source` key. Alternative:
   the owner copies the actor path from the editor (right-click the
   actor → Copy Reference, if the menu has it for an actor).
6. **Fixture actors.** Are a Point Light and a `BP_SummaryActor`
   instance enough? I propose to add no static mesh actor: it adds no
   new case for the parser.

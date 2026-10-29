# UCommander – feature scope

This file lists the agreed features. Each feature has an ID. Plan points,
step briefs, commits and PRs refer to features by ID.

Status values:

| Status | Meaning |
|---|---|
| MVP | Part of the MVP (plan points 1–15). |
| Next | First item after the MVP. |
| Later | Agreed, but after the MVP. The order is in `PLAN.md`. |
| Dropped | Not in scope now. We can reconsider it later. |

## Packer plugin (UCommanderPacker, WCX) – containers

| ID | Feature | Status |
|---|---|---|
| A1 | List the entries of a `.pak` file as a folder tree, with the mount point. | MVP |
| A2 | List the entries of an IoStore container (directory index from `.utoc`). | MVP |
| A3 | Show the uncompressed size and the compressed size of each entry. | MVP |
| A4 | Show the compression method and the encryption flag of each entry. | MVP |
| A5 | Extract entries with F5 (uncompressed, Zlib and LZ4 entries). | MVP |
| A6 | Extract a package with its companion files as one unit. | Later |
| A7 | Show cooked packages in a container as folders. | Later |
| A8 | Show container metadata as a virtual `!info.txt` entry. | Later |
| A9 | Verify entry hashes with TC "Test archive". | Later |
| A10 | Read AES keys from local configuration. | Later |
| A11 | Show more containers of one game as one merged view. | Later |
| A12 | Replace an entry in a `.pak` file. | Later |

## Packer plugin (UCommanderPacker, WCX) – packages and maps

| ID | Feature | Status |
|---|---|---|
| B1 | List the objects in a package, with class and serial size. | MVP |
| B2 | Show the object hierarchy (outer chain) as folders. | Later |
| B3 | Show the imports as a virtual `!imports` folder. | Later |
| B4 | Extract one object as a text property dump. | Later |
| B5 | Extract an object in a converted format (PNG, WAV, CSV, JSON). | Later |
| B6 | List the actors of a legacy `.umap`. | MVP |
| B7 | List the external actors of an OFPA `.umap`, with friendly names. | MVP |
| B8 | Group World Partition actors by data layer, grid or actor folder. | Later |

## Lister plugin (UCommanderLister, WLX)

| ID | Feature | Status |
|---|---|---|
| L1 | Show the package summary. | MVP |
| L2 | Show the editor thumbnail of a package. | MVP |
| L3 | Supply the thumbnail to the TC thumbnail view. | MVP |
| L4 | Show the name map, import table and export table. | MVP |
| L5 | Show the asset registry tags of a package. | Later |
| L6 | Decode and show texture mips. | Later |
| L7 | Show the properties of an object as a tree. | Later |
| L8 | Text-only mode for F3 search, copy and "Compare by content". | Later |
| L9 | Show a reference graph. | Later |
| L10 | Play sound wave previews. | Later |
| L11 | Show simple mesh previews. | Later |
| L12 | Show a container summary. | Later |

## Content plugin (UCommanderContent, WDX)

| ID | Field | Status |
|---|---|---|
| D1 | Asset class. | MVP |
| D2 | Asset name and object path. | MVP |
| D3 | OFPA: actor label. | MVP |
| D4 | OFPA: actor class, native base class, Blueprint parent class. | MVP |
| D5 | OFPA: actor GUID and outer map path. | MVP |
| D6 | OFPA: data layers, runtime grid, spatially loaded flag, bounds. | Later |
| D7 | Engine version and package file version. | MVP |
| D8 | Cooked or editor, unversioned properties, editor-only data. | Later |
| D9 | Thumbnail present. | Later |
| D10 | Total size including companion files. | Later |
| D11 | Texture fields: size, pixel format, mip count, LOD group, sRGB. | Later |
| D12 | Mesh fields: LOD count, vertex count, material count. | Later |
| D13 | Dependency count and first dependencies. | Later |
| D14 | Edit values. | Later |

## File system plugin (WFX)

| ID | Feature | Status |
|---|---|---|
| F1 | Show a project as a virtual `/Game` tree. | Later |
| F2 | Show OFPA actors with friendly names, in actor folders. | Later |
| F3 | Show a game build as one virtual tree. | Later |
| F4 | Show the asset registry contents. | Later |

## Cross-cutting

| ID | Topic | Status |
|---|---|---|
| X1 | UE 5.8 is required. Earlier UE 5.x versions are best effort. README lists the limits. | MVP |
| X2 | Cooked packages: Zen package headers. | Next |
| X3 | Oodle decompression with a DLL that the user supplies. Never in the repository. | Later |
| X4 | AES keys in local configuration. Never in the repository. | Later |
| X5 | One local configuration file for the plugins. | Later |
| X6 | Find the `.uproject` above the current path. Necessary for object paths (D2). | MVP |
| X7 | Header-only parsing and a cache. The content plugin never blocks. | MVP |
| X8 | Read-only until a write feature (A12, D14). | MVP |
| X9 | Win64 only. TC plugin installer ZIPs (`pluginst.inf`). | MVP |
| X10 | Check the license of each reference before we use it. No engine code. | MVP |

## Bridge (UE editor plugin)

| ID | Feature | Status |
|---|---|---|
| U1–U9 | Editor connection, editor actions, engine-based previews, UnrealPak backend, `uetc` CLI. | Dropped |

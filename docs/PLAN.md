# UCommander – development plan

Each plan point has one branch, one step brief (`docs/steps/`), and one PR.
Feature IDs are in [FEATURES.md](FEATURES.md). The workflow is in
[AGENTS.md](../AGENTS.md).

Status values: `Planned`, `In progress`, `In review`, `Done`.

## Phase 0 – foundation and walking skeleton

| Pt | Name | Content | Release | Status |
|---|---|---|---|---|
| P0 | Repository setup | Workspace, licenses, README, AGENTS.md, docs, lints, cargo-deny, PR template, RustRover run configurations. | — | Done |
| P1 | CI | `ci.yml`: format check, lint, test, cargo-deny, release build. Branch protection on `main`. | — | In review |
| P2 | Content plugin skeleton | `uc-tc` content API. `uc-content` with one field: "Is package" (package file tag check). FFI test harness. | — | In review |
| P3 | Package and install | `cargo xtask check`, `cargo xtask package` (ZIPs with `pluginst.inf`), `cargo xtask install`. | — | Planned |
| P4 | Release workflow | `release.yml`: tag `v*` → build, test, ZIPs, GitHub Release. | v0.1.0 | Planned |

## Phase 1 – MVP

| Pt | Name | IDs | Release | Status |
|---|---|---|---|---|
| 1 | Package summary parser and `uc dump` | — | | Planned |
| 2 | Name map, import table, export table | — | | Planned |
| 3 | Content: asset class | D1 | v0.2.0 | Planned |
| 4 | Content: name, path, engine version | D2, D7 | | Planned |
| 5 | OFPA actor data parser | — | | Planned |
| 6 | Content: OFPA label, class, GUID, map | D3–D5 | v0.3.0 | Planned |
| 7 | Lister skeleton: summary | L1 | | Planned |
| 8 | Lister: tables view | L4 | v0.4.0 | Planned |
| 9 | Thumbnail in the lister | L2 | | Planned |
| 10 | Thumbnail in the TC thumbnail view | L3 | v0.5.0 | Planned |
| 11 | Packer: objects in a package | B1 | | Planned |
| 12 | Packer: actors in a legacy map | B6 | | Planned |
| 13 | Packer: external actors of an OFPA map | B7 | v0.6.0 | Planned |
| 14 | `.pak` read and extract | A1, A3–A5 | v0.7.0 | Planned |
| 15 | IoStore read and extract | A2–A5 | v0.8.0 (MVP) | Planned |

## After the MVP

The order can change.

1. Zen package headers, for cooked packages (X2).
2. Oodle and AES (X3, X4, A10).
3. Cooked packages as folders in containers, container info and verification
   (A7, A8, A9, L12).
4. Asset registry tags, text-only view, property tree (L5, L8, L7).
5. Texture decode and object export (L6, B4, B5).
6. Extended content fields and map grouping (D6, D8–D13, B8).
7. Write features (A12, D14).
8. Merged container view, WFX, mesh and sound preview, reference graph
   (A11, F1–F4, L10, L11, L9).
9. Reconsider the bridge (U1–U9).

Version 1.0.0 comes after the MVP, when the field names of the content plugin
are stable.

## Decisions

| Topic | Decision |
|---|---|
| Language | Rust. |
| Platform | Windows x64 only. TC 11.0 or later. |
| License | `MIT OR Apache-2.0`. |
| Branches | `pt/NN-short-name` from `main`. Merge commit. |
| Review | The agent commits on the branch. The owner reviews the branch at the PR. A second agent reviews each PR first. |
| Fixtures | Hybrid: fixture UE project in a separate repository, curated fixture files in `tests/fixtures/` (Git LFS), large containers as fixture-repository release files, synthetic fixtures in test code. |
| Versioning | SemVer. One workspace version. Tag `vX.Y.Z` on `main` starts a release. |

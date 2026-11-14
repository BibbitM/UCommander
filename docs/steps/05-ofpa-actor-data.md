# 05 – OFPA actor data parser

Branch `pt/05-ofpa-actor-data` from `main`.

## Goal

Read One File Per Actor (OFPA) data from UE 5.8 editor packages in
`uc-format`. Give point 6 the data for D3–D5. Read the actor label, actor
class, native base class, Blueprint parent class when present, actor GUID,
and outer map package path.

## Engine versions

Use the UE 5.8 layout as the required baseline. Compare it with the UE 5.7
layout and handle known differences on a best-effort basis. Test with a
UE 5.7 fixture if one is available. Use UE 5.9 Preview for compatibility
tests only. Record differences found in Preview tests for later work.

## Acceptance criteria

1. A public parser reads an external actor package from bytes supplied by the
   caller. The parser does no file I/O. Document the byte ranges that point
   6 must read.
2. A supported external actor package gives the actor data. An ordinary
   editor package gives no actor data. Include the existing
   `BP_SummaryActor.uasset` fixture in this test. Its `PkgInfo` output
   contains an `ActorMetaData` tag.
3. Confirm the source of each field and the conditions for identifying an
   external actor package against UE 5.8 source and fixtures. Record the
   source file and version or the fixture in `docs/formats/ofpa.md`. Record
   the layout, version checks, and how to obtain the outer map package path.
4. Reuse the package summary and table readers. Read only the data needed
   for D3–D5. Check offsets, counts and lengths against the available bytes
   before use. Truncated or damaged data gives an error with context. It
   never causes a panic or an unbounded allocation.
5. Skip unknown data when the format gives a safe way to skip it. Report a
   warning. If an unknown layout prevents decoding the actor data, report
   that limitation without invalidating the package header.
6. Compare UE 5.8 fixtures with `.expected.json` values from the editor.
   Cover a native actor and an actor with a Blueprint parent. Use synthetic
   tests for damaged data and version handling.
7. `cargo xtask check` passes.

## Fixture request

Use the stock UE 5.8 engine and the fixtures project. Make a small World
Partition map with OFPA enabled. Place and save one native actor, such as
a Point Light. Make a Blueprint derived from Actor and a child Blueprint
derived from it. Place and save an instance of the child Blueprint. Give
both actors distinct labels that differ from their object names.

Supply the map and the two external actor packages. Keep their paths
relative to the project Content folder. Supply the UE editor version and
editor evidence for each actor's label, class path, native base class path,
Blueprint parent class path when present, actor GUID, and outer map package
path. Include `PkgInfo` output for each external actor package if the
commandlet can read it. Use owner-made assets.

## Increments

| # | Commit title | Content |
|---|---|---|
| 0 | Add point 5 step brief | Commit this brief after approval. |
| 1 | Document OFPA actor data | Trace the UE 5.8 read and write paths. Confirm the fixture layout and record the sources. |
| 2 | Locate external actor metadata | Identify external actor packages and locate the data needed for decoding. Test ordinary packages and damaged ranges. |
| 3 | Decode OFPA actor descriptors | Read the fields required by D3–D5. Split the work into smaller commits if the format needs it. Test truncation, invalid lengths and unknown versions. |
| 4 | Verify OFPA actor fixtures | Add the owner-made fixtures with Git LFS and compare the parser results with editor values. |
| 5 | Update the changelog for point 5 | Record the parser change. Update the README if needed. |
| 6 | Set point 5 status to done | Update `docs/PLAN.md` in the last commit after the branch review. |

Follow the TDD sequence in `AGENTS.md` §6 for parser changes.

## Out of scope

- Total Commander fields and the sample column set (point 6).
- Listing external actors from a map (point 13).
- Data layers, runtime grid, spatially loaded flag and bounds (D6, point 6a).
- General asset registry tag display (L5), cooked packages and package writes.

## Owner actions

1. Approve this brief before implementation.
2. Supply the fixtures and editor values.
3. Approve the push and PR when the branch is ready.

## Open questions

- Can the editor provide the actor GUID and class paths for each fixture?
  If these values need an editor command, agree on the command before
  recording the expected values.
- Confirm the on-disk source and version conditions for each field in
  increment 1. Bring any unresolved format details to the owner before
  implementing them.

# Changelog

All notable changes to UCommander are in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project uses [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Package summary parser for UE 5.8 editor packages, with errors for
  unsupported packages (UE4, unversioned, byte-swapped, compressed).
- Development tool `uc dump <file>`: shows the package summary as text.
- Name map, import table and export table parser for UE 5.8 editor
  packages, with the object name, class name and full path of each object.
  A damaged name index, package index or outer chain gives an error.
- `uc dump` also shows the name map, the import table and the export table.

## [0.1.0] - 2026-11-04

### Added

- Repository setup: Cargo workspace, licenses, agent instructions, feature
  scope and plan.
- CI on GitHub Actions: format check, lint, test, release build and
  dependency check for each PR and for each push to `main`.
- `cargo xtask` tasks: `check`, `package`, `install`, `plan-status`,
  `tag-version` and `release-notes`.
- Release workflow: a version tag on `main` publishes a GitHub Release with
  the plugin ZIP and `SHA256SUMS.txt`.
- Plugin ZIP with TC auto-install, the README, the changelog, the licenses, the
  feature scope and the plan.
  CI keeps the ZIP of each run for 14 days.
- CI check: a plan point PR must set its status to `Done`.
- `UCommanderContent` content plugin with the field **Is package**: "Yes"
  for a `.uasset` or `.umap` file that starts with the UE package file tag.

### Changed

- The plugins link the C runtime statically, so they need no VC++ runtime.

[Unreleased]: https://github.com/BibbitM/UCommander/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/BibbitM/UCommander/releases/tag/v0.1.0

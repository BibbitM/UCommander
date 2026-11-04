# Changelog

All notable changes to UCommander are in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project uses [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-11-04

### Added

- Repository setup: Cargo workspace, licenses, agent instructions, feature
  scope and plan.
- CI on GitHub Actions: format check, lint, test, release build and
  dependency check for each PR and for each push to `main`.
- `cargo xtask` tasks: `check`, `package`, `install` and `plan-status`.
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

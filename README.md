# UCommander

[![CI](https://github.com/BibbitM/UCommander/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/BibbitM/UCommander/actions/workflows/ci.yml)

Total Commander plugins for Unreal Engine files.

Source code: https://github.com/BibbitM/UCommander

> **Status:** In development. Releases: https://github.com/BibbitM/UCommander/releases

## Purpose

UCommander shows the contents of Unreal Engine files in
[Total Commander](https://www.ghisler.com/). It adds three plugins:

| Plugin | TC plugin type | Use |
|---|---|---|
| `UCommanderContent` | Content (`.wdx64`) | Columns, search fields and rename fields for packages. Examples: asset class, OFPA actor label, engine version. |
| `UCommanderLister` | Lister (`.wlx64`) | Package summary, tables and thumbnail in F3 and in Quick View. Thumbnails in the TC thumbnail view. |
| `UCommanderPacker` | Packer (`.wcx64`) | Open `.uasset`, `.umap`, `.pak` and `.utoc`/`.ucas` files like archives. |

See [docs/FEATURES.md](docs/FEATURES.md) for the full feature scope, and
[docs/PLAN.md](docs/PLAN.md) for the development plan.

## Requirements

- Total Commander 11.0 or later, 64-bit.
- Unreal Engine 5.8 packages. Earlier 5.x versions are best effort.

## Installation

1. Download the file `UCommanderContent-<version>.zip` from the
   [latest release](https://github.com/BibbitM/UCommander/releases/latest).
   To check the download, compare its SHA-256 hash with `SHA256SUMS.txt`.
2. Open the ZIP in Total Commander with Enter.
3. Accept the installation.
4. Make a custom column with a field of `UCommanderContent`.

The plugin needs no extra runtime.

### Content fields

| Field | Value |
|---|---|
| Is package | "Yes" when the file starts with the UE package file tag. |
| Asset class | The class of the main asset of the package, for example `Blueprint`, `World` or `StaticMesh`. |
| Asset class path | The class path of the main asset, for example `/Script/Engine.Blueprint`. |
| Asset name | The name of the main asset, for example `BP_Hero`. |
| Object path | The object path of the main asset, for example `/Game/Heroes/BP_Hero.BP_Hero`. |
| Package path | The package path, for example `/Game/Heroes/BP_Hero`. It comes from the package, not from the file path. Empty when the package has no valid package name. |
| Engine version | The engine version that saved the package, for example `5.8.3`. |
| Full engine version | The same version with the changelist and the branch, for example `5.8.3-58210709+++UE5+Release-5.8`. |
| File version UE5 | The UE5 file version of the package, for example 1018. A number. |
| File version UE4 | The UE4 file version of the package, for example 522. A number. |
| File version licensee | The licensee file version of the package. 0 for a stock engine. A number. |

All fields except **Is package** are empty when the file is not a
package that UCommander can read. The asset fields are empty when the
package has no main asset. The plugin reads only the package header, and
keeps the values of the last 16 files in a cache.

## Development

Install [Rust](https://rustup.rs/). The repository pins the toolchain
version in `rust-toolchain.toml`, so rustup installs it automatically. Also
install cargo-deny:

```bash
cargo install cargo-deny --locked
```

The test fixtures in `tests/fixtures/` use Git LFS. Install it once, before
you clone the repository, or run `git lfs pull` after the clone:

```bash
git lfs install
```

Run all checks:

```bash
cargo xtask check
```

Show the package summary, the name map, the import table and the export
table of a `.uasset` or `.umap` file:

```bash
cargo run -p uc-cli -- dump <file>
```

The output is for people, not for scripts. The format can change.

Make the plugin ZIP in `target\dist\`:

```bash
cargo xtask package
```

Replace the installed plugin with a new build:

```bash
cargo xtask install
```

`install` finds the plugin in `wincmd.ini`, so install the ZIP once first. TC
can stay open. If the plugin is under `Program Files`, Windows shows a UAC
prompt. Restart TC to load the new version.

Limits of `install`:

- A path with non-ASCII characters in an ANSI `wincmd.ini` does not match,
  for example `D:\Narzędzia`. `install` then reports that TC has not
  registered the plugin. A UTF-8 or UTF-16 `wincmd.ini` works.
- `install` does not follow `AlternateUserIni` or `RedirectSection`.

### Release

1. In a PR, set the new version in `Cargo.toml`, and move the
   `[Unreleased]` entries of `CHANGELOG.md` to a section for the version.
2. Merge the PR.
3. Tag the merge commit on `main`, and push the tag:

   ```bash
   git tag -a v0.1.0 -m "UCommander 0.1.0"
   ```

   ```bash
   git push origin v0.1.0
   ```

The workflow `release.yml` checks the tag, runs all checks, and publishes
the GitHub Release with the plugin ZIP and `SHA256SUMS.txt`. To test the
workflow without a release, start it manually on a branch. The run artifact
then has the files.

## License

Copyright BIBBIT Michal Nowak.

UCommander is licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.

## Trademarks

Unreal and Unreal Engine are trademarks or registered trademarks of Epic
Games, Inc. Total Commander is a product of Ghisler Software GmbH.
UCommander is not affiliated with or endorsed by either company.

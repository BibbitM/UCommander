# UCommander

[![CI](https://github.com/BibbitM/UCommander/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/BibbitM/UCommander/actions/workflows/ci.yml)

Total Commander plugins for Unreal Engine files.

Source code: https://github.com/BibbitM/UCommander

> **Status:** In development. There is no release yet.

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

1. Get the file `UCommanderContent-<version>.zip`. There is no release yet, so
   build it with `cargo xtask package`. It goes to `target\dist\`.
2. Open the ZIP in Total Commander with Enter.
3. Accept the installation.
4. Make a custom column with the field `UCommanderContent.Is package`.

The plugin needs no extra runtime.

## Development

Install [Rust](https://rustup.rs/). The repository pins the toolchain
version in `rust-toolchain.toml`, so rustup installs it automatically. Also
install cargo-deny:

```bash
cargo install cargo-deny --locked
```

Run all checks:

```bash
cargo xtask check
```

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

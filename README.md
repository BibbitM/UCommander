# UCommander

[![CI](https://github.com/BibbitM/UCommander/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/BibbitM/UCommander/actions/workflows/ci.yml)

Total Commander plugins for Unreal Engine files.

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

## Build

Install [Rust](https://rustup.rs/). The repository pins the toolchain
version in `rust-toolchain.toml`, so rustup installs it automatically.

```bash
cargo build --workspace
```

```bash
cargo test --workspace
```

## Manual installation (development)

There is no release and no installer yet. To try the content plugin:

1. Run `cargo build --workspace --release`.
2. Copy `target\release\uc_content.dll` to a folder for TC plugins, with the
   name `UCommanderContent.wdx64`.
3. In TC, open **Configuration → Options → Plugins → Content plugins
   (.WDX) → Configure → Add**.
4. Select `UCommanderContent.wdx64`.
5. Make a custom column with the field `UCommanderContent.Is package`.

TC keeps the DLL loaded. Close TC before you copy a new version.

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

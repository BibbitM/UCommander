# P0 – Repository setup

Status: **Approved – in progress**

Branch: `pt/p0-repo-setup` (from `main`)

## Goal

Make an empty but complete Rust workspace for UCommander. After P0, an agent
or a person can clone the repository, run one command, and get a build and a
test result. P0 adds no plugin features.

## Acceptance criteria

1. `cargo build --workspace` succeeds on Windows x64 (MSVC).
2. `cargo test --workspace` succeeds. Each crate has one test.
3. `cargo fmt --all --check` succeeds.
4. `cargo clippy --workspace --all-targets -- -D warnings` succeeds.
5. `cargo deny check` succeeds.
6. The repository contains: `LICENSE-MIT`, `LICENSE-APACHE`, `README.md`,
   `CHANGELOG.md`, `AGENTS.md`, `CLAUDE.md`, `.gitignore`, `.gitattributes`,
   `docs/FEATURES.md`, `docs/PLAN.md` and the PR template.
7. `git status` shows no build output and no IDE files after a build and
   after RustRover opens the project.
8. Each commit builds and passes the checks that exist at that commit.

## Increments (one commit each)

| # | Commit title | Content |
|---|---|---|
| 0 | Add P0 step brief | This file, after approval. |
| 1 | Add licenses | `LICENSE-MIT`, `LICENSE-APACHE`. Copyright line: `Copyright BIBBIT Michal Nowak.` |
| 2 | Add README | Name, purpose, status ("in development"), license section. No install steps yet. |
| 3 | Add git ignore and attributes rules | `.gitignore`: `target/`, `.idea/` (except shared files), `xtask.toml`, local config. `.gitattributes`: line endings, LFS rules for `tests/fixtures/**/*.uasset`, `*.umap`, `*.uexp`, `*.ubulk`, `*.pak`, `*.utoc`, `*.ucas`. |
| 4+5 | Add Cargo workspace and empty crates | One commit. A workspace without members does not build, so increments 4 and 5 are joined. Root `Cargo.toml`: workspace members, shared version `0.0.0`, edition 2024, `license = "MIT OR Apache-2.0"`, `publish = false`, workspace lints. `rust-toolchain.toml` (channel `1.99.1`, target `x86_64-pc-windows-msvc`, components `rustfmt` and `clippy`), `rustfmt.toml`. |
| (5) | (joined with 4) | `uc-format`, `uc-tc`, `uc-content`, `uc-lister`, `uc-packer` (cdylib), `uc-cli` (binary `uc`), `xtask`. One test in each crate. |
| 6 | Add cargo-deny configuration | `deny.toml`: allowed licenses, advisories, bans, sources. |
| 7 | Add agent instructions | `AGENTS.md` (rules, commands, gates, naming, EULA and data rules). `CLAUDE.md` with one line: `@AGENTS.md`. |
| 8 | Add Claude Code permissions | `.claude/settings.json`: allow cargo and `git commit`; ask for `git push`, `git tag`, `gh pr create`, `gh pr merge`. |
| 9 | Add feature scope and plan | `docs/FEATURES.md` (agreed IDs and priorities), `docs/PLAN.md` (plan points, status). |
| 10 | Add changelog and PR template | `CHANGELOG.md` ("Keep a Changelog"), `.github/pull_request_template.md`. |
| 11 | Add RustRover run configurations | `.run/` with "Format check", "Clippy", "Test workspace", "Deny", and the compound "Check all" that starts all four. |

## Out of scope

- CI workflows (P1).
- TC plugin exports and FFI tests (P2).
- `xtask` commands (P3). P0 adds only the empty crate.
- Release workflow (P4).
- Creating the GitHub repository. This needs owner approval at the first push.

## Owner actions

- None in P0. Fixtures start in point 1.

## Decisions

1. **Branch.** P0 uses the normal workflow: branch `pt/p0-repo-setup`, then a
   PR to `main`. This also tests the workflow.
2. **Rust version.** Pin stable `1.99.1` (the current stable on 2026-10-29).
3. **cargo-deny.** Installed locally with `cargo install cargo-deny --locked`.
4. **Copyright line.** `Copyright BIBBIT Michal Nowak.`

## Notes for the end of P0

- The first push needs the GitHub repository `BibbitM/UCommander`. Creating
  it is a gate: the owner approves it, and selects public or private.

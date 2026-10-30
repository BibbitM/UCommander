# AGENTS.md

Instructions for coding agents that work on UCommander. Read this file at the
start of each session. Then read `docs/PLAN.md` and the step brief of the
current plan point in `docs/steps/`.

## 1. Project

UCommander is a set of Total Commander (TC) plugins for Unreal Engine (UE)
files. It is written in Rust and is open source (`MIT OR Apache-2.0`).

| Crate | Type | Released file |
|---|---|---|
| `crates/uc-format` | Library. Parser for UE packages and containers. | — |
| `crates/uc-tc` | Library. TC plugin API types and safe FFI helpers. | — |
| `crates/uc-content` | cdylib. Content plugin (WDX). | `UCommanderContent.wdx64` |
| `crates/uc-lister` | cdylib. Lister plugin (WLX). | `UCommanderLister.wlx64` |
| `crates/uc-packer` | cdylib. Packer plugin (WCX). | `UCommanderPacker.wcx64` |
| `crates/uc-cli` | Binary `uc`. Shows package data. Development and test tool. | — |
| `crates/uc-test-host` | Library, tests only. Loads a plugin DLL and calls its exports. | — |
| `xtask` | Build automation (`cargo xtask`). | — |

Targets: Windows x64 only (`x86_64-pc-windows-msvc`). TC 11.0 or later.
UE 5.8 is required. Earlier UE 5.x versions are best effort.

## 2. Language and style

Write all text in the spirit of ASD-STE100 (Simplified Technical English).
This applies to documentation, comments, commit messages, PR descriptions and
messages to the owner.

- Use short sentences. One instruction in each sentence.
- Use the active voice and the imperative for instructions.
- Use one term for one concept. Do not use synonyms.
- Use the terms in the glossary below with this meaning only.

| Term | Meaning |
|---|---|
| Package | One `.uasset` or `.umap` file and its companion files (`.uexp`, `.ubulk`, `.uptnl`). |
| Object | One export in a package. |
| Container | A `.pak` file, or a `.utoc` + `.ucas` pair. |
| Editor package | An uncooked package from a project `Content` folder. |
| Cooked package | A package from a build. |
| Plan point | One item in `docs/PLAN.md`. It has one branch and one PR. |
| Increment | One small step in a plan point. It is one commit. |

## 3. Commands

Run these commands from the repository root.

| Purpose | Command |
|---|---|
| Build | `cargo build --workspace` |
| Test | `cargo test --workspace` |
| Format | `cargo fmt --all` |
| Format check | `cargo fmt --all --check` |
| Lint | `cargo clippy --workspace --all-targets -- -D warnings` |
| Dependency check | `cargo deny check` |

All five checks (format check, lint, build, test, dependency check) must pass
before each commit. `cargo xtask check` replaces this list in plan point P3.

## 4. Naming

Follow the Rust API Guidelines.

- Package names: kebab-case with the prefix `uc-` (`uc-format`). The only
  exception is `xtask`, which uses the usual Rust name for build automation.
- Types and traits: `UpperCamelCase`.
- Functions, methods, modules and variables: `snake_case`.
- Constants and statics: `SCREAMING_SNAKE_CASE`.
- Public product names use `UCommander`. Internal names use `uc`.
- Use the names from the UE format for format types where possible, without
  the UE prefix letter: `PackageFileSummary`, not `FPackageFileSummary`.

## 5. Architecture rules

1. `uc-format` must not depend on TC code or Windows APIs.
2. Each exported `extern "C"` function catches all panics with
   `std::panic::catch_unwind`, and returns a TC error code. A panic must never
   close TC.
3. Each `unsafe` block has a `// SAFETY:` comment.
4. The content plugin must not block. Read headers only, and use a cache.
5. The plugins are read-only. Do not write to UE files until a plan point
   adds a write feature.
6. Parsers treat all input as untrusted. A damaged file gives an error, never
   a panic and never an unbounded allocation.
7. Unknown custom versions and unknown data do not stop the parser. Skip
   them, and report a warning.
8. Plugin crates use `crate-type = ["cdylib", "rlib"]`, so that `cargo test`
   builds the DLL. Their FFI tests load it with `plugin_dll_path`.

## 6. Tests

- Test features, not internal structure. Use the public API of a crate, or
  the exported plugin functions.
- Use TDD for each feature:
  1. Write a test with a synthetic minimal input. Run it. It must fail.
  2. Write the minimum code to make it pass.
  3. Write a test with a real fixture from `tests/fixtures/`. Compare with
     its `.expected.json` file.
  4. Make it pass. Refactor.
  5. Add one robustness test: truncated or damaged input gives an error.
- Expected values come from the UE editor, not from our parser output.
  Snapshot tests of parser output are only for display text (CLI, lister).
- Do not change an existing expected value without owner approval.

## 7. Workflow

### 7.1 Plan point

1. Update `main`. Create the branch `pt/<id>-short-name`.
2. Write the step brief `docs/steps/<id>-short-name.md`: goal, acceptance
   criteria, increments, out of scope, owner actions, open questions.

`<id>` is the plan point ID from `docs/PLAN.md`, in lowercase. Phase 0 uses
`p0` to `p4`. Phase 1 and later use two digits: point 3 is `03`.
Examples: `pt/p2-content-skeleton` and `docs/steps/p2-content-skeleton.md`,
`pt/03-content-asset-class` and `docs/steps/03-content-asset-class.md`.
3. Wait for owner approval of the brief (gate).
4. Do the increments. Commit after each increment.
5. Update `CHANGELOG.md` (`## [Unreleased]`) and `README.md` if necessary.
6. Update `docs/PLAN.md`: set the status of the plan point.
7. Run all checks. Push and open a PR (gates).
8. Fix CI failures and review findings with new commits.

### 7.2 Commits

- One intent in each commit. Do not mix a refactor and a feature.
- Each commit builds and passes all checks.
- Target size: less than about 200 changed lines, excluding fixtures.
- Tests and code go in the same commit.
- Title: imperative, 50 characters or less, no period. Example:
  `Read package summary tag`.
- Body: the intent. Tell why the change is necessary and what it makes
  possible. Wrap at 72 characters.
- After a push, do not rewrite history: no amend, no rebase, no force push.
- Exception: Dependabot makes its own commits and PR titles. They do not
  follow the title rules. Do not change them.

### 7.3 Gates

Stop and wait for owner approval before each of these actions:

- The step brief of a plan point.
- `git push`.
- Creating a PR, a GitHub repository or a release.
- A version change or a git tag.
- A new dependency. Give the reason, the license and the alternatives.
- A change to `AGENTS.md`, `.github/`, `xtask/` or `.claude/settings.json`.
- A change to an existing expected test value.

## 8. Legal and data rules

- Do not copy Unreal Engine source code into this repository. You can read
  the engine source to understand a format. Write the facts in your own words
  in `docs/formats/`, and write new code from those notes.
- If you adapt code from another open-source project, keep its license
  notice, and record the source in the file and in `docs/formats/`.
- Do not add Oodle files (DLLs, headers, libraries) to the repository.
- Do not add AES keys, company assets, company paths or company names.
  Local configuration goes in files that `.gitignore` excludes.
- Make fixtures only from assets that the owner made. Do not add Epic sample
  content or Starter Content.

## 9. Format knowledge

- Record each format fact in `docs/formats/*.md` with its source: a
  repository, file and version, or "observed in fixture X".
- If you are not sure about a format detail, ask the owner. Do not guess.
  UE 5.8 version numbers and custom engine changes are typical questions.

## 10. Fixtures

- You cannot make UE assets. When a plan point needs a fixture, write the
  request in the step brief: asset type, settings and expected values.
- The owner makes the asset in the fixtures project and exports it.

## 11. Environment notes

- The owner uses RustRover and GitHub Desktop.
- GitHub Desktop can stash uncommitted changes when the branch changes.
  If files are missing, run `git stash list` before you write them again.
- Shared RustRover run configurations are in `.run/`.

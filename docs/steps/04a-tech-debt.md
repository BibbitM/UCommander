# 04a – Technical debt after point 4

Branch: `pt/04a-tech-debt` (from `main`)

Features: none. Release after this point: none.

## Goal

Fix the gaps that point 4 and its review found, before point 5 starts.
The tests stop filling the system temporary folder. The FFI tests get
their expected values from the fixture files. The format notes and the
TC notes get two facts that are not checked now. `AGENTS.md` gets the
workflow rules and the lessons of point 4.

## Acceptance criteria

1. No test of `uc-content` or `uc-cli` writes to the system temporary
   folder (`%TEMP%`). The tests write to the Cargo temporary folder of
   the target folder (`CARGO_TARGET_TMPDIR`, `target/tmp`). The file
   names do not contain a process ID, so the number of files does not
   grow with each test run. `cargo clean` removes them.
2. The FFI tests of `uc-content` read the expected values of the
   fixtures from the `.expected.json` files, as the `uc-format` fixture
   tests do. No expected value is in the test code. No expected value
   changes.
3. `docs/formats/package.md` records which builds of UE 5.8.3 write the
   package name of the summary, with the source.
4. `docs/tc-api/content.md` records the order of the `ContentGetValueW`
   calls of TC when the user sorts a folder by a plugin column. The
   source is an experiment in TC (owner action 3).
5. `AGENTS.md` has the workflow rules and the lessons of "Design" below.
6. CI accepts a plan point ID with a letter after the number (`4a`):
   the branch `pt/04a-tech-debt` and the `docs/PLAN.md` row `4a`.
7. All checks pass.

## Design

### Test files in the target folder

Cargo gives the integration tests of a package the environment variable
`CARGO_TARGET_TMPDIR` at compile time: a folder in the target folder.
Source: The Cargo Book, "Environment variables Cargo sets for crates".
`env!("CARGO_TARGET_TMPDIR")` gives it to the test code.

| Test code | Now | New |
|---|---|---|
| `uc-content/tests/common/mod.rs`, `test_file` | `%TEMP%/uc-content-<folder>-<pid>/<name>` | `<tmp>/uc-content/<folder>/<name>` |
| `uc-content/tests/value.rs`, `test_file` | `%TEMP%/uc-content-tests-<pid>/<name>` | the shared `test_file` |
| `uc-content/tests/*.rs`, missing file | `%TEMP%/uc-content-…-no-such-file.uasset` | `<tmp>/uc-content/no-such-file.uasset`. The test checks that it does not exist. |
| `uc-cli/tests/dump.rs`, `temp_file` | `%TEMP%/uc-dump-<pid>-<name>` | `<tmp>/uc-dump/<name>` |

Each test uses a file name that no other test of the same test binary
uses. Two runs of the same test binary at the same time can write the
same file. This is not a typical case, and the file content is the same
for each run.

The `xtask` tests already remove their folders. The production code of
`xtask install` uses `%TEMP%` for the error file of the elevated
process. It removes the file. These do not change.

Observed now: `%TEMP%` has 1587 `uc-dump-*` files (2.2 GB) from earlier
test runs. The `uc-content-*` folders were removed after point 4.

### Expected values from the fixture files

`uc-content` gets `serde` (with `derive`) and `serde_json` as
dev-dependencies, with the same versions as `uc-format`. The tests read
these keys: `asset_class`, `asset_class_path`, `asset_name`,
`object_path`, `package_path`, `summary.saved_by_engine_version`,
`summary.file_version_ue5`, `summary.file_version_ue4`,
`summary.file_version_licensee`.

The short engine version (`5.8.3`) is not a key. The test takes the
text before the first `-` of `saved_by_engine_version`. This follows
the UE text form (`docs/formats/package.md`, "Engine version").

A shared function in `tests/common/mod.rs` loads the expected values of
a fixture. The tests of the other cases (damaged, not supported, small
`maxlen`) do not change.

### Package name of the summary

UE 5.8.3 sets the package name of the summary only in a build with
editor-only data: the editor and the cook commandlet. The value is the
name of the package in memory. Other builds keep the default value, an
empty text. Source: UE583
`CoreUObject/Private/UObject/LinkerSave.cpp`, the function that prepares
the saver. The comment at the read function says that UE does not use
the value now (UE583 `PackageFileSummary.cpp`).

So the point 4 rule (a name that does not start with `/` gives no
package path) is correct for an empty name. UE 5.8.3 does not write
`None`. No new fixture is necessary.

### TC call order (experiment)

1. I make a local build of the content plugin that writes one line for
   each `ContentGetValueW` call to a log file: the time, the thread, the
   file name and the field index. This build is not committed.
2. The owner installs it, opens a folder with more than 16 packages, and
   adds two plugin columns. Then the owner sorts by one plugin column.
3. I read the log, record the order in `docs/tc-api/content.md`, and
   install the normal build again (`cargo xtask install`).

If TC asks for one field of all files before the next field, 16 cache
entries are not enough for a folder with more than 16 packages. Then I
stop and tell the owner, with the numbers from the log.

### `AGENTS.md` (gate)

The proposed text. The owner approves the exact text in the increment.

§7.1, new text for steps 6 to 8:

> 6. Ask a second agent to review the whole branch. Fix the findings
>    with new commits.
> 7. As the last commit, set the status of the plan point to `Done` in
>    `docs/PLAN.md`. …
> 8. Run all checks. Push and open a PR (gates).
> 9. If changes are necessary after the `Done` commit, revert it with
>    `git revert` (the status goes back to `Planned`). Fix with new
>    commits. Then set `Done` again in a new last commit.

§7.2, new rule:

> - Do not rewrite commits, also before the first push: no amend of an
>   old commit, no fold, no rebase. Fix an error with a new commit. If an
>   old commit fails a check, tell the owner which commit fails and
>   which commit fixes it, and write it in the PR.

§12, new lessons:

> - A pipe hides the exit code of a check: `cargo xtask check | tail`
>   gives the exit code of `tail`. Use `set -o pipefail`, or do not use a
>   pipe before `&&`.
> - The FFI test helper loads and frees the DLL for each call. When no
>   other reference exists, Windows unloads the DLL, and its static data
>   (for example the cache) is lost. A test of plugin state holds its own
>   `Plugin` for its duration.
> - The tests of one test file run in parallel in one process. A test
>   that clears or fills global plugin state goes into its own test
>   file.

### `xtask plan-status` (gate)

Now the plan point ID of a branch and of a `PLAN.md` row must be digits
(or `P` and digits). The change accepts digits and one lower-case letter
(`4a`). The branch `pt/04a-x` gives `4a`. Leading zeros go as before:
`pt/04a-x` and `pt/4a-x` give `4a`. Tests in `xtask` for the branch, the
row and a status check.

## Fixture request

None.

## Increments (one commit each)

| # | Commit title | Content |
|---|---|---|
| 0 | Add point 4a step brief | This file, after approval. `docs/PLAN.md`: row `4a`, status `Planned`. |
| 1 | Accept plan point IDs with a letter | `xtask/src/plan.rs` and its tests (gate). |
| 2 | Write content test files to the target folder | `uc-content` tests: `CARGO_TARGET_TMPDIR`, shared `test_file`, missing-file path. |
| 3 | Write dump test files to the target folder | `uc-cli/tests/dump.rs`. The snapshots do not change. |
| 4 | Read FFI expected values from fixture files | `uc-content` dev-dependencies (gate), shared loader, text, numeric and asset class tests. |
| 5 | Add package name facts to the format notes | `docs/formats/package.md`. |
| 6 | Record the TC call order for a sort | `docs/tc-api/content.md`, after the experiment. |
| 7 | Add point 4 rules and lessons to AGENTS.md | §7.1, §7.2, §12 (gate). |
| 8 | Set point 4a status to done | `docs/PLAN.md`. The last commit, after the review (rule of increment 7). |

`CHANGELOG.md` and `README.md` do not change: the changes are for
development only.

## New dependencies (gate)

| Crate | Kind | Reason | License | Alternatives |
|---|---|---|---|---|
| `serde` 1.0.229 (`derive`) | dev-dependency of `uc-content` | Read the `.expected.json` files in the FFI tests. Already a dev-dependency of `uc-format`, so no new crate in `Cargo.lock`. | MIT OR Apache-2.0 | Keep the values in the test code (now). A small JSON reader in the test code. A `build.rs` that writes the values as Rust code. |
| `serde_json` 1.0.154 | dev-dependency of `uc-content` | Same. | MIT OR Apache-2.0 | Same. |

## Owner actions

1. Approve this brief, the new dev-dependencies, and the gates of
   increments 1 and 7. Done. The owner approves the exact `AGENTS.md`
   text at the review of the branch.
2. Answer the open questions. Done.
3. The TC experiment (increment 6): install the log build, open a
   folder with more than 16 packages, add two plugin columns, and sort by
   one of them. Then tell me.

## Out of scope

- A change of the cache size. If the experiment shows that it is
  necessary, I stop and ask.
- Cleanup of the old files in `%TEMP%`. It is not part of the
  repository. See open question 3.
- Tests of `xtask` and the production code of `xtask install`.

## Owner decisions

1. After a review, a `Done` commit is reverted with `git revert`, the
   fixes come in new commits, and a new commit sets `Done` again. No
   `WIP` status. A review before the `Done` commit is the normal
   sequence. (Owner answer after point 4, with the proposal that the
   owner accepted when point 4a started.)
2. The owner approved this brief, the new dev-dependencies, the gates
   of increments 1 and 7, and the proposal of each open question below.

## Open questions (answered)

The answers are in "Owner decisions".

1. **Plan point ID.** CI accepts only digits as an ID now, so `4a` needs
   the `xtask` change (gate). I propose this change. Alternatives:
   renumber the points 5 to 16 (many references change: D15 in point 6,
   the releases), or use a branch without the prefix `pt/`, which CI does
   not check.
2. **New dev-dependencies.** I propose `serde` and `serde_json` for the
   `uc-content` tests (see the table above).
3. **Old files in `%TEMP%`.** Remove the 1587 `uc-dump-*` files
   (2.2 GB) now, as the `uc-content-*` folders after point 4? I propose
   yes. It is not part of this branch.
4. **Log build for the experiment.** I propose that the log goes to
   `%TEMP%\uc-content-calls.log`, and that I remove it after the
   experiment. The build is not committed.

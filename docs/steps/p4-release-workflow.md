# P4 – Release workflow

Branch: `pt/p4-release-workflow` (from `main`)

## Goal

Make a GitHub Release from a version tag, with no manual build. After P4, the
owner pushes the tag `vX.Y.Z` on `main`, and GitHub builds, tests and packages
the plugins, and publishes the release with the plugin ZIP. P4 also sets the
version to 0.1.0, so that the first release is `v0.1.0`.

## Acceptance criteria

1. A push of a tag `v*` starts the workflow `release.yml`.
2. The workflow fails, and makes no release, when one of these is true:
   - The tag is not `v` + the workspace version (`xtask::VERSION`).
   - The tagged commit is not on `main`.
   - `CHANGELOG.md` has no section for the version, or the section is empty.
   - A check fails: format check, lint, test or dependency check.
3. The release has the title `UCommander X.Y.Z`, the text of the
   `CHANGELOG.md` section of the version, and the asset
   `UCommanderContent-X.Y.Z.zip`.
4. The release also has the asset `SHA256SUMS.txt`, with the SHA-256 hash
   of each ZIP.
5. The ZIP comes from `cargo xtask package`, the same task as in CI. The
   package task already refuses a DLL that needs the VC runtime.
6. Security (from the P1 review): `contents: write` only at the job level of
   the release job. No build cache. The concurrency group does not start
   with `ci-`, and it does not cancel a running release. No `RUSTFLAGS`.
   `persist-credentials: false`.
7. A manual run (`workflow_dispatch`) does all steps except the publish
   step. It uploads the ZIP and the release text as a run artifact. With
   it, I test the workflow on the branch before the merge.
8. The workspace version is 0.1.0. `CHANGELOG.md` has the section
   `## [0.1.0] - <date>` and the comparison links.
9. `README.md` does not say "There is no release yet". It links the
   releases page.

## Design

### New xtask tasks

| Task | Steps |
|---|---|
| `tag-version <tag>` | Fail if `<tag>` is not `v` + `VERSION`. Example: `v0.1.0` passes for 0.1.0, `v0.1.1` and `0.1.0` fail. |
| `release-notes <version> <file>` | Find `## [<version>]` in `CHANGELOG.md`. Write the lines until the next `## ` heading to `<file>`. Fail if there is no section, or if it is empty. |

The tests use text, not the real `CHANGELOG.md`, so a later changelog cannot
break them.

### Workflow

One job `release` on `windows-latest`. The name is not `check` or `deny`,
because the required checks match by job name.

1. Check out with LFS and the full history (`fetch-depth: 0`), for the
   `main` check.
2. Install the pinned toolchain (`rustup toolchain install`).
3. `cargo xtask tag-version "$env:TAG"`. Only for a tag push.
4. Check that the tag commit is on `main`:
   `git merge-base --is-ancestor $env:GITHUB_SHA origin/main`. Only for a
   tag push.
5. `cargo xtask release-notes`, to `target/dist/release-notes.md`.
6. Format check, lint, test.
7. Install cargo-deny with `taiki-e/install-action` (already in CI) and run
   `cargo deny check`.
8. `cargo xtask package`.
9. Write `SHA256SUMS.txt` with `Get-FileHash`.
10. Manual run: upload `target/dist/` as a run artifact.
11. Tag push: `gh release create` with the ZIP, `SHA256SUMS.txt` and the
    notes file. `gh` is on the GitHub runners, and it uses `GITHUB_TOKEN`.
    **No new action.**

The tag name comes from `github.ref_name` through the variable `TAG`, so
that a tag name cannot inject commands. This is the same method as `BRANCH`
in `ci.yml`.

`plan-status` does not run, because a tag has no plan point branch (P3
review).

## Increments (one commit each)

| # | Commit title | Content |
|---|---|---|
| 0 | Add P4 step brief | This file, after approval. |
| 1 | Add xtask tag-version | Task and tests: the correct tag passes, a different version fails, a tag without `v` fails. |
| 2 | Add xtask release-notes | Task and tests: the section text is found, the next section is not included, a missing section fails, an empty section fails. |
| 3 | Add release workflow | `.github/workflows/release.yml` as in "Design". |
| 4 | Set version to 0.1.0 | `Cargo.toml`, `Cargo.lock`. `CHANGELOG.md`: `[Unreleased]` becomes `[0.1.0] - <date>`, with comparison links. |
| 5 | Update docs for P4 | README: releases link, installation from the release, release steps for the owner. |
| 6 | Set P4 status to done | `docs/PLAN.md`. The last commit. |

Each increment follows the TDD sequence in `AGENTS.md` §6. Increment 3 has
no automatic test. The manual run (criterion 7) tests it.

## Verification

1. Before the PR: run `release.yml` on a branch. All steps pass. The run
   artifact has the ZIP, `SHA256SUMS.txt` and the release text of 0.1.0.

   GitHub starts a manual run (`workflow_dispatch`) only for a workflow
   that is on `main`, so the manual run was not possible before the merge.
   The owner chose a temporary branch, as in P1:
   - Commit `Run tag steps only for a tag ref`: the tag steps check
     `startsWith(github.ref, 'refs/tags/')`, not the event. A branch run
     then never publishes.
   - The branch `ci-test/p4-release` had one extra commit with a `push`
     trigger for that branch. After the run, I deleted the branch. The
     commit never went to `main`.

   **Result** (run 41923063896): all steps passed. The tag steps and the
   publish step were skipped. The artifact had
   `UCommanderContent-0.1.0.zip` with the 8 files and `version=0.1.0` in
   `pluginst.inf`, `SHA256SUMS.txt` with LF line endings
   (`sha256sum -c` passed), and the release text of 0.1.0.
2. After the merge: the owner tags the merge commit `v0.1.0` and pushes the
   tag. The release shows on GitHub with the ZIP.
3. The owner installs the ZIP from the release in TC (as in the P3 manual
   test).

## Owner actions (gates)

1. Approve this brief.
2. Approve the version change to 0.1.0 (increment 4).
3. Approve the new workflow file in `.github/` (increment 3).
4. Approve the push of the branch, the manual run and the PR.
5. After the merge: tag and push `v0.1.0`. I can do it with your approval:
   `git tag -a v0.1.0 -m "UCommander 0.1.0"` on the merge commit, then
   `git push origin v0.1.0`.

## Out of scope

- ZIPs for the lister and the packer (points 7 and 11). The workflow takes
  each ZIP in `target/dist/`, so they come without a workflow change.
- Code signing of the DLLs.
- Automatic version change or changelog generation.
- Publishing crates to crates.io (`publish = false`).

## Decisions to confirm

1. **Publish or draft.** I propose that the workflow publishes the release
   directly. All checks run before the publish step. The alternative is a
   draft that you publish by hand on GitHub.
2. **Tag protection.** I propose a ruleset for tags `v*`: no delete, no
   update (move). Then a published version always points to the same commit.
   I apply it with `gh api`, after your approval.
3. **Checksums.** `SHA256SUMS.txt` (criterion 4) is optional. I propose it,
   because a colleague can check a ZIP that they got from another place.
4. **Release date.** The `CHANGELOG.md` date is the date of increment 4. If
   the merge is on a later day, I change the date in a new commit before the
   tag.

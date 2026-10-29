# P1 – CI

Status: **In review**

Branch: `pt/p1-ci` (from `main`)

## Goal

Run all checks on GitHub for each PR and for each push to `main`. After P1, a
PR cannot be merged into `main` until all checks pass.

## Acceptance criteria

1. A PR to `main` starts the CI workflow. A push to `main` also starts it.
2. CI runs the same checks as `AGENTS.md` §3: format check, lint, build,
   test and dependency check. It also builds the release profile.
3. A failure in any check makes the workflow fail (verified, see
   "Verification").
4. CI checks out LFS files (`lfs: true`).
5. The workflow uses the pinned toolchain from `rust-toolchain.toml`.
6. The workflow has read-only permissions (`contents: read`).
7. A new push to the same PR cancels the older run.
8. Branch protection on `main`: PR required, the CI checks required, no
   force push, no branch delete. The rules also apply to the owner.
9. The README shows the CI status badge.

## Increments (one commit each)

| # | Commit title | Content |
|---|---|---|
| 0a | Set P0 status to done | `docs/PLAN.md` and the P0 brief: P0 is merged. |
| 0b | Add P1 step brief | This file, after approval. |
| 1 | Add CI workflow | `.github/workflows/ci.yml`. Job `check` on `windows-latest`: checkout with LFS, format check, lint, test. Triggers: `pull_request`, push to `main`, `workflow_dispatch`. Concurrency group, `contents: read`, timeout. |
| 2 | Build release profile in CI | Add `cargo build --workspace --release` to the `check` job. The plugins ship in the release profile. |
| 3 | Add dependency check job | Job `deny` on `ubuntu-latest` with `EmbarkStudios/cargo-deny-action`. The action runs only on Linux. The result is the same, because `deny.toml` sets the Windows target. |
| 4 | Cache Rust build output in CI | `Swatinem/rust-cache` in the `check` job, so later runs are faster. |
| 5 | Add Dependabot for GitHub Actions | `.github/dependabot.yml`: weekly update PRs for the pinned actions. |
| 6 | Add CI badge to README | Status badge for `ci.yml`. |
| 7 | Update changelog and plan for P1 | `CHANGELOG.md` and `docs/PLAN.md`. |

## Verification

- **Pass:** the PR for P1 shows both jobs green.
- **Fail:** push a temporary branch `ci-test/p1-fail` with one commit that
  breaks the format check. Run the workflow on it with `workflow_dispatch`.
  It must fail. Then delete the branch. This commit never goes to `main`.
  The push needs owner approval (gate).

## Owner actions (gates)

1. Approve the push of the temporary failure branch.
2. Approve the branch protection settings on `main`. I apply them with
   `gh api` after the first CI run, because GitHub needs the check names
   (`check`, `deny`) first. I do not merge the P1 PR before that.

## Out of scope

- Packaging and plugin ZIP artifacts (P3).
- Release workflow (P4).
- Fuzzing (later).

## Decisions to confirm

1. **Action pins.** Pin each third-party action to a full commit SHA, with
   the version in a comment. A tag can move, but a commit SHA cannot.
   Dependabot (increment 5) keeps the pins up to date.
2. **New actions.** These are dependencies, so they need approval:
   `actions/checkout`, `Swatinem/rust-cache` (MIT/Apache-2.0),
   `EmbarkStudios/cargo-deny-action` (MIT/Apache-2.0).
3. **Rules for the owner.** Branch protection also applies to you, as the
   admin. The rules require no approving review, because you cannot approve
   your own PR. So you can still merge your own PRs when CI passes.

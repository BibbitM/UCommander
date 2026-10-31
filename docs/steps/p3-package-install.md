# P3 – Package and install

Branch: `pt/p3-package-install` (from `main`)

## Goal

Make one command for each repeated task: run all checks, make the plugin ZIP,
and install the new plugin on the developer's computer. After P3, the owner
gets a new plugin in TC with `cargo xtask install`, and a colleague installs it
from a ZIP file. Colleagues do not need Visual Studio or the VC++ runtime.

## Acceptance criteria

1. `cargo xtask check` runs format check, lint, test and dependency check, in
   this order. It stops at the first failure, with a non-zero exit code.
2. `cargo xtask package` makes `target/dist/UCommanderContent-<version>.zip`.
3. The ZIP contains exactly these files: `UCommanderContent.wdx64`,
   `pluginst.inf`, `README.md`, `CHANGELOG.md`, `LICENSE-MIT`,
   `LICENSE-APACHE`. No `.pdb`, no `.dll`.
4. `pluginst.inf` has the version from the workspace `Cargo.toml`.
5. `uc_content.dll` does not import `VCRUNTIME140.dll` (static C runtime).
6. `cargo xtask install` replaces the plugin file that TC has registered, also
   when TC has it loaded. A test proves this with a loaded DLL.
7. `install` needs no configuration, and it works for each TC installation
   folder. It tells the user to restart TC. If TC has not registered the plugin,
   it stops with an instruction.
8. CI uploads the ZIP as a run artifact for each PR.
9. Manual test: TC installs the ZIP when the owner opens it with Enter.
10. `README.md` works outside the repository: it has the source link at the
    top, and its links are absolute GitHub URLs.
11. `docs/PLAN.md` is the only place with the status of a plan point. Its
    values are `Planned` and `Done`.
12. A PR from a `pt/<id>-…` branch fails CI until the status of that plan point
    is `Done` in `docs/PLAN.md`.

## Installer format

Sources: the HOWTO in the TC forum, and `HISTORY.TXT` of TC 11 (`version=`
from 2013). The TC guides do not describe this file.

```ini
[plugininstall]
description=UCommander content fields for Unreal Engine packages
type=wdx
file=UCommanderContent.wdx64
defaultdir=UCommanderContent
version=0.0.0
```

**Not confirmed:** no source that I could read gives the `file=` value for a
64-bit-only plugin. `file=UCommanderContent.wdx64` is my reading. The manual
test (criterion 9) proves it. If TC refuses the file, try
`file=UCommanderContent.wdx`.

Each `pluginst.inf` has one `type` and one `file`, so one ZIP installs one
plugin. Each plugin gets its own ZIP. P3 packages only the content plugin. The
lister and the packer have no exports yet, so their ZIPs come with points 7
and 11.

## Design

`xtask` has a library and a thin `main.rs`, so that tests can call each task.

| Task | Steps |
|---|---|
| `check` | Run the four `cargo` commands. Stop at the first failure. |
| `package` | Build release. Select `uc_content.dll` by exact name. Copy it to a staging folder as `UCommanderContent.wdx64`. Write `pluginst.inf` from `packaging/content/pluginst.inf` with the version. Add the README, the CHANGELOG and the licenses. Make the ZIP. |
| `plan-status` | Get the plan point ID from the branch name. Find its row in `docs/PLAN.md`. Fail if the status is not `Done`. A branch that does not start with `pt/` passes, for example a Dependabot branch. |
| `install` | Build release. Rename the installed file to `*.old`. Windows allows a rename of a loaded DLL, but not an overwrite. Copy the new file. Delete old `*.old` files, and skip those that are locked. Tell the user to restart TC. |

- The ZIP comes from `tar.exe -a`, which is in Windows 10 and later. **No new
  crates.io dependency.**
- The version comes from `CARGO_PKG_VERSION` of `xtask`, which uses the
  workspace version.
- The install path comes from TC, on each computer:
  1. Find `wincmd.ini` from the registry value `IniFileName`, in
     `HKCU\Software\Ghisler\Total Commander`, then in `HKLM`.
  2. Find the line in `[ContentPlugins]` that ends with
     `UCommanderContent.wdx64`. Its path is the file to replace.
  3. Expand `%COMMANDER_PATH%` with the registry value `InstallDir`.
- `reg.exe` reads the registry, so there is no new dependency. `install` never
  changes `wincmd.ini`, because TC writes that file when it closes.
- The first installation uses the ZIP, so that TC registers the plugin in
  `plugins\wdx\UCommanderContent\`. If TC has no such line, `install` stops and
  tells the user to install the ZIP.
- A plugin under `Program Files` needs administrator rights. When a write
  gives "access denied", `install` starts `xtask replace-plugin` with
  `Start-Process -Verb RunAs`, which shows one UAC prompt. Only this small step
  has administrator rights.
- A static C runtime comes from `.cargo/config.toml`:
  `target-feature=+crt-static` for `x86_64-pc-windows-msvc`. The 64-bit page of
  the guide also recommends a static runtime (`/MT`).

### Status at the merge

The status documents went stale because the status changed in two steps: a
branch wrote `In review`, and nothing wrote `Done` after the merge. P3 removes
the second step:

- The last commit of a plan point sets `Done` in `docs/PLAN.md`. The merge
  brings it to `main`, so `main` is correct at the moment of the merge.
- `In progress` and `In review` go away. GitHub already shows these states.
- The briefs lose their `Status:` line, so that only one place has the status.
- `plan-status` runs in CI on each PR, so a forgotten status blocks the merge.

A bot commit after the merge is not necessary. It would also need a token with
write access, and an exception to the branch protection on `main`.

## Increments (one commit each)

| # | Commit title | Content |
|---|---|---|
| 0 | Add P3 step brief | This file, after approval. |
| 1 | Add cargo xtask alias | `.cargo/config.toml`: `[alias] xtask = "run --package xtask --"`. Split `xtask` into library and `main.rs`. |
| 2 | Add xtask check | Task and test: an unknown task fails with usage. `AGENTS.md` §3: `cargo xtask check` replaces the list. |
| 3 | Link the C runtime statically | `.cargo/config.toml`. Test: the release DLL does not contain the text `VCRUNTIME140.dll`. |
| 4 | Add pluginst.inf template | `packaging/content/pluginst.inf`. Test: the version is filled in, and the file has CRLF line endings. |
| 5 | Make README work outside the repository | Source link at the top. Absolute GitHub URLs for all links. |
| 6 | Add xtask package | Task. Tests: the ZIP has exactly the 6 files, and the plugin file is the DLL (starts with `MZ`, not a `.pdb`). |
| 7 | Add xtask install | Task. Remove `xtask.toml` from `.gitignore`. Tests with a temporary `wincmd.ini`: the path is found, `%COMMANDER_PATH%` is expanded, a missing line gives the instruction. Tests with files: a loaded DLL is replaced, an old `*.old` file is removed. |
| 8 | Upload plugin ZIP in CI | `ci.yml`: `cargo xtask package` replaces "Build release". `actions/upload-artifact`, pinned by SHA, 14 days. |
| 9 | Use one status for each plan point | `docs/PLAN.md`: values `Planned` and `Done`. Remove `Status:` from the briefs. `AGENTS.md` §7.1 and the PR template: the last commit sets `Done`. |
| 10 | Check plan status in CI | `xtask plan-status`. Tests: `Done` passes, `Planned` fails, a `pt/03-…` branch finds row 3, a `pt/p3-…` branch finds row P3, other branches pass. `ci.yml`: step for `pull_request` events, with `GITHUB_HEAD_REF`. |
| 11 | Update docs for P3 | README (installation, `cargo xtask`), CHANGELOG, `docs/PLAN.md` (`Done`), RustRover run configurations. |

Each increment follows the TDD sequence in `AGENTS.md` §6.

## Manual test

1. Run `cargo xtask package`.
2. In TC, remove the current plugin from the content plugin list.
3. Open `target\dist\UCommanderContent-0.0.0.zip` with Enter.
4. Accept the installation.
5. Make sure that the column `UCommanderContent.Is package` works.
6. Run `cargo xtask install` while TC is open.
7. Accept the UAC prompt, if it shows.
8. Restart TC. Make sure that the column still works.

Optional: repeat steps 3–5 on a computer without Visual Studio.

## Owner actions

1. Do the manual test.

## Out of scope

- Release workflow and GitHub Releases (P4).
- Version change (P4 sets 0.1.0).
- ZIPs for the lister and the packer (points 7 and 11).
- Automatic update for colleagues.

## Decisions

Confirmed by the owner:

- **ZIP tool.** `tar.exe`, not the `zip` crate, so that there is no new
  crates.io dependency.
- **License files in the ZIP.** Both licenses go in the ZIP, because the MIT
  and Apache licenses require a copy with each distribution.
- **Status model.** `Planned` and `Done` only, set in the last commit of the
  branch, and checked by CI. No bot commit after the merge.
- **New action.** `actions/upload-artifact` (MIT) for CI.
- **Administrator rights.** When the plugin folder is under `Program Files`,
  `install` shows one UAC prompt.

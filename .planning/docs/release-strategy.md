# GitHub Release Strategy

## Why This Document Exists

VoxWeave needs a reliable, repeatable, automated way to distribute Windows builds. Manual builds are prone to environment-specific differences, and a portable edition needs explicit packaging so its settings, downloaded models, and logs remain self-contained.

This document describes the GitHub Actions strategy for installer and portable releases using Tauri's official release action.

## Context

VoxWeave is a Tauri-based Windows application. The first-party [`tauri-apps/tauri-action`](https://github.com/tauri-apps/tauri-action) builds the application and creates a GitHub Release. VoxWeave's workflow additionally:

- validates the tag against all three project version files;
- creates draft releases only;
- treats hyphenated versions as prereleases and builds NSIS only;
- creates and validates a portable ZIP after the Tauri build.

## Implementation Plan

### Step 1: Automated Workflow Configuration

`.github/workflows/release.yml` is configured to:

- trigger only for version tags (`v*`);
- run on `windows-latest` with Node LTS, Rust stable, and pnpm 9;
- install dependencies with `pnpm install --frozen-lockfile`;
- use `tauri-action` to build a draft release;
- package `voxweave.exe`, `voxweave.portable`, and `README_portable.txt` in a portable ZIP;
- inspect the ZIP entries before upload, preventing a nested build directory or missing file from reaching the draft.

### Step 2: Version Synchronization

Before every release, synchronize the version exactly across:

1. `package.json`
2. `src-tauri/tauri.conf.json`
3. `src-tauri/Cargo.toml`

The pushed tag without its leading `v` must equal those values. The workflow fails before it builds if any value differs.

### Step 3: Release Triggering

Pushing a tag triggers a release build independently of ordinary commits:

```powershell
git tag v1.0.0
git push origin v1.0.0
```

### Step 4: Verification and Publishing

The workflow creates a **Draft Release**, never a public release. Before publishing:

1. Verify installer and portable artifacts.
2. Copy the matching version section from `CHANGELOG.md` into the GitHub release body.
3. Download and test the portable ZIP from a writable location.
4. Publish manually only after those checks pass.

## Decisions

| Feature | Decision |
|---|---|
| CI provider | **GitHub Actions** |
| Release tool | **tauri-action** |
| Trigger | **Git tags** (`v*`) |
| Release state | **Draft by default** |
| Stable bundles | Configured Tauri bundle set |
| Prerelease bundles | **NSIS only** plus portable ZIP |
| Portable artifact | `voxweave.exe` + `voxweave.portable` + `README_portable.txt` |

## GitHub Token

The workflow uses `${{ secrets.GITHUB_TOKEN }}`, which GitHub creates automatically for each run. It needs no repository secret configuration.

The following permission lets the token create and update draft releases:

```yaml
permissions:
  contents: write
```

Only introduce a personal access token if a later workflow genuinely needs permissions outside this repository.

## File Changes Summary

| File | Action | Purpose |
|---|---|---|
| `.github/workflows/release.yml` | Create | Version validation, draft release, and portable archive pipeline |
| `.github/release/README_portable.txt` | Create | Instructions bundled with the portable ZIP |
| `.planning/docs/release-strategy.md` | Create | This release process |
| `CHANGELOG.md` | Create/update | Human-authored GitHub release notes source |

## Verification

1. Push a disposable test tag such as `v0.0.0-test.1` and verify the workflow starts.
2. Confirm the version-validation and build steps complete in GitHub Actions.
3. Confirm the resulting draft prerelease contains an NSIS installer and `VoxWeave-0.0.0-test.1-portable-windows-x64.zip`; an MSI is not expected.
4. Extract the ZIP to a writable directory and verify `data/config.json` and `data/logs` are created next to the executable.
5. Move the complete portable directory, relaunch it, and verify a selected managed model still resolves from `data/models`.
6. Delete the disposable draft release, tag, and temporary branch.

## Release Procedures

Run commands from the repository root:

```powershell
cd D:\develop\projects\VoxWeave
```

The workflow is tag-driven and checks all three project version files. A tag containing a hyphen, such as `v1.0.0-alpha.1`, becomes a GitHub prerelease.

Prereleases build the NSIS installer and portable ZIP only: MSI versions require numeric-only components and cannot represent semantic prerelease labels. Stable tags keep the configured full bundle behavior.

Every release uploads:

```text
VoxWeave-<version>-portable-windows-x64.zip
```

Its ZIP root contains exactly:

1. `voxweave.exe`
2. `voxweave.portable`
3. `README_portable.txt`

The marker enables portable mode. When `voxweave.exe` sees `voxweave.portable` beside it, settings, managed Whisper models, and file logs use a sibling `data` folder. The folder contains API keys and potentially sensitive diagnostic logs; it must be kept private and writable. Models are not bundled but travel with the folder after download.

---

### Test Release: `v0.0.0-test.1`

Use this to verify the real release path without publishing anything.

1. Create a temporary branch:

   ```powershell
   git status --short
   git switch -c codex/test-release-v0.0.0-test.1
   ```

2. Change the version to `0.0.0-test.1` in:

   - `package.json`
   - `src-tauri/tauri.conf.json`
   - `src-tauri/Cargo.toml`

3. Update `CHANGELOG.md` with a `0.0.0-test.1` section and retain an empty `Unreleased` section above it.

4. Verify the prerelease bundle locally. `test.1` is not MSI-compatible, so build NSIS only:

   ```powershell
   pnpm install --frozen-lockfile
   pnpm tauri build --bundles nsis
   cd src-tauri
   cargo test
   cd ..
   ```

5. Commit and push the temporary branch:

   ```powershell
   git add package.json src-tauri/tauri.conf.json src-tauri/Cargo.toml CHANGELOG.md
   git commit -m "Prepare test release v0.0.0-test.1"
   git push -u origin codex/test-release-v0.0.0-test.1
   ```

6. Create and push the test tag:

   ```powershell
   git tag v0.0.0-test.1
   git push origin v0.0.0-test.1
   ```

7. In GitHub **Actions**, wait for the `Publish release` workflow for `v0.0.0-test.1` to complete.

8. In GitHub **Releases**, inspect the draft `VoxWeave v0.0.0-test.1`. It must be a prerelease. Verify the NSIS installer and `VoxWeave-0.0.0-test.1-portable-windows-x64.zip`; do not expect an MSI and do not publish it.

9. Download and extract the portable ZIP to a writable folder. Run `voxweave.exe`, change a setting, restart, and confirm `data/config.json` and `data/logs` remain beside the executable. If practical, download a small model, move the entire folder, relaunch, and confirm the model remains selected.

10. Delete the draft release in GitHub.

11. Remove the test tag and branch locally and remotely:

   ```powershell
   git push origin --delete v0.0.0-test.1
   git tag -d v0.0.0-test.1
   git switch main
   git branch -D codex/test-release-v0.0.0-test.1
   git push origin --delete codex/test-release-v0.0.0-test.1
   ```

---

### First Alpha Release: `v1.0.0-alpha.1`

Use exactly:

```text
version: 1.0.0-alpha.1
tag:     v1.0.0-alpha.1
```

1. Start from current `main`:

   ```powershell
   git switch main
   git pull
   ```

2. Change all three project version files to `1.0.0-alpha.1`.

3. Update `CHANGELOG.md`, moving the finished notes into a dated `1.0.0-alpha.1` section while keeping `Unreleased` above it:

   ```markdown
   ## Unreleased

   No changes yet.

   ## 1.0.0-alpha.1 — 2026-07-18
   ```

4. Verify locally:

   ```powershell
   pnpm install --frozen-lockfile
   pnpm tauri build --bundles nsis
   cd src-tauri
   cargo test
   cd ..
   ```

5. Commit the release preparation:

   ```powershell
   git add package.json src-tauri/tauri.conf.json src-tauri/Cargo.toml CHANGELOG.md
   git commit -m "Prepare release v1.0.0-alpha.1"
   ```

6. Tag and push the release commit:

   ```powershell
   git tag v1.0.0-alpha.1
   git push origin main
   git push origin v1.0.0-alpha.1
   ```

7. Inspect the completed GitHub Actions run and the draft `VoxWeave v1.0.0-alpha.1`, which must be marked as a prerelease.

8. Replace the placeholder GitHub release body with the full `1.0.0-alpha.1` section from `CHANGELOG.md`.

9. Verify the NSIS installer and `VoxWeave-1.0.0-alpha.1-portable-windows-x64.zip`. An MSI is not expected. Download the portable ZIP and verify the marker-selected `data` layout before publishing.

10. Click **Publish release** only after the artifact checks pass.

---

### First Stable Release: `v1.0.0`

Use exactly:

```text
version: 1.0.0
tag:     v1.0.0
```

1. Start from current `main`:

   ```powershell
   git switch main
   git pull
   ```

2. Change all three project version files to `1.0.0`.

3. Update `CHANGELOG.md`. Keep `Unreleased` first, add a dated `1.0.0` section above the alpha section, and summarize any fixes made after the alpha:

   ```markdown
   ## Unreleased

   No changes yet.

   ## 1.0.0 — 2026-07-18

   Stable release of the beta-tested VoxWeave feature set.

   ## 1.0.0-alpha.1 — 2026-07-18
   ```

4. Verify locally:

   ```powershell
   pnpm install --frozen-lockfile
   pnpm tauri build
   cd src-tauri
   cargo test
   cd ..
   ```

5. Commit, tag, and push:

   ```powershell
   git add package.json src-tauri/tauri.conf.json src-tauri/Cargo.toml CHANGELOG.md
   git commit -m "Prepare release v1.0.0"
   git tag v1.0.0
   git push origin main
   git push origin v1.0.0
   ```

6. Inspect the draft `VoxWeave v1.0.0`. It must not be marked as a prerelease. Replace the release body with the `1.0.0` changelog section.

7. Verify the installer assets configured by Tauri and `VoxWeave-1.0.0-portable-windows-x64.zip`. Download and run the portable ZIP from a writable location before publishing.

8. Click **Publish release** only after all assets are verified.

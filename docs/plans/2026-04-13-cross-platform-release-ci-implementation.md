# Cross-Platform Release CI Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add GitHub Actions automation that publishes native macOS, Linux, and Windows release assets for tagged versions and manual release reruns.

**Architecture:** Use one GitHub Actions workflow with a runner matrix for native builds on macOS, Ubuntu, and Windows. Keep versioning outside the workflow, use `GITHUB_TOKEN` for release creation/upload, and support both tag-push and manual-dispatch triggers in the same pipeline.

**Tech Stack:** GitHub Actions, Rust Cargo, shell scripting in workflow steps, GitHub CLI/API-compatible release actions.

---

### Task 1: Add workflow skeleton with dual triggers

**Files:**
- Create: `.github/workflows/release.yml`

**Step 1: Write the failing structural check**

Create the workflow file with the intended top-level triggers and matrix placeholders, then validate that the file exists and contains:

- `push` on tags `v*`
- `workflow_dispatch`
- a job matrix covering macOS, Linux, and Windows

The “failing” part for this task is repository behavior: before the workflow exists, GitHub cannot run cross-platform release builds at all.

**Step 2: Verify the repo currently lacks release CI**

Run: `ls .github/workflows`

Expected: directory missing or no release workflow present.

**Step 3: Write the minimal workflow scaffold**

Add a workflow with:

- workflow name
- both triggers
- `permissions: contents: write`
- a matrix with three targets

**Step 4: Review YAML structure locally**

Run: `python - <<'PY'
import yaml, pathlib
path = pathlib.Path('.github/workflows/release.yml')
print(path.exists())
print(yaml.safe_load(path.read_text())['name'])
PY`

Expected: file parses and prints the workflow name.

### Task 2: Implement native build-and-package steps per platform

**Files:**
- Modify: `.github/workflows/release.yml`

**Step 1: Add matrix metadata**

For each matrix entry, define:

- runner
- Rust target
- binary name
- asset archive name
- archive format

Suggested targets:

- `aarch64-apple-darwin`
- `x86_64-unknown-linux-gnu`
- `x86_64-pc-windows-msvc`

**Step 2: Add Rust setup and build steps**

Use official or minimal Rust setup steps to:

- install the matrix target
- run `cargo build --release --target <target>`

**Step 3: Add packaging steps**

Package the produced binary into:

- `.zip` on macOS
- `.tar.gz` on Linux
- `.zip` on Windows

Ensure filenames follow the chosen release naming format.

**Step 4: Validate the workflow text locally**

Run: `python - <<'PY'
import yaml, pathlib
doc = yaml.safe_load(pathlib.Path('.github/workflows/release.yml').read_text())
print(sorted(doc['on'].keys(), key=str))
print(doc['jobs'].keys())
PY`

Expected: workflow still parses and includes the intended triggers/jobs.

### Task 3: Add release creation and asset upload behavior

**Files:**
- Modify: `.github/workflows/release.yml`

**Step 1: Resolve release tag in one place**

Add steps or environment variables that derive the target tag from:

- pushed tag ref
- manual `workflow_dispatch` input

**Step 2: Create release if needed**

Add a step that creates the GitHub Release when absent.

The step should not fail if the release already exists for a manual rerun path.

**Step 3: Upload or overwrite assets**

Add a step that uploads the packaged archive for each matrix job to the target release.

Prefer overwrite-friendly behavior so reruns can repair a release.

**Step 4: Re-parse the workflow locally**

Run: `python - <<'PY'
import yaml, pathlib
doc = yaml.safe_load(pathlib.Path('.github/workflows/release.yml').read_text())
print(doc['permissions'])
print(list(doc['jobs']))
PY`

Expected: workflow still parses and exposes `contents: write`.

### Task 4: Document the release process briefly

**Files:**
- Modify: `README.md`

**Step 1: Add a small Releases section**

Document:

- tag-driven release path
- manual-dispatch recovery path
- the platforms expected from CI

Keep this short and operational.

**Step 2: Review the README diff**

Run: `git diff -- README.md .github/workflows/release.yml`

Expected: only release workflow/documentation scope is present.

### Task 5: Validate final workflow and repo state

**Files:**
- Review: `.github/workflows/release.yml`
- Review: `README.md`

**Step 1: Run local structural validation**

Run: `python - <<'PY'
import yaml, pathlib
path = pathlib.Path('.github/workflows/release.yml')
doc = yaml.safe_load(path.read_text())
print('workflow_ok', bool(doc))
print('jobs', list(doc['jobs']))
PY`

Expected: successful parse and listed jobs.

**Step 2: Confirm file-size constraints**

Run: `wc -l .github/workflows/release.yml README.md`

Expected: modified source/documentation files remain reasonable and under project limits where applicable.

**Step 3: Inspect final diff**

Run: `git diff -- .github/workflows/release.yml README.md`

Expected: only the release CI workflow and README note changed.

### Task 6: Prepare rollout notes

**Files:**
- Review: `.github/workflows/release.yml`
- Review: `README.md`
- Review: `docs/plans/2026-04-13-cross-platform-release-ci-design.md`
- Review: `docs/plans/2026-04-13-cross-platform-release-ci-implementation.md`

**Step 1: Summarize operational behavior**

Document briefly:

- how to trigger tag-based releases
- how to rerun manually for an existing tag
- what assets will be produced

**Step 2: Note the first live validation path**

Call out that the first complete proof will be a GitHub Actions run on a new tag or manual dispatch, since native matrix builds cannot be fully simulated locally.

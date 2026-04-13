## Cross-Platform Release CI Design

### Goal

Add a GitHub Actions release workflow that can publish native release assets for macOS, Linux, and Windows without relying on local cross-compilation from a developer machine.

### Problem

The current release process is manual and host-dependent. macOS arm64 assets can be built locally on the current machine, but Linux and Windows assets require extra cross-toolchains that are not present. That makes multi-platform releases fragile and hard to repeat.

### Chosen Direction

Use one GitHub Actions workflow with a native OS matrix and two triggers:

- `push` on tags matching `v*`
- `workflow_dispatch` with a tag/version input

This makes tags the default release path while keeping a repair path for rerunning or backfilling assets.

### Alternatives Considered

1. Tag push + manual dispatch in one workflow

Recommended. Gives a canonical release path plus operational recovery without adding a second workflow.

2. Tag push only

Simpler, but less practical when a single asset fails and needs rerunning for an existing release.

3. Manual dispatch only

Flexible, but too easy to make release publishing inconsistent.

### Architecture

Create a single workflow, likely `.github/workflows/release.yml`, that:

- determines the target tag from the event
- runs a build job matrix on native GitHub-hosted runners:
  - `macos-latest`
  - `ubuntu-latest`
  - `windows-latest`
- packages platform-specific release assets
- uploads those assets to the GitHub Release for the target tag

The workflow should avoid custom cross-linkers, Docker-based release logic, or extra release orchestration services in the first version.

### Release Behavior

#### Tag Push

On `push` of `v*`, the workflow should:

- build binaries for all configured platforms
- create the GitHub Release if it does not exist
- upload all assets to that release

#### Manual Dispatch

On `workflow_dispatch`, the workflow should accept a tag input such as `v0.1.2` and:

- reuse that tag as the release target
- create the release if it does not exist
- otherwise upload or overwrite the release assets for that tag

This supports repair and asset backfill.

### Packaging

Recommended asset names:

- `git-reviewer-vX.Y.Z-macos-arm64.zip`
- `git-reviewer-vX.Y.Z-linux-x86_64.tar.gz`
- `git-reviewer-vX.Y.Z-windows-x86_64.zip`

Recommended packaging formats:

- macOS: `.zip`
- Linux: `.tar.gz`
- Windows: `.zip`

Windows assets should contain `git-reviewer.exe`.

### Version Expectations

The workflow assumes the git tag and Rust package version are intentionally aligned by the release process, but it should not try to edit versions itself.

Version bumping stays a source-control concern, not a workflow concern.

### Permissions And Safety

The workflow should request only the permissions it needs, ideally:

- `contents: write`

It should avoid any secret beyond the default `GITHUB_TOKEN` unless a concrete GitHub API limitation appears.

### Implementation Scope

Expected files:

- `.github/workflows/release.yml`
- `README.md` for a short `Releases` section or release workflow note

### Testing And Validation

Validation should focus on keeping the workflow observable and simple:

- ensure trigger parsing works for both tag push and manual dispatch
- ensure asset naming is deterministic
- ensure release creation/upload works when the release already exists
- ensure the workflow does not depend on local machine toolchains

Local verification can only partially validate YAML structure, so the first real end-to-end proof will be a workflow run on GitHub.

### Non-Goals

- No custom release dashboard
- No changelog generation automation in the first version
- No signing/notarization
- No container-based build farm
- No local cross-compilation fallback logic

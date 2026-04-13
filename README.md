# Git Reviewer

`git-reviewer` is a local Rust application for reviewing Git diffs in a GitHub-like browser UI.

It starts a local web server for a target repository, renders the diff against a selected remote base branch, lets you add inline review comments, and exports the review payload to the clipboard with surrounding context.

## Features

- Run from any directory inside a Git worktree: `git-reviewer .`
- Default base branch selection: `origin/main`, then `origin/master`, then first remote branch
- Remote branch dropdown
- Split and unified diff modes
- Directory tree for changed files
- Inline comments on every visible diff line
- Comment modal with add, edit, delete, and cancel
- Local persistence in `localStorage`
- `Refresh` button when repo state changes
- `Changed` and `Stale` comment status after remap
- Clipboard export grouped by `file path - HEAD sha`
- Clipboard fallback modal when browser clipboard write fails
- Hexagonal backend and lightweight hexagonal frontend structure

## Running

Build the binary:

```bash
cargo build
```

Run against the current repo:

```bash
cargo run -- .
```

Run against another repo:

```bash
cargo run -- /path/to/repo
```

Use a fixed port:

```bash
cargo run -- /path/to/repo --port 4000
```

The app prints a local URL such as `http://127.0.0.1:4000` or an ephemeral port if `--port` is omitted.

## Requirements

- Rust toolchain
- `git` installed and available in `PATH`
- A local Git worktree with at least one remote branch
- Node.js only for Playwright tests, not for runtime

## Review Export Format

`Copy to clipboard` exports all comments grouped by file and current `HEAD` sha.

Each comment block contains:

- `file path - HEAD sha`
- the commented line location
- 10 lines above and 10 below the anchor
- the comment text

Base-side comments read context from the base revision. Head-side comments read context from the current worktree content.

## Architecture

## Backend

- `src/domain/`: domain models for repo state, review files, hunks, lines, and comment anchors
- `src/application/`: use cases and ports
- `src/adapters/outbound/`: `git` CLI and filesystem adapters
- `src/adapters/inbound/http/`: HTTP handlers, DTOs, and embedded asset serving
- `src/bootstrap/`: CLI entrypoint, repo resolution, server startup

## Frontend

- `web/core/`: comment remapping, file tree shaping, state helpers
- `web/use-cases/`: shell loading, base switching, refresh watching, export flow
- `web/ports/`: browser-side API contract helpers
- `web/adapters/`: DOM rendering and HTTP transport

## Project Constraints

- Hexagonal architecture is required for both backend and frontend
- Source files must stay at or below 400 lines
- Changes should stay small and responsibility-focused

## Testing

Rust verification:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

Browser verification:

```bash
npm install
npm test
```

## Important Endpoints

- `GET /`
- `GET /api/repo-context`
- `GET /api/review?base=<branch>`
- `GET /api/status`
- `POST /api/clipboard-export`

## Agent Guidance

See `AGENTS.md` for project-specific maintenance rules and `skills/karpathy-guidelines/SKILL.md` for the vendored Karpathy-inspired behavior guidelines.

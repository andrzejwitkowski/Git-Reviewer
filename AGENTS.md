# AGENTS.md

This file defines the most important project rules for future changes.

## Mission

Maintain `git-reviewer` as a local Git diff review tool with a GitHub-like UX, built in Rust with a browser frontend.

## Non-Negotiable Rules

- Keep the backend hexagonal: `domain`, `application`, `ports`, `adapters`, `bootstrap`
- Keep the frontend hexagonal in a lightweight form: `core`, `use-cases`, `ports`, `adapters`
- Do not put business logic in HTTP handlers, browser fetch wrappers, or DOM rendering code
- Do not bypass the application layer from inbound adapters
- Keep every source file at or below 400 lines
- Prefer the smallest correct change
- Do not refactor unrelated code while doing feature work
- Do not remove user changes or unrelated worktree changes unless explicitly asked

## Product Expectations

- `git-reviewer .` must work from any directory inside a Git worktree
- Default base branch priority is `origin/main`, then `origin/master`, then first remote branch
- The left panel is a directory tree of changed files, not smart grouping
- Every visible diff line is commentable
- Comments support add, edit, delete, cancel
- Comments persist in `localStorage`
- Refresh detection is explicit: show `Refresh`, do not auto-reload in the background
- `Changed` means the comment still maps but the line text changed
- `Stale` means the comment could not be remapped confidently
- Clipboard export groups by `file path - HEAD sha` and includes 10 lines above and below

## Backend Guidelines

- Use the system `git` binary through the outbound adapter
- Keep path parsing robust for spaces, quoted paths, renames, deletes, and binary diffs
- Treat malformed HTTP input as a client error when possible
- Keep inbound HTTP handlers thin: map request -> use case -> DTO
- Keep export formatting inside the application/domain layer, not in HTTP handlers

## Frontend Guidelines

- Keep state transitions in `core` or `use-cases`, not inline in rendering helpers
- DOM adapters should render and bind events, not own business rules
- Avoid race conditions when async requests can resolve out of order
- Make UI affordances honest: do not show interactive controls that silently no-op
- Preserve comment remapping behavior across refresh, including rename-safe remap for head-side comments

## Change Workflow

- Prefer TDD for behavior changes and bug fixes
- Add or update tests for every regression-prone change
- Run the full verification set before claiming completion:
  - `cargo fmt --check`
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - `cargo test`
  - `npm test`

## Karpathy Guidelines

This repo vendors the Karpathy-inspired guidelines from:

- `https://github.com/forrestchang/andrej-karpathy-skills`

Local copy:

- `skills/karpathy-guidelines/SKILL.md`

Apply these principles in future changes:

- Think before coding
- Simplicity first
- Surgical changes
- Goal-driven execution

## When Unsure

- Ask before introducing new abstractions
- Ask before widening scope beyond the requested behavior
- Prefer a direct fix over a reusable framework unless reuse already exists in the codebase

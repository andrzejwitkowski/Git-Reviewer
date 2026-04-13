# Local Changes Commit Review Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a synthetic `LOCAL CHANGES` entry at the top of commit review so users can inspect current staged and unstaged work in the same dropdown as historical commits.

**Architecture:** Keep the current branch-review and commit-review split, but extend commit review with a sentinel selection that maps to a worktree diff instead of `parent(commit)..commit`. Build the synthetic entry in the review-commits use case, route the sentinel through the existing `/api/review` commit mode, and keep comment scoping and refresh behavior isolated for local changes.

**Tech Stack:** Rust backend, Axum HTTP API, browser frontend with lightweight hexagonal structure, Playwright E2E tests, Rust integration tests.

---

### Task 1: Model local-changes commit selection

**Files:**
- Modify: `src/domain/repo.rs`
- Modify: `src/adapters/inbound/http/dto.rs`
- Modify: `web/adapters/dom-renderer-helpers.js`
- Test: `tests/http_api.rs`
- Test: `tests/e2e/review-shell-navigation.spec.ts`

**Step 1: Write the failing tests**

Add coverage that the commit list returned by the API starts with a synthetic `LOCAL CHANGES` entry and that the UI renders it first in the commit dropdown.

**Step 2: Run tests to verify they fail**

Run: `cargo test http_api::returns_commits_for_selected_base_branch`
Expected: FAIL because the first entry is still the newest real commit.

Run: `npx playwright test tests/e2e/review-shell-navigation.spec.ts`
Expected: FAIL because the commit dropdown has no `LOCAL CHANGES` entry.

**Step 3: Write minimal implementation**

Extend the commit summary model/DTO with enough information to represent a synthetic entry. Keep the display text simple: `LOCAL CHANGES` with no real SHA dependency in the label.

**Step 4: Run tests to verify they pass**

Run: `cargo test http_api::returns_commits_for_selected_base_branch`
Expected: PASS

Run: `npx playwright test tests/e2e/review-shell-navigation.spec.ts`
Expected: PASS for the new dropdown assertion.

### Task 2: Load local changes diff in commit review mode

**Files:**
- Modify: `src/application/ports/git_repository.rs`
- Modify: `src/adapters/outbound/git_cli/mod.rs`
- Modify: `src/application/use_cases/load_review.rs`
- Modify: `src/application/use_cases/load_review_commits.rs`
- Test: `tests/git_cli.rs`
- Test: `tests/review_use_cases.rs`
- Test: `tests/http_api.rs`

**Step 1: Write the failing tests**

Add tests for a commit-mode request using the sentinel value that should return the current worktree/index diff, and an empty review when there are no local changes.

**Step 2: Run tests to verify they fail**

Run: `cargo test local_changes`
Expected: FAIL because the sentinel is treated as an invalid commit today.

**Step 3: Write minimal implementation**

Add a git repository method for raw local-changes diff against `HEAD`, then branch inside `LoadReview` commit mode so the sentinel loads that diff instead of a historical commit diff. Reuse existing parsing and large-file handling as much as possible.

**Step 4: Run tests to verify they pass**

Run: `cargo test local_changes`
Expected: PASS

### Task 3: Preserve frontend state, comments, and refresh for local changes

**Files:**
- Modify: `web/core/comment-store.js`
- Modify: `web/use-cases/load-shell.js`
- Modify: `web/use-cases/reload-shell.js`
- Modify: `web/use-cases/update-review-mode.js`
- Modify: `web/use-cases/update-review-base.js`
- Modify: `web/use-cases/update-review-commit.js`
- Modify: `web/use-cases/review-request.js`
- Test: `tests/e2e/review-shell-navigation.spec.ts`
- Test: `tests/e2e/review-shell-comments.spec.ts`
- Test: `tests/e2e/review-shell-clipboard-refresh.spec.ts`

**Step 1: Write the failing tests**

Add E2E coverage for:
- `LOCAL CHANGES` selected by default in commit mode
- empty state when the worktree is clean
- comment scoping isolated from real commits
- refresh preserving the `LOCAL CHANGES` selection and updating the diff

**Step 2: Run tests to verify they fail**

Run: `npm test`
Expected: FAIL in the new local-changes commit-review scenarios.

**Step 3: Write minimal implementation**

Treat the sentinel as a normal commit-mode selection in frontend state, but keep its scope distinct in comment storage and preserve it across refresh. Do not add new UI controls; only extend current dropdown behavior.

**Step 4: Run tests to verify they pass**

Run: `npm test`
Expected: PASS

### Task 4: Full verification

**Files:**
- Modify: any touched files above only if verification exposes real issues

**Step 1: Run formatting and linting**

Run: `cargo fmt --check`
Expected: PASS

Run: `cargo clippy --all-targets --all-features -- -D warnings`
Expected: PASS

**Step 2: Run full test suites**

Run: `cargo test`
Expected: PASS

Run: `npm test`
Expected: PASS

**Step 3: Commit**

Run only if explicitly requested:

```bash
git add src web tests docs/plans/2026-04-13-local-changes-commit-review-implementation.md
git commit -m "feat: add local changes commit review"
```

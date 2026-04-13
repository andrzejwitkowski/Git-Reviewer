# Inline Comment Thread Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Restyle the inline comment thread under diff lines into a compact, code-aligned annotation while preserving existing behavior.

**Architecture:** Keep the current inline comment row and existing state model. Make a minimal renderer change to support clearer structure and move the redesign into CSS so behavior, data flow, and modal editing remain unchanged.

**Tech Stack:** Rust backend, browser JavaScript frontend, Playwright end-to-end tests, npm test runner.

---

### Task 1: Add failing coverage for compact inline thread behavior

**Files:**
- Modify: `tests/e2e/review-shell-comments.spec.ts`

**Step 1: Write the failing test**

Add assertions to the existing inline comment test that verify:

- the header stays visible after collapse
- the inline comment keeps the existing title text
- a changed or stale status renders in the inline header when present
- split mode keeps `colspan="3"`

Use the current test structure and fixture pattern already present in `tests/e2e/review-shell-comments.spec.ts`.

**Step 2: Run test to verify it fails**

Run: `npm test -- review-shell-comments.spec.ts`

Expected: at least one assertion fails because the current inline comment markup and styling do not yet match the compact thread expectations.

**Step 3: Keep the failure narrow**

If more than one unrelated assertion fails, trim the new assertions so the test only captures the compact thread expectations needed for this change.

**Step 4: Re-run test to confirm the targeted failure**

Run: `npm test -- review-shell-comments.spec.ts`

Expected: one or more failures directly tied to the new inline comment expectations.

### Task 2: Update renderer structure for a compact thread shell

**Files:**
- Modify: `web/adapters/dom-renderer.js`
- Test: `tests/e2e/review-shell-comments.spec.ts`

**Step 1: Change only the inline comment renderer markup**

Update `renderInlineComment` so the comment row renders a compact thread structure with clear class names for:

- thread shell
- summary/header row
- title
- optional status element
- body section

Keep:

- `data-testid="inline-comment"`
- `data-testid="inline-comment-cell"`
- `data-testid="inline-comment-header"`
- `data-testid="inline-comment-toggle"`
- `data-testid="inline-comment-body"`
- current `colspan` logic for unified and split modes

**Step 2: Preserve existing behavior**

Do not change:

- collapse state storage
- modal editing flow
- line selection behavior
- comment marker behavior
- status text mapping helpers unless needed for header rendering

**Step 3: Run the focused test**

Run: `npm test -- review-shell-comments.spec.ts`

Expected: tests still fail, but only for styling or final expected structure that CSS has not satisfied yet.

### Task 3: Restyle the inline comment as a compact code-adjacent thread

**Files:**
- Modify: `web/styles.css`
- Test: `tests/e2e/review-shell-comments.spec.ts`

**Step 1: Replace the stretched card styling**

Adjust the existing inline comment CSS so the thread:

- feels attached to the code cell
- uses tighter spacing
- has a compact summary/header
- avoids reading like a full-width panel

**Step 2: Add non-active status treatment**

Style a visible but restrained status pill or label for `Changed` and `Stale` inside the header.

Do not render special chrome for active comments beyond the normal compact header.

**Step 3: Keep collapsed state readable**

Ensure collapsing the thread hides the body while leaving the header row crisp and legible.

**Step 4: Check responsive behavior**

Make sure the compact thread still fits the existing mobile layout without horizontal overflow caused by padding or status treatment.

**Step 5: Run the focused test**

Run: `npm test -- review-shell-comments.spec.ts`

Expected: the inline comment test passes.

### Task 4: Verify split and unified diff compatibility

**Files:**
- Modify: `tests/e2e/review-shell-comments.spec.ts` if a small extra assertion is still needed
- Test: `tests/e2e/review-shell-comments.spec.ts`

**Step 1: Add or refine a split-mode assertion if missing**

If the focused test still only covers unified mode, add the smallest possible assertion that creates a comment in split mode and checks `colspan="3"`.

**Step 2: Run the focused test file again**

Run: `npm test -- review-shell-comments.spec.ts`

Expected: all tests in that file pass in both unified and split scenarios.

### Task 5: Run project verification for this change

**Files:**
- No code changes expected

**Step 1: Run frontend tests**

Run: `npm test`

Expected: full browser test suite passes.

**Step 2: Run backend tests**

Run: `cargo test`

Expected: Rust test suite passes, confirming the frontend change did not break server-backed flows.

**Step 3: Optional formatter and lint verification if touched indirectly**

If any Rust files changed unexpectedly, also run:

- `cargo fmt --check`
- `cargo clippy --all-targets --all-features -- -D warnings`

Expected: both pass.

### Task 6: Review changed files and prepare completion handoff

**Files:**
- Review: `web/adapters/dom-renderer.js`
- Review: `web/styles.css`
- Review: `tests/e2e/review-shell-comments.spec.ts`
- Review: `docs/plans/2026-04-13-inline-comment-thread-design.md`
- Review: `docs/plans/2026-04-13-inline-comment-thread-implementation.md`

**Step 1: Inspect the diff for scope control**

Confirm the change stayed focused on inline comment rendering and testing only.

**Step 2: Confirm file-size constraint**

Check that no modified source file exceeds 400 lines after the change.

**Step 3: Prepare summary**

Summarize:

- what changed in the inline thread presentation
- what tests were run
- whether any residual visual risk remains

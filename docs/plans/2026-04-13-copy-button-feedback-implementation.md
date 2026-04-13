# Copy Button Feedback Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add clear, temporary success feedback to the `Copy to clipboard` button after a successful clipboard write.

**Architecture:** Keep the change entirely in the frontend. Add a tiny transient UI state for the copy button, update the existing export flow to set and clear that state only on successful clipboard writes, and render the button label and styling from that state.

**Tech Stack:** Browser JavaScript, Playwright end-to-end tests, CSS.

---

### Task 1: Add failing browser coverage for copy success feedback

**Files:**
- Modify: `tests/e2e/review-shell-clipboard-refresh.spec.ts`

**Step 1: Write the failing test**

Extend the successful clipboard export test so it verifies:

- the button initially reads `Copy to clipboard`
- after a successful click it changes to `Copied`
- it later resets to `Copy to clipboard`

Keep the existing clipboard interception and export payload assertions.

**Step 2: Run test to verify it fails**

Run: `npm test -- review-shell-clipboard-refresh.spec.ts`

Expected: FAIL because the button label never changes from `Copy to clipboard`.

**Step 3: Keep the failure narrow**

If multiple new assertions fail, reduce scope so the only missing behavior is the success feedback.

**Step 4: Re-run the focused test file**

Run: `npm test -- review-shell-clipboard-refresh.spec.ts`

Expected: targeted failure only.

### Task 2: Add transient copy feedback state and action flow

**Files:**
- Modify: `web/core/review-shell-state.js`
- Modify: `web/use-cases/export-comments.js`
- Modify: `web/app.js`
- Test: `tests/e2e/review-shell-clipboard-refresh.spec.ts`

**Step 1: Add state for button feedback**

Extend shell state with a minimal copy feedback mode, such as `copyState: 'idle'`.

**Step 2: Update export flow**

Modify the export action so the success state is set only after `navigator.clipboard.writeText(...)` succeeds.

Do not set success state when:

- clipboard fallback modal is shown
- export generation fails

**Step 3: Handle automatic reset**

Add the smallest timer-based reset in the app action layer so the button returns to idle after roughly 1.5 to 2 seconds.

Keep timer ownership local to the copy action flow instead of pushing timing logic into the renderer.

**Step 4: Run the focused clipboard test file**

Run: `npm test -- review-shell-clipboard-refresh.spec.ts`

Expected: success-feedback assertions pass, or only rendering/styling assertions remain.

### Task 3: Render and style the success state

**Files:**
- Modify: `web/adapters/dom-renderer.js`
- Modify: `web/styles.css`
- Test: `tests/e2e/review-shell-clipboard-refresh.spec.ts`

**Step 1: Render label from state**

Update the existing copy button rendering so it reflects the copy feedback state.

Expected labels:

- idle: `Copy to clipboard`
- copied: `Copied`

**Step 2: Add success styling**

Add a clear but lightweight success treatment to the button, for example a green-tinted background/border state, while keeping the existing control-group look.

**Step 3: Keep disabled behavior intact**

Preserve the current disabled behavior when there are no comments.

**Step 4: Run the focused clipboard test file**

Run: `npm test -- review-shell-clipboard-refresh.spec.ts`

Expected: the successful copy feedback test passes.

### Task 4: Protect fallback and error paths

**Files:**
- Modify: `tests/e2e/review-shell-clipboard-refresh.spec.ts` if needed
- Test: `tests/e2e/review-shell-clipboard-refresh.spec.ts`

**Step 1: Add or refine assertions for non-success flows**

Verify the smallest possible expectations that:

- fallback modal path does not show `Copied`
- export error path does not show `Copied`

Reuse the existing fallback and error tests.

**Step 2: Run the focused clipboard test file again**

Run: `npm test -- review-shell-clipboard-refresh.spec.ts`

Expected: full clipboard/refresh test file passes.

### Task 5: Run verification for the change

**Files:**
- No code changes expected

**Step 1: Run frontend tests**

Run: `npm test`

Expected: all Playwright tests pass.

**Step 2: Run backend tests**

Run: `cargo test`

Expected: Rust tests pass.

**Step 3: Run full project gate if any Rust files changed unexpectedly**

If Rust code was touched, also run:

- `cargo fmt --check`
- `cargo clippy --all-targets --all-features -- -D warnings`

Expected: both pass.

### Task 6: Review scope and prepare handoff

**Files:**
- Review: `web/core/review-shell-state.js`
- Review: `web/use-cases/export-comments.js`
- Review: `web/app.js`
- Review: `web/adapters/dom-renderer.js`
- Review: `web/styles.css`
- Review: `tests/e2e/review-shell-clipboard-refresh.spec.ts`
- Review: `docs/plans/2026-04-13-copy-button-feedback-design.md`
- Review: `docs/plans/2026-04-13-copy-button-feedback-implementation.md`

**Step 1: Inspect diff for scope control**

Confirm the change stayed limited to copy-button feedback and related tests.

**Step 2: Confirm file-size constraint**

Check that no modified source file exceeds 400 lines.

**Step 3: Prepare summary**

Summarize:

- what changed in the copy button behavior
- what tests were run
- any residual UX risk

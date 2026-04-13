## Inline Comment Thread Design

### Goal

Improve the inline comment presentation rendered directly under a commented diff line so it feels attached to the code instead of looking like a stretched standalone card.

### Constraints

- Keep the existing product model: inline comments stay under the line they annotate.
- Keep edit and delete actions in the existing modal.
- Preserve collapse and expand behavior.
- Preserve unified and split diff support.
- Keep the change small and responsibility-focused.

### Chosen Direction

Use a compact thread presentation.

This keeps the current table-row placement, but changes the visual treatment from a full card into a tighter code-adjacent annotation. It fits the existing diff layout better than a callout bubble and remains more legible than a nearly invisible strip.

### Alternatives Considered

1. Compact thread

Recommended. Best match for the current diff table and the expected GitHub-like review model. It fixes the stretched-card problem without introducing a new interaction model.

2. Callout bubble

More expressive, but more visually decorative and more likely to feel disconnected from the line-based diff table.

3. Minimal strip

Simplest styling option, but too subtle for the primary way comments remain visible in the interface.

### UI Structure

Keep the existing inline comment row:

- `tr.inline-comment-row`
- `td[data-testid="inline-comment-cell"]`

Inside the cell, render a compact thread shell with:

- a single-row summary/header
- a collapse toggle on the left
- a title such as `Comment on line R12`
- an optional status pill on the right for `Changed` or `Stale`
- the comment body below the header when expanded

This keeps the markup close to the current implementation while making the visual hierarchy clearer.

### Visual Rules

- Align the thread visually with the code cell, not the whole table width.
- Reduce padding and radius so the element reads as an inline annotation.
- Use a subtle background lift and border to separate the thread from the diff row.
- Keep the header crisp and compact.
- Only render a visible status treatment when the comment is not active.
- When collapsed, show only the compact summary row.

### Data And Behavior

No data-model changes are needed.

The following behavior stays unchanged:

- inline comment remains attached to the same line anchor
- collapse state is tracked by `collapsedCommentIds`
- status still derives from existing comment remap logic
- modal remains the only edit and delete surface

### Implementation Scope

Expected files:

- `web/adapters/dom-renderer.js`
- `web/styles.css`
- `tests/e2e/review-shell-comments.spec.ts`

The renderer change should stay minimal and primarily support cleaner class naming for the compact thread structure. Most of the redesign should live in CSS.

### Testing

Keep existing inline comment behavior coverage and add assertions for:

- compact thread header remains visible when body is collapsed
- non-active statuses render as explicit pills or labels in the header
- unified mode keeps `colspan="2"`
- split mode keeps `colspan="3"`

### Non-Goals

- No return of the removed right-side comments column
- No inline editing inside the diff row
- No comment threading or replies
- No broader diff layout redesign

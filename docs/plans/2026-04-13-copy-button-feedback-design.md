## Copy Button Feedback Design

### Goal

Make the `Copy to clipboard` action feel interactive and clearly acknowledged when it succeeds.

### Problem

The current button triggers clipboard export correctly, but the UI does not visibly change after a successful click. That makes the action feel inert even when it worked.

### Chosen Direction

Use an inline success state on the existing button.

After a successful clipboard write, the button should temporarily change both label and styling so the user gets confirmation exactly where the action happened.

### Alternatives Considered

1. Success state on the button

Recommended. Lowest complexity, most local feedback, and no need to introduce a new notification surface.

2. Pulse-only feedback

Too subtle for an action that has no other obvious visible output.

3. Toast message

Clear, but heavier than needed for this workflow and introduces a separate feedback pattern.

### Behavior

- Default state: `Copy to clipboard`
- Success state after clipboard write succeeds:
  - label becomes `Copied`
  - button gets a success visual treatment
  - state resets automatically after about 1.5 to 2 seconds
- If the clipboard API fails and fallback modal opens, do not show success state
- If export generation fails and the error banner is shown, do not show success state
- Repeated successful clicks can restart the success timer

### UI Rules

- Keep the button in the same location and size class as the current control group
- Use a distinct but consistent success color treatment
- Add a pressed/settled visual cue so the interaction feels intentional, not just text replacement
- Avoid introducing a spinner or loading state unless the copy operation becomes asynchronous enough to justify it

### State Changes

Add a small transient state to the frontend shell state, such as a copy button feedback mode.

The feature only needs two states:

- `idle`
- `copied`

The timer for resetting should be owned close to the action handler so rendering stays simple.

### Implementation Scope

Expected files:

- `web/core/review-shell-state.js`
- `web/use-cases/export-comments.js`
- `web/app.js`
- `web/adapters/dom-renderer.js`
- `web/styles.css`
- `tests/e2e/review-shell-clipboard-refresh.spec.ts`

### Testing

Add or adjust browser coverage for:

- successful copy changes the button text to `Copied`
- the success state resets back to `Copy to clipboard`
- clipboard fallback does not falsely show the success state
- export failure still shows the error banner without success styling

### Non-Goals

- No toast system
- No global notification center
- No backend API changes
- No change to clipboard export payload or fallback modal contents

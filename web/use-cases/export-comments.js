export const exportComments = async (port, state, hooks) => {
  const comments = state.comments.filter((comment) => comment.status !== 'stale');
  let text = '';

  try {
    text = await port.exportClipboard(state.review.headSha, comments);
  } catch (error) {
    state.error = error.message;
    return;
  }

  try {
    await navigator.clipboard.writeText(text);
    state.clipboardFallbackText = '';
    state.error = null;
  } catch {
    state.clipboardFallbackText = text;
    hooks.onClipboardFallback?.();
  }
};

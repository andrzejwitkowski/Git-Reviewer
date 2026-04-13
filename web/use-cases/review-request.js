export const buildReviewRequest = (state, baseBranch) => ({
  mode: state.reviewMode,
  base: baseBranch,
  commit: state.reviewMode === 'commit' ? state.selectedCommit : null,
  expandedPaths: state.expandedPaths
});

export const selectInitialPath = (files) => {
  return files.find((file) => file.path.startsWith('src/'))?.path
    ?? files[0]?.path
    ?? null;
};

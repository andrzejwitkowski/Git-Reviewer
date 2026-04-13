export const createShellState = () => ({
  repoContext: null,
  review: null,
  repoSnapshot: null,
  nextSnapshot: null,
  expandedPaths: [],
  selectedPath: null,
  diffMode: 'unified',
  comments: [],
  activeCommentId: null,
  clipboardFallbackText: '',
  pendingRefresh: false,
  error: null,
  isLoading: true,
  reviewRequestId: 0,
  collapsedCommentIds: []
});

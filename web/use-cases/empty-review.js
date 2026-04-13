export const createEmptyCommitReview = (baseBranch) => ({
  reviewMode: 'commit',
  baseBranch,
  selectedCommit: null,
  mergeBaseSha: '',
  headSha: '',
  files: []
});

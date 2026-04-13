import { activeScope, loadStoredComments } from '../core/comment-store.js';

export const loadShell = async (port, state, storage) => {
  state.isLoading = true;
  state.error = null;

  const repoContext = await port.loadRepoContext();
  const review = await port.loadReview(repoContext.defaultBaseBranch, state.expandedPaths);
  const repoSnapshot = await port.loadRepoStatus();

  state.repoContext = repoContext;
  state.review = review;
  state.repoSnapshot = repoSnapshot;
  state.selectedPath = selectInitialPath(review.files);
  state.comments = loadStoredComments(storage, activeScope(repoContext, review));
  state.isLoading = false;
};

const selectInitialPath = (files) => {
  return files.find((file) => file.path.startsWith('src/'))?.path
    ?? files[0]?.path
    ?? null;
};

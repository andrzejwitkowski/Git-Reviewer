import { activeScope, loadStoredComments, moveStoredComments, saveStoredComments } from '../core/comment-store.js';
import { remapComments } from '../core/review-comments.js';

export const reloadShell = async (port, state, storage) => {
  const previousScope = activeScope(state.repoContext, state.review);
  const previousComments = state.comments;
  const repoContext = await port.loadRepoContext();
  const review = await port.loadReview(
    state.review?.baseBranch || repoContext.defaultBaseBranch,
    state.expandedPaths
  );
  const nextScope = activeScope(repoContext, review);
  const persistedComments = previousScope.headSha === nextScope.headSha
    ? loadStoredComments(storage, nextScope)
    : remapComments(previousComments, review);

  state.repoContext = repoContext;
  state.review = review;
  state.selectedPath = review.files.find((file) => file.path === state.selectedPath)?.path
    || review.files[0]?.path
    || null;
  state.comments = persistedComments;
  state.pendingRefresh = false;
  state.repoSnapshot = await port.loadRepoStatus();
  state.isLoading = false;

  if (previousScope.headSha === nextScope.headSha) {
    saveStoredComments(storage, nextScope, persistedComments);
  } else {
    moveStoredComments(storage, previousScope, nextScope, persistedComments);
  }
};

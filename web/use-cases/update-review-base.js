import { activeScope, loadStoredComments } from '../core/comment-store.js';

export const updateReviewBase = async (port, state, storage, baseBranch) => {
  state.isLoading = true;
  state.error = null;
  state.expandedPaths = [];
  state.reviewRequestId += 1;
  const requestId = state.reviewRequestId;

  let review;

  try {
    review = await port.loadReview(baseBranch, state.expandedPaths);
  } catch (error) {
    if (requestId !== state.reviewRequestId) {
      return false;
    }

    throw error;
  }

  if (requestId !== state.reviewRequestId) {
    return false;
  }

  state.review = review;
  state.comments = loadStoredComments(storage, activeScope(state.repoContext, review));
  state.selectedPath = review.files.find((file) => file.path === state.selectedPath)?.path
    ?? review.files[0]?.path
    ?? null;
  state.isLoading = false;
  return true;
};

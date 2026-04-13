import { activeScope, loadStoredComments } from '../core/comment-store.js';
import { resetTransientReviewState } from './review-state-reset.js';
import { buildReviewRequest, selectInitialPath } from './review-request.js';

export const updateReviewCommit = async (port, state, storage, commitSha) => {
  state.isLoading = true;
  state.error = null;
  state.expandedPaths = [];
  state.reviewRequestId += 1;
  const requestId = state.reviewRequestId;
  const baseBranch = state.review?.baseBranch || state.repoContext?.defaultBaseBranch;
  resetTransientReviewState(state);

  try {
    const review = await port.loadReview(buildReviewRequest({
      ...state,
      selectedCommit: commitSha
    }, baseBranch));
    if (requestId !== state.reviewRequestId) {
      return false;
    }

    state.selectedCommit = commitSha;
    state.review = review;
    state.comments = loadStoredComments(storage, activeScope(state.repoContext, review));
    state.selectedPath = selectInitialPath(review.files);
    state.isLoading = false;
    return true;
  } catch (error) {
    if (requestId !== state.reviewRequestId) {
      return false;
    }

    throw error;
  }
};

import { activeScope, loadStoredComments } from '../core/comment-store.js';
import { loadStoredReviewPreferences } from '../core/review-preferences.js';
import { createEmptyCommitReview } from './empty-review.js';
import { resetTransientReviewState } from './review-state-reset.js';
import { buildReviewRequest, selectInitialPath } from './review-request.js';

export const updateReviewMode = async (port, state, storage, reviewMode) => {
  state.isLoading = true;
  state.error = null;
  state.expandedPaths = [];
  state.reviewRequestId += 1;
  const requestId = state.reviewRequestId;
  const baseBranch = state.review?.baseBranch || state.repoContext?.defaultBaseBranch;
  let availableCommits = [];
  let selectedCommit = null;

  resetTransientReviewState(state);

  try {
    if (reviewMode === 'commit') {
      availableCommits = await port.loadCommits(baseBranch);
      selectedCommit = availableCommits[0]?.sha || null;
      if (!selectedCommit) {
        if (requestId !== state.reviewRequestId) {
          return false;
        }
        const review = createEmptyCommitReview(baseBranch);
        state.reviewMode = reviewMode;
        state.availableCommits = availableCommits;
        state.selectedCommit = null;
        state.review = review;
        state.comments = loadStoredComments(storage, activeScope(state.repoContext, review));
        state.viewedPaths = loadStoredReviewPreferences(storage, activeScope(state.repoContext, review)).viewedPaths;
        state.selectedPath = null;
        state.isLoading = false;
        return true;
      }
    }

    const review = await port.loadReview(buildReviewRequest({
      ...state,
      reviewMode,
      selectedCommit
    }, baseBranch));
    if (requestId !== state.reviewRequestId) {
      return false;
    }

    state.reviewMode = reviewMode;
    state.availableCommits = availableCommits;
    state.selectedCommit = selectedCommit;
    state.review = review;
    state.comments = loadStoredComments(storage, activeScope(state.repoContext, review));
    state.viewedPaths = loadStoredReviewPreferences(storage, activeScope(state.repoContext, review)).viewedPaths;
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

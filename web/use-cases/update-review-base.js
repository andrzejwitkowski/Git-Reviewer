import { activeScope, loadStoredComments } from '../core/comment-store.js';
import { loadStoredReviewPreferences } from '../core/review-preferences.js';
import { createEmptyCommitReview } from './empty-review.js';
import { resetTransientReviewState } from './review-state-reset.js';
import { buildReviewRequest, selectInitialPath } from './review-request.js';

export const updateReviewBase = async (port, state, storage, baseBranch) => {
  state.isLoading = true;
  state.error = null;
  state.expandedPaths = [];
  resetTransientReviewState(state);
  state.reviewRequestId += 1;
  const requestId = state.reviewRequestId;

  let review;
  let availableCommits = [];
  let selectedCommit = null;

  try {
    if (state.reviewMode === 'commit') {
      availableCommits = await port.loadCommits(baseBranch);
      selectedCommit = availableCommits[0]?.sha || null;
      if (!selectedCommit) {
        if (requestId !== state.reviewRequestId) {
          return false;
        }
        const review = createEmptyCommitReview(baseBranch);
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
    review = await port.loadReview(buildReviewRequest({
      ...state,
      selectedCommit
    }, baseBranch));
  } catch (error) {
    if (requestId !== state.reviewRequestId) {
      return false;
    }

    throw error;
  }

  if (requestId !== state.reviewRequestId) {
    return false;
  }

  state.availableCommits = availableCommits;
  state.selectedCommit = selectedCommit;
  state.review = review;
  state.comments = loadStoredComments(storage, activeScope(state.repoContext, review));
  state.viewedPaths = loadStoredReviewPreferences(storage, activeScope(state.repoContext, review)).viewedPaths;
  state.selectedPath = review.files.find((file) => file.path === state.selectedPath)?.path
    ?? selectInitialPath(review.files);
  state.isLoading = false;
  return true;
};

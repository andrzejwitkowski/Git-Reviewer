import { activeScope, loadStoredComments, moveStoredComments, saveStoredComments } from '../core/comment-store.js';
import { loadStoredReviewPreferences } from '../core/review-preferences.js';
import { remapComments } from '../core/review-comments.js';
import { createEmptyCommitReview } from './empty-review.js';
import { resetTransientReviewState } from './review-state-reset.js';
import { buildReviewRequest, selectInitialPath } from './review-request.js';

export const reloadShell = async (port, state, storage) => {
  const previousScope = activeScope(state.repoContext, state.review);
  const previousComments = state.comments;
  resetTransientReviewState(state);
  const repoContext = await port.loadRepoContext();
  const baseBranch = state.review?.baseBranch || repoContext.defaultBaseBranch;
  let availableCommits = [];
  let selectedCommit = null;
  if (state.reviewMode === 'commit') {
    availableCommits = await port.loadCommits(baseBranch);
    selectedCommit = availableCommits.find((entry) => entry.sha === state.selectedCommit)?.sha
      || availableCommits[0]?.sha
      || null;
    if (!selectedCommit) {
      const review = createEmptyCommitReview(baseBranch);
      const nextScope = activeScope(repoContext, review);
      const persistedComments = loadStoredComments(storage, nextScope);

      state.repoContext = repoContext;
      state.availableCommits = availableCommits;
      state.selectedCommit = null;
      state.review = review;
      state.selectedPath = null;
      state.comments = persistedComments;
      state.viewedPaths = loadStoredReviewPreferences(storage, nextScope).viewedPaths;
      state.pendingRefresh = false;
      state.repoSnapshot = await port.loadRepoStatus();
      state.isLoading = false;
      saveStoredComments(storage, nextScope, persistedComments);
      return;
    }
  }
  const review = await port.loadReview(buildReviewRequest({
    ...state,
    selectedCommit
  }, baseBranch));
  const nextScope = activeScope(repoContext, review);
  const shouldRemap = previousScope.reviewMode === nextScope.reviewMode
    && previousScope.baseBranch === nextScope.baseBranch
    && previousScope.selectedCommit === nextScope.selectedCommit;
  const shouldReuseStoredComments = shouldRemap
    && previousScope.selectedCommit !== 'LOCAL_CHANGES'
    && previousScope.headSha === nextScope.headSha;
  const persistedComments = !shouldRemap
    ? loadStoredComments(storage, nextScope)
    : shouldReuseStoredComments
    ? loadStoredComments(storage, nextScope)
    : remapComments(previousComments, review);

  state.repoContext = repoContext;
  state.availableCommits = availableCommits;
  state.selectedCommit = selectedCommit;
  state.review = review;
  state.selectedPath = review.files.find((file) => file.path === state.selectedPath)?.path
    || selectInitialPath(review.files);
  state.comments = persistedComments;
  state.viewedPaths = loadStoredReviewPreferences(storage, nextScope).viewedPaths;
  state.pendingRefresh = false;
  state.repoSnapshot = await port.loadRepoStatus();
  state.isLoading = false;

  if (!shouldRemap || shouldReuseStoredComments) {
    saveStoredComments(storage, nextScope, persistedComments);
  } else {
    moveStoredComments(storage, previousScope, nextScope, persistedComments);
  }
};

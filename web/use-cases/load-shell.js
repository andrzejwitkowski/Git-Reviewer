import { activeScope, loadStoredComments } from '../core/comment-store.js';
import { buildReviewRequest, selectInitialPath } from './review-request.js';

export const loadShell = async (port, state, storage) => {
  state.isLoading = true;
  state.error = null;

  const repoContext = await port.loadRepoContext();
  const review = await port.loadReview(buildReviewRequest(state, repoContext.defaultBaseBranch));
  const repoSnapshot = await port.loadRepoStatus();

  state.repoContext = repoContext;
  state.availableCommits = [];
  state.review = review;
  state.repoSnapshot = repoSnapshot;
  state.selectedPath = selectInitialPath(review.files);
  state.comments = loadStoredComments(storage, activeScope(repoContext, review));
  state.isLoading = false;
};

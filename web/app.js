import { createShellState } from './core/review-shell-state.js';
import { activeScope, saveStoredComments } from './core/comment-store.js';
import { buildLineCatalog, commentIndex, createCommentDraft } from './core/review-comments.js';
import { loadShell } from './use-cases/load-shell.js';
import { reloadShell } from './use-cases/reload-shell.js';
import { updateReviewBase } from './use-cases/update-review-base.js';
import { updateReviewMode } from './use-cases/update-review-mode.js';
import { updateReviewCommit } from './use-cases/update-review-commit.js';
import { buildReviewRequest } from './use-cases/review-request.js';
import { exportComments } from './use-cases/export-comments.js';
import { watchRefresh } from './use-cases/watch-refresh.js';
import { createHttpReviewPort } from './adapters/http-review-port.js';
import { createShellRenderer } from './adapters/dom-renderer.js';

const COPY_FEEDBACK_DURATION_MS = 1500;

const bootstrap = async () => {
  const port = createHttpReviewPort();
  const state = createShellState();
  const storage = window.localStorage;
  const renderer = createShellRenderer(document, {
    onBaseBranchChange: async (branch) => {
      try {
        const applied = await updateReviewBase(port, state, storage, branch);
        if (!applied) {
          return;
        }
      } catch (error) {
        state.error = error.message;
        state.isLoading = false;
      }
      renderShell();
    },
    onReviewModeChange: async (reviewMode) => {
      try {
        const applied = await updateReviewMode(port, state, storage, reviewMode);
        if (!applied) {
          return;
        }
      } catch (error) {
        state.error = error.message;
        state.isLoading = false;
      }
      renderShell();
    },
    onCommitChange: async (commitSha) => {
      try {
        const applied = await updateReviewCommit(port, state, storage, commitSha);
        if (!applied) {
          return;
        }
      } catch (error) {
        state.error = error.message;
        state.isLoading = false;
      }
      renderShell();
    },
    onDiffModeChange: (mode) => {
      state.diffMode = mode;
      renderShell();
    },
    onFileSelect: (path) => {
      state.selectedPath = path;
      renderShell();
    },
    onLargeFileFetch: async (path) => {
      if (state.expandedPaths.includes(path)) {
        return;
      }
      state.expandedPaths = state.expandedPaths.concat(path);
      state.selectedPath = path;
      state.isLoading = true;
      renderShell();
      try {
        const review = await port.loadReview(buildReviewRequest(state, state.review?.baseBranch));
        state.review = review;
        state.selectedPath = path;
        state.isLoading = false;
      } catch (error) {
        state.error = error.message;
        state.isLoading = false;
      }
      renderShell();
    },
    onLineSelect: (payload) => {
      const catalog = buildLineCatalog(state.review);
      const record = catalog.byKey.get(payload.lineKey);
      const existing = commentIndex(state.comments).get(payload.lineKey) || null;
      state.activeCommentId = existing?.id || null;
      state.modalDraft = {
        lineKey: payload.lineKey,
        body: existing?.body || '',
        path: record?.path || payload.path,
        status: existing?.status || 'active'
      };
      renderShell();
    },
    onCommentCancel: () => {
      state.activeCommentId = null;
      state.modalDraft = null;
      renderShell();
    },
    onCommentChange: (body) => {
      state.modalDraft.body = body;
    },
    onCommentSave: () => {
      const catalog = buildLineCatalog(state.review);
      const record = catalog.byKey.get(state.modalDraft.lineKey);
      if (!record || !state.modalDraft.body.trim()) {
        return;
      }
      const draft = createCommentDraft(state.review, record, state.modalDraft.body.trim(), state.activeCommentId);
      state.comments = state.comments.filter((comment) => comment.id !== draft.id).concat(draft);
      saveStoredComments(storage, activeScope(state.repoContext, state.review), state.comments);
      state.activeCommentId = null;
      state.modalDraft = null;
      renderShell();
    },
    onCommentDelete: () => {
      state.comments = state.comments.filter((comment) => comment.id !== state.activeCommentId);
      state.collapsedCommentIds = state.collapsedCommentIds.filter((id) => id !== state.activeCommentId);
      saveStoredComments(storage, activeScope(state.repoContext, state.review), state.comments);
      state.activeCommentId = null;
      state.modalDraft = null;
      renderShell();
    },
    onCopyToClipboard: async () => {
      resetCopyFeedback();
      renderShell();
      await exportComments(port, state, {
        onClipboardCopySuccess: startCopyFeedbackReset,
        onClipboardFallback: renderShell
      });
      renderShell();
    },
    onClipboardFallbackClose: () => {
      state.clipboardFallbackText = '';
      renderShell();
    },
    onRefresh: async () => {
      state.isLoading = true;
      state.error = null;
      renderShell();

      try {
        await reloadShell(port, state, storage);
      } catch (error) {
        state.error = error.message;
        state.isLoading = false;
      }

      renderShell();
    },
    onStaleCommentSelect: (commentId) => {
      state.activeCommentId = commentId;
      const comment = state.comments.find((entry) => entry.id === commentId);
      state.modalDraft = {
        lineKey: '',
        body: comment?.body || '',
        path: comment?.mapping.path || '',
        status: comment?.status || 'stale'
      };
      renderShell();
    },
    onInlineCommentToggle: (commentId) => {
      if (state.collapsedCommentIds.includes(commentId)) {
        state.collapsedCommentIds = state.collapsedCommentIds.filter((id) => id !== commentId);
      } else {
        state.collapsedCommentIds = state.collapsedCommentIds.concat(commentId);
      }
      renderShell();
    }
  });
  let copyFeedbackResetId = null;

  const renderShell = () => {
    renderer.render(state);
  };

  const resetCopyFeedback = () => {
    if (copyFeedbackResetId !== null) {
      window.clearTimeout(copyFeedbackResetId);
      copyFeedbackResetId = null;
    }
    state.clipboardCopied = false;
  };

  const startCopyFeedbackReset = () => {
    if (copyFeedbackResetId !== null) {
      window.clearTimeout(copyFeedbackResetId);
    }
    copyFeedbackResetId = window.setTimeout(() => {
      state.clipboardCopied = false;
      copyFeedbackResetId = null;
      renderShell();
    }, COPY_FEEDBACK_DURATION_MS);
  };

  renderShell();

  try {
    await loadShell(port, state, storage);
  } catch (error) {
    state.error = error.message;
    state.isLoading = false;
  }

  renderShell();
  watchRefresh(window, port, state, renderShell);
};

bootstrap();

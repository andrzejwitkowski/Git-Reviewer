import { createShellState } from './core/review-shell-state.js';
import { activeScope, saveStoredComments } from './core/comment-store.js';
import { buildLineCatalog, commentIndex, createCommentDraft } from './core/review-comments.js';
import { loadShell } from './use-cases/load-shell.js';
import { reloadShell } from './use-cases/reload-shell.js';
import { updateReviewBase } from './use-cases/update-review-base.js';
import { exportComments } from './use-cases/export-comments.js';
import { watchRefresh } from './use-cases/watch-refresh.js';
import { createHttpReviewPort } from './adapters/http-review-port.js';
import { createShellRenderer } from './adapters/dom-renderer.js';

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
      renderer.render(state);
    },
    onDiffModeChange: (mode) => {
      state.diffMode = mode;
      renderer.render(state);
    },
    onFileSelect: (path) => {
      state.selectedPath = path;
      renderer.render(state);
    },
    onLargeFileFetch: async (path) => {
      if (state.expandedPaths.includes(path)) {
        return;
      }
      state.expandedPaths = state.expandedPaths.concat(path);
      state.selectedPath = path;
      state.isLoading = true;
      renderer.render(state);
      try {
        const review = await port.loadReview(state.review?.baseBranch, state.expandedPaths);
        state.review = review;
        state.selectedPath = path;
        state.isLoading = false;
      } catch (error) {
        state.error = error.message;
        state.isLoading = false;
      }
      renderer.render(state);
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
      renderer.render(state);
    },
    onCommentCancel: () => {
      state.activeCommentId = null;
      state.modalDraft = null;
      renderer.render(state);
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
      renderer.render(state);
    },
    onCommentDelete: () => {
      state.comments = state.comments.filter((comment) => comment.id !== state.activeCommentId);
      state.collapsedCommentIds = state.collapsedCommentIds.filter((id) => id !== state.activeCommentId);
      saveStoredComments(storage, activeScope(state.repoContext, state.review), state.comments);
      state.activeCommentId = null;
      state.modalDraft = null;
      renderer.render(state);
    },
    onCopyToClipboard: async () => {
      await exportComments(port, state, { onClipboardFallback: () => renderer.render(state) });
      renderer.render(state);
    },
    onClipboardFallbackClose: () => {
      state.clipboardFallbackText = '';
      renderer.render(state);
    },
    onRefresh: async () => {
      state.isLoading = true;
      state.error = null;
      renderer.render(state);

      try {
        await reloadShell(port, state, storage);
      } catch (error) {
        state.error = error.message;
        state.isLoading = false;
      }

      renderer.render(state);
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
      renderer.render(state);
    },
    onInlineCommentToggle: (commentId) => {
      if (state.collapsedCommentIds.includes(commentId)) {
        state.collapsedCommentIds = state.collapsedCommentIds.filter((id) => id !== commentId);
      } else {
        state.collapsedCommentIds = state.collapsedCommentIds.concat(commentId);
      }
      renderer.render(state);
    }
  });

  renderer.render(state);

  try {
    await loadShell(port, state, storage);
  } catch (error) {
    state.error = error.message;
    state.isLoading = false;
  }

  renderer.render(state);
  watchRefresh(window, port, state, () => renderer.render(state));
};

bootstrap();

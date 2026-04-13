import { commentIndex, staleComments } from '../core/review-comments.js';
import { escapeHtml } from '../core/file-tree.js';
import { renderBranches, renderTree } from './dom-renderer-helpers.js';

export const createShellRenderer = (documentRef, hooks) => {
  const nodes = selectNodes(documentRef);
  bindStaticEvents(nodes, hooks);

  return {
    render(state) {
      nodes.repoPath.textContent = state.repoContext?.repoPath || 'Loading repository...';
      nodes.errorBanner.textContent = state.error || '';
      nodes.errorBanner.classList.toggle('hidden', !state.error);
      renderBranches(nodes.baseBranch, state.repoContext, state.review);
      renderTree(nodes.fileTree, state, hooks);
      renderRefresh(nodes.refreshButton, state.pendingRefresh);
      renderCopyButton(nodes.copyButton, state.comments.length > 0);
      renderDiff(nodes.diffView, state, hooks);
      renderStaleComments(nodes.staleComments, state, hooks);
      renderCommentModal(nodes.commentModal, state, hooks);
      renderClipboardFallback(nodes.clipboardFallbackModal, state);
      nodes.loadingState.classList.toggle('hidden', !state.isLoading);
      nodes.emptyState.classList.toggle('hidden', state.isLoading || Boolean(state.review?.files.length));
      nodes.diffView.dataset.mode = state.diffMode;
    }
  };
};

const selectNodes = (documentRef) => ({
  repoPath: documentRef.querySelector('[data-testid="repo-path"]'),
  baseBranch: documentRef.querySelector('[data-testid="base-branch"]'),
  fileTree: documentRef.querySelector('[data-testid="file-tree"]'),
  diffView: documentRef.querySelector('[data-testid="diff-view"]'),
  loadingState: documentRef.querySelector('[data-testid="loading-state"]'),
  emptyState: documentRef.querySelector('[data-testid="empty-state"]'),
  errorBanner: documentRef.querySelector('[data-testid="error-banner"]'),
  unifiedButton: documentRef.querySelector('[data-testid="toggle-unified"]'),
  splitButton: documentRef.querySelector('[data-testid="toggle-split"]'),
  copyButton: documentRef.querySelector('[data-testid="copy-button"]'),
  refreshButton: documentRef.querySelector('[data-testid="refresh-button"]'),
  commentModal: documentRef.querySelector('[data-testid="comment-modal"]'),
  staleComments: documentRef.querySelector('[data-testid="stale-comments"]'),
  clipboardFallbackModal: documentRef.querySelector('[data-testid="clipboard-fallback-modal"]')
});

const bindStaticEvents = (nodes, hooks) => {
  nodes.baseBranch.addEventListener('change', () => hooks.onBaseBranchChange(nodes.baseBranch.value));
  nodes.unifiedButton.addEventListener('click', () => hooks.onDiffModeChange('unified'));
  nodes.splitButton.addEventListener('click', () => hooks.onDiffModeChange('split'));
  nodes.copyButton.addEventListener('click', () => hooks.onCopyToClipboard());
  nodes.refreshButton.addEventListener('click', () => hooks.onRefresh());
  nodes.commentModal.querySelector('[data-testid="comment-body"]').addEventListener('input', (event) => {
    hooks.onCommentChange(event.target.value);
  });
  nodes.commentModal.querySelector('[data-testid="comment-save"]').addEventListener('click', () => hooks.onCommentSave());
  nodes.commentModal.querySelector('[data-testid="comment-cancel"]').addEventListener('click', () => hooks.onCommentCancel());
  nodes.commentModal.querySelector('[data-testid="comment-delete"]').addEventListener('click', () => hooks.onCommentDelete());
  nodes.clipboardFallbackModal.querySelector('[data-testid="clipboard-fallback-close"]').addEventListener('click', () => hooks.onClipboardFallbackClose());
};

const renderRefresh = (button, pendingRefresh) => {
  button.classList.toggle('hidden', !pendingRefresh);
};

const renderCopyButton = (button, hasComments) => {
  button.disabled = !hasComments;
  button.removeAttribute('title');
};

const renderDiff = (node, state, hooks) => {
  const files = state.review?.files || [];
  const selected = files.filter((file) => !state.selectedPath || file.path === state.selectedPath);
  const comments = commentIndex(state.comments);
  node.innerHTML = selected.map((file) => renderFileCard(file, state.diffMode, comments, state.collapsedCommentIds)).join('')
    || '<div class="panel-state">Select a file to inspect its diff.</div>';
  attachLineEvents(node, hooks);
  attachLargeFileEvents(node, hooks);
  attachInlineCommentEvents(node, hooks);
};

const renderFileCard = (file, diffMode, comments, collapsedCommentIds) => `
  <article class="file-card">
    <header class="file-card-header">${escapeHtml(file.path)}</header>
    ${renderFileBody(file, diffMode, comments, collapsedCommentIds)}
  </article>
`;

const renderFileBody = (file, diffMode, comments, collapsedCommentIds) => {
  if (file.isBinary) {
    return '<div class="panel-state">Binary file changed.</div>';
  }

  if (file.isLarge && !file.isLoaded) {
    return `
      <div class="panel-state large-file-state">
        <strong>Large file</strong>
        <span>${file.lineCount} lines. Fetch on demand.</span>
        <button type="button" data-testid="large-file-fetch" data-path="${escapeHtml(file.path)}">Fetch now</button>
      </div>
    `;
  }

  return file.hunks.map((hunk) => renderHunk(file, hunk, diffMode, comments, collapsedCommentIds)).join('');
};

const renderHunk = (file, hunk, diffMode, comments, collapsedCommentIds) => `
  <table class="diff-table"><tbody>
    ${hunk.lines.map((line) => renderLine(file, line, diffMode, comments, collapsedCommentIds)).join('')}
  </tbody></table>
`;

const renderLine = (file, line, diffMode, comments, collapsedCommentIds) => {
  const primary = line.newLineNumber == null
    ? { side: 'base', number: line.oldLineNumber, path: file.oldPath || file.path }
    : { side: 'head', number: line.newLineNumber, path: file.newPath || file.path };
  const comment = primary.number == null ? null : comments.get(`${primary.path}::${primary.side}::${primary.number}`);
  const inlineComment = comment ? renderInlineComment(comment, diffMode, collapsedCommentIds.includes(comment.id)) : '';
  const marker = comment ? '<button type="button" class="comment-marker" data-testid="comment-marker">1</button>' : '';
  const attributes = primary.number == null
    ? ''
    : ` data-line-key="${primary.path}::${primary.side}::${primary.number}" data-path="${primary.path}"`;

  if (diffMode === 'split') {
    return `
      <tr class="diff-line diff-line-${line.kind}" data-testid="diff-line" data-kind="${line.kind}"${attributes}>
        <td class="line-number">${line.oldLineNumber ?? ''}</td>
        <td>${line.kind === 'added' ? '' : escapeHtml(line.text)}</td>
        <td class="line-number">${line.newLineNumber ?? ''}</td>
        <td>${line.kind === 'removed' ? '' : escapeHtml(line.text)} ${marker}</td>
      </tr>
      ${inlineComment}
    `;
  }

  return `
    <tr class="diff-line diff-line-${line.kind}" data-testid="diff-line" data-kind="${line.kind}"${attributes}>
      <td class="line-number">${line.oldLineNumber ?? ''}</td>
      <td class="line-number">${line.newLineNumber ?? ''}</td>
      <td>${escapeHtml(line.text)} ${marker}</td>
    </tr>
    ${inlineComment}
  `;
};

const renderInlineComment = (comment, diffMode, collapsed) => {
  const statusLabel = comment.status === 'active' ? '' : renderStatusLabel(comment.status);

  return `
  <tr class="inline-comment-row" data-testid="inline-comment" data-comment-id="${comment.id}">
    <td class="line-number"></td>
    <td data-testid="inline-comment-cell" colspan="${diffMode === 'split' ? '3' : '2'}">
      <div class="inline-comment-card">
        <div class="inline-comment-header" data-testid="inline-comment-header">
          <button type="button" class="inline-comment-toggle" data-testid="inline-comment-toggle" data-comment-id="${comment.id}">
            ${collapsed ? '▸' : '▾'}
          </button>
          <span>Comment on line ${formatLineLabel(comment)}</span>
          ${statusLabel ? `<span>${statusLabel}</span>` : ''}
        </div>
        <div class="inline-comment-thread${collapsed ? ' hidden' : ''}">
          <div class="inline-comment-body" data-testid="inline-comment-body">${escapeHtml(comment.body)}</div>
        </div>
      </div>
    </td>
  </tr>
`;
};

const attachLineEvents = (node, hooks) => {
  node.querySelectorAll('[data-testid="diff-line"]').forEach((line) => {
    line.addEventListener('mouseenter', () => line.classList.add('is-hovered'));
    line.addEventListener('mouseleave', () => line.classList.remove('is-hovered'));
    if (line.dataset.lineKey) {
      line.addEventListener('click', () => hooks.onLineSelect({ lineKey: line.dataset.lineKey, path: line.dataset.path }));
    }
  });
};

const attachLargeFileEvents = (node, hooks) => {
  node.querySelectorAll('[data-testid="large-file-fetch"]').forEach((button) => {
    button.addEventListener('click', () => hooks.onLargeFileFetch(button.dataset.path));
  });
};

const attachInlineCommentEvents = (node, hooks) => {
  node.querySelectorAll('[data-testid="inline-comment-toggle"]').forEach((button) => {
    button.addEventListener('click', (event) => {
      event.stopPropagation();
      hooks.onInlineCommentToggle(button.dataset.commentId);
    });
  });
};

const renderStaleComments = (node, state, hooks) => {
  const comments = staleComments(state.comments);
  node.innerHTML = comments.map((comment) => `
    <button type="button" data-testid="stale-comment-trigger" data-comment-id="${comment.id}">
      ${escapeHtml(comment.body)}
    </button>
  `).join('');
  node.classList.toggle('hidden', comments.length === 0);
  node.querySelectorAll('[data-comment-id]').forEach((button) => {
    button.addEventListener('click', () => hooks.onStaleCommentSelect(button.dataset.commentId));
  });
};

const renderCommentModal = (node, state) => {
  const isOpen = Boolean(state.modalDraft);
  node.classList.toggle('hidden', !isOpen);
  if (!isOpen) {
    return;
  }
  const comment = state.comments.find((entry) => entry.id === state.activeCommentId);
  const isStale = (comment?.status || state.modalDraft.status) === 'stale';
  node.querySelector('[data-testid="comment-path"]').textContent = state.modalDraft.path;
  node.querySelector('[data-testid="comment-body"]').value = state.modalDraft.body;
  node.querySelector('[data-testid="comment-status"]').textContent = labelForStatus(comment?.status || state.modalDraft.status);
  node.querySelector('[data-testid="comment-delete"]').classList.toggle('hidden', !comment);
  node.querySelector('[data-testid="comment-save"]').classList.toggle('hidden', isStale);
  node.querySelector('[data-testid="comment-body"]').readOnly = isStale;
};

const renderClipboardFallback = (node, state) => {
  const isOpen = Boolean(state.clipboardFallbackText);
  node.classList.toggle('hidden', !isOpen);
  const textarea = node.querySelector('[data-testid="clipboard-fallback-text"]');
  textarea.value = state.clipboardFallbackText;
  textarea.textContent = state.clipboardFallbackText;
};

const labelForStatus = (status) => {
  if (status === 'changed') {
    return 'Changed';
  }
  if (status === 'stale') {
    return 'Stale';
  }
  return '';
};

const renderStatusLabel = (status) => labelForStatus(status) || 'Active';

const formatLineLabel = (comment) => `${comment.mapping.side === 'base' ? 'L' : 'R'}${comment.mapping.lineNumber}`;

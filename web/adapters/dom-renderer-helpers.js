import { buildFileTree, escapeHtml } from '../core/file-tree.js';

export const renderBranches = (select, repoContext, review) => {
  const branches = repoContext?.remoteBranches || [];
  const selected = review?.baseBranch || repoContext?.defaultBaseBranch || '';
  select.innerHTML = branches
    .map((branch) => `<option value="${escapeHtml(branch)}">${escapeHtml(branch)}</option>`)
    .join('');
  select.value = selected;
};

export const renderReviewMode = (select, state) => {
  select.value = state.reviewMode || 'branch';
};

export const renderCommitSelect = (wrapper, select, state) => {
  const commits = state.availableCommits || [];
  wrapper.classList.toggle('hidden', state.reviewMode !== 'commit');
  select.innerHTML = commits
    .map((commit) => `<option value="${escapeHtml(commit.sha)}">${escapeHtml(commitLabel(commit))}</option>`)
    .join('');
  select.value = state.selectedCommit || commits[0]?.sha || '';
};

const commitLabel = (commit) => {
  if (commit.isLocalChanges) {
    return commit.subject;
  }

  return `${commit.shortSha} ${commit.subject}`;
};

export const renderTree = (node, state, hooks) => {
  const files = state.review?.files || [];
  node.innerHTML = '';
  const tree = buildFileTree(files.map((file) => file.path));
  const commentCounts = countCommentsByPath(state.comments);
  tree.forEach((entry) => node.appendChild(renderTreeEntry(node.ownerDocument, entry, state.selectedPath, hooks, commentCounts)));
};

const renderTreeEntry = (documentRef, entry, selectedPath, hooks, commentCounts) => {
  if (entry.type === 'directory') {
    const wrapper = documentRef.createElement('div');
    const label = documentRef.createElement('div');
    const group = documentRef.createElement('div');

    label.className = 'file-tree-node';
    label.textContent = entry.name;
    group.className = 'file-tree-group';
    entry.children.forEach((child) => {
      group.appendChild(renderTreeEntry(documentRef, child, selectedPath, hooks, commentCounts));
    });
    wrapper.append(label, group);
    return wrapper;
  }

  const button = documentRef.createElement('button');
  const name = documentRef.createElement('span');
  button.type = 'button';
  button.className = `file-tree-file${selectedPath === entry.path ? ' is-active' : ''}`;
  name.textContent = entry.name;
  button.appendChild(name);
  const count = commentCounts.get(entry.path) || 0;
  if (count > 0) {
    const badge = documentRef.createElement('span');
    badge.className = 'file-comment-count';
    badge.dataset.testid = 'file-comment-count';
    badge.textContent = String(count);
    button.appendChild(badge);
  }
  button.addEventListener('click', () => hooks.onFileSelect(entry.path));
  return button;
};

const countCommentsByPath = (comments) => comments.reduce((counts, comment) => {
  const path = comment.mapping.path;
  counts.set(path, (counts.get(path) || 0) + 1);
  return counts;
}, new Map());

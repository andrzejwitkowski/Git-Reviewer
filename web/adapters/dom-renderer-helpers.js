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
  const viewedPaths = new Set(state.viewedPaths);
  const collapsedDirectories = new Set(state.collapsedDirectories);
  tree.forEach((entry) => node.appendChild(renderTreeEntry(
    node.ownerDocument,
    entry,
    state.selectedPath,
    hooks,
    commentCounts,
    viewedPaths,
    collapsedDirectories
  )));
};

const renderTreeEntry = (documentRef, entry, selectedPath, hooks, commentCounts, viewedPaths, collapsedDirectories) => {
  if (entry.type === 'directory') {
    const wrapper = documentRef.createElement('div');
    const label = documentRef.createElement('div');
    const toggle = documentRef.createElement('button');
    const name = documentRef.createElement('span');
    const group = documentRef.createElement('div');
    const isCollapsed = collapsedDirectories.has(entry.path);

    label.className = 'file-tree-node';
    toggle.type = 'button';
    toggle.className = 'file-tree-toggle';
    toggle.dataset.testid = 'tree-toggle';
    toggle.setAttribute('aria-expanded', String(!isCollapsed));
    toggle.textContent = isCollapsed ? '+' : '-';
    name.textContent = entry.name;
    label.append(toggle, name);
    group.className = 'file-tree-group';
    group.classList.toggle('hidden', isCollapsed);
    entry.children.forEach((child) => {
      group.appendChild(renderTreeEntry(documentRef, child, selectedPath, hooks, commentCounts, viewedPaths, collapsedDirectories));
    });
    toggle.addEventListener('click', () => hooks.onDirectoryToggle(entry.path));
    wrapper.append(label, group);
    return wrapper;
  }

  const button = documentRef.createElement('button');
  const leading = documentRef.createElement('span');
  const name = documentRef.createElement('span');
  const trailing = documentRef.createElement('span');
  button.type = 'button';
  button.className = `file-tree-file${selectedPath === entry.path ? ' is-active' : ''}`;
  leading.className = 'file-tree-leading';
  trailing.className = 'file-tree-trailing';
  if (viewedPaths.has(entry.path)) {
    const viewedBadge = documentRef.createElement('span');
    viewedBadge.className = 'file-viewed-badge';
    viewedBadge.dataset.testid = 'file-viewed-badge';
    viewedBadge.textContent = '✓';
    leading.appendChild(viewedBadge);
  }
  name.textContent = entry.name;
  leading.appendChild(name);
  button.appendChild(leading);
  const count = commentCounts.get(entry.path) || 0;
  if (count > 0) {
    const badge = documentRef.createElement('span');
    badge.className = 'file-comment-count';
    badge.dataset.testid = 'file-comment-count';
    badge.textContent = String(count);
    trailing.appendChild(badge);
  }
  button.appendChild(trailing);
  button.addEventListener('click', () => hooks.onFileSelect(entry.path));
  return button;
};

const countCommentsByPath = (comments) => comments.reduce((counts, comment) => {
  const path = comment.mapping.path;
  counts.set(path, (counts.get(path) || 0) + 1);
  return counts;
}, new Map());

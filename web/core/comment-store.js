const STORAGE_KEY = 'git-reviewer-comments-v1';

export const loadStoredComments = (storage, scope) => {
  const allComments = readStorage(storage);
  return normalizeComments(allComments[scopeKey(scope)] || []);
};

export const saveStoredComments = (storage, scope, comments) => {
  const allComments = readStorage(storage);
  allComments[scopeKey(scope)] = comments;
  storage.setItem(STORAGE_KEY, JSON.stringify(allComments));
};

export const moveStoredComments = (storage, fromScope, toScope, comments) => {
  const allComments = readStorage(storage);
  delete allComments[scopeKey(fromScope)];
  allComments[scopeKey(toScope)] = comments;
  storage.setItem(STORAGE_KEY, JSON.stringify(allComments));
};

export const activeScope = (repoContext, review) => ({
  repoPath: repoContext?.repoPath || '',
  baseBranch: review?.baseBranch || '',
  headSha: review?.headSha || ''
});

const scopeKey = (scope) => `${scope.repoPath}::${scope.baseBranch}::${scope.headSha}`;

const readStorage = (storage) => {
  try {
    return JSON.parse(storage.getItem(STORAGE_KEY) || '{}');
  } catch {
    return {};
  }
};

const normalizeComments = (comments) => comments.map((comment) => ({
  ...comment,
  status: comment.status || 'active',
  anchor: {
    ...comment.anchor,
    oldPath: comment.anchor.oldPath || null,
    newPath: comment.anchor.newPath || null
  },
  mapping: comment.mapping || buildMapping(comment.anchor)
}));

export const buildMapping = (anchor) => ({
  path: anchor.side === 'base' ? (anchor.oldPath || anchor.newPath) : (anchor.newPath || anchor.oldPath),
  side: anchor.side,
  lineNumber: anchor.lineNumber,
  text: anchor.text
});

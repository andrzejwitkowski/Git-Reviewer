const lineKey = (path, side, lineNumber) => `${path}::${side}::${lineNumber}`;

export const buildLineCatalog = (review) => {
  const byKey = new Map();
  const entries = [];

  for (const file of review?.files || []) {
    for (const hunk of file.hunks) {
      for (const line of hunk.lines) {
        for (const anchor of lineAnchors(file, line)) {
          const key = lineKey(anchor.path, anchor.side, anchor.lineNumber);
          const value = {
            ...anchor,
            key,
            oldPath: file.oldPath || null,
            newPath: file.newPath || null
          };
          byKey.set(key, value);
          entries.push(value);
        }
      }
    }
  }

  return { byKey, entries };
};

export const createCommentDraft = (review, lineRecord, body, existingId) => ({
  id: existingId || crypto.randomUUID(),
  body,
  status: 'active',
  sourceHeadSha: review.headSha,
  anchor: {
    baseSha: review.mergeBaseSha,
    oldPath: lineRecord.oldPath,
    newPath: lineRecord.newPath,
    side: lineRecord.side,
    lineNumber: lineRecord.lineNumber,
    text: lineRecord.text
  },
  mapping: {
    path: lineRecord.path,
    side: lineRecord.side,
    lineNumber: lineRecord.lineNumber,
    text: lineRecord.text
  }
});

export const commentIndex = (comments) => {
  const index = new Map();
  for (const comment of comments) {
    if (comment.status === 'stale') {
      continue;
    }
    index.set(lineKey(comment.mapping.path, comment.mapping.side, comment.mapping.lineNumber), comment);
  }
  return index;
};

export const remapComments = (comments, nextReview) => {
  const catalog = buildLineCatalog(nextReview);
  return comments.map((comment) => remapComment(comment, catalog, nextReview));
};

export const staleComments = (comments) => comments.filter((comment) => comment.status === 'stale');

const remapComment = (comment, catalog, nextReview) => {
  const directKey = lineKey(comment.mapping.path, comment.mapping.side, comment.mapping.lineNumber);
  const directMatch = catalog.byKey.get(directKey);

  if (directMatch) {
    return hydrateComment(comment, directMatch, nextReview, directMatch.text === comment.mapping.text ? 'active' : 'changed');
  }

  const candidates = catalog.entries.filter((entry) => {
    if (entry.side !== comment.mapping.side) {
      return false;
    }
    if (!matchesCommentPath(comment, entry)) {
      return false;
    }
    if (entry.text !== comment.mapping.text) {
      return false;
    }
    return Math.abs(entry.lineNumber - comment.mapping.lineNumber) <= 3;
  });

  if (candidates.length === 1) {
    return hydrateComment(comment, candidates[0], nextReview, 'active');
  }

  return {
    ...comment,
    status: 'stale',
    sourceHeadSha: nextReview.headSha
  };
};

const matchesCommentPath = (comment, entry) => {
  if (entry.path === comment.mapping.path) {
    return true;
  }

  if (comment.mapping.side === 'head') {
    return entry.oldPath === comment.anchor.oldPath || entry.newPath === comment.anchor.newPath;
  }

  return entry.oldPath === comment.anchor.oldPath;
};

const hydrateComment = (comment, lineRecord, nextReview, status) => ({
  ...comment,
  status,
  sourceHeadSha: nextReview.headSha,
  anchor: {
    baseSha: nextReview.mergeBaseSha,
    oldPath: lineRecord.oldPath,
    newPath: lineRecord.newPath,
    side: lineRecord.side,
    lineNumber: lineRecord.lineNumber,
    text: lineRecord.text
  },
  mapping: {
    path: lineRecord.path,
    side: lineRecord.side,
    lineNumber: lineRecord.lineNumber,
    text: lineRecord.text
  }
});

const lineAnchors = (file, line) => {
  const anchors = [];
  if (line.oldLineNumber != null) {
    anchors.push({
      path: file.oldPath || file.path,
      side: 'base',
      lineNumber: line.oldLineNumber,
      text: line.text
    });
  }
  if (line.newLineNumber != null) {
    anchors.push({
      path: file.newPath || file.path,
      side: 'head',
      lineNumber: line.newLineNumber,
      text: line.text
    });
  }
  return anchors;
};

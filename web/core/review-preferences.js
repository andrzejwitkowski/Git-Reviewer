const STORAGE_KEY = 'git-reviewer-preferences-v1';

export const loadStoredReviewPreferences = (storage, scope) => {
  const allPreferences = readStorage(storage);
  const preferences = allPreferences[scopeKey(scope)] || {};

  return {
    viewedPaths: normalizePathList(preferences.viewedPaths || [])
  };
};

export const saveStoredReviewPreferences = (storage, scope, preferences) => {
  const allPreferences = readStorage(storage);
  allPreferences[scopeKey(scope)] = {
    viewedPaths: normalizePathList(preferences.viewedPaths || [])
  };
  storage.setItem(STORAGE_KEY, JSON.stringify(allPreferences));
};

export const setViewedPath = (viewedPaths, path, viewed) => {
  const nextPaths = viewed
    ? viewedPaths.concat(path)
    : viewedPaths.filter((entry) => entry !== path);

  return normalizePathList(nextPaths);
};

const scopeKey = (scope) => `${scope.repoPath}::${scope.reviewMode || 'branch'}::${scope.baseBranch}::${scope.selectedCommit || ''}::${scope.headSha}`;

const readStorage = (storage) => {
  try {
    return JSON.parse(storage.getItem(STORAGE_KEY) || '{}');
  } catch {
    return {};
  }
};

const normalizePathList = (paths) => Array.from(new Set(paths)).sort();

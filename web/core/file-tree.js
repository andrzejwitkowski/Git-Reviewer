export const buildFileTree = (paths) => {
  const root = [];

  paths.sort().forEach((path) => {
    const parts = path.split('/');
    let level = root;
    let currentPath = '';

    parts.forEach((part, index) => {
      currentPath = currentPath ? `${currentPath}/${part}` : part;
      const isFile = index === parts.length - 1;
      let existing = level.find(
        (entry) => entry.name === part && entry.type === (isFile ? 'file' : 'directory')
      );

      if (!existing) {
        existing = isFile
          ? { type: 'file', name: part, path: currentPath }
          : { type: 'directory', name: part, path: currentPath, children: [] };
        level.push(existing);
      }

      if (!isFile) {
        level = existing.children;
      }
    });
  });

  return root;
};

export const escapeHtml = (value) => value
  .replaceAll('&', '&amp;')
  .replaceAll('<', '&lt;')
  .replaceAll('>', '&gt;')
  .replaceAll('"', '&quot;');

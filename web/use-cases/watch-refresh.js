export const watchRefresh = (windowRef, port, state, onChange) => {
  let disposed = false;

  const check = async () => {
    if (disposed || !state.review) {
      return;
    }

    try {
      const snapshot = await port.loadRepoStatus();
      state.pendingRefresh = hasSnapshotChanged(state.repoSnapshot, snapshot);
      state.nextSnapshot = snapshot;
      onChange();
    } catch {
      // ignore background polling failures
    }
  };

  const intervalId = windowRef.setInterval(check, 5000);
  const focusListener = () => {
    void check();
  };

  windowRef.addEventListener('focus', focusListener);

  return () => {
    disposed = true;
    windowRef.clearInterval(intervalId);
    windowRef.removeEventListener('focus', focusListener);
  };
};

const hasSnapshotChanged = (current, next) => JSON.stringify(current) !== JSON.stringify(next);

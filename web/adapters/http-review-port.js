import { requestJson, withQuery } from '../ports/review-port.js';

export const createHttpReviewPort = () => ({
  async loadRepoContext() {
    return requestJson('/api/repo-context');
  },
  async loadCommits(base) {
    return requestJson(withQuery('/api/commits', [
      ['base', base]
    ]));
  },
  async loadReview({ mode = 'branch', base, commit = null, expandedPaths = [] }) {
    const entries = [
      ['base', base],
      ['expand', expandedPaths]
    ];
    if (mode === 'commit') {
      entries.unshift(['commit', commit]);
      entries.unshift(['mode', mode]);
    }
    return requestJson(withQuery('/api/review', [
      ...entries
    ]));
  },
  async loadRepoStatus() {
    return requestJson('/api/status');
  },
  async exportClipboard(headSha, comments) {
    const response = await requestJson('/api/clipboard-export', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({ headSha, comments })
    });
    return response.text;
  }
});

import { requestJson, withQuery } from '../ports/review-port.js';

export const createHttpReviewPort = () => ({
  async loadRepoContext() {
    return requestJson('/api/repo-context');
  },
  async loadReview(base, expandedPaths = []) {
    return requestJson(withQuery('/api/review', [
      ['base', base],
      ['expand', expandedPaths]
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

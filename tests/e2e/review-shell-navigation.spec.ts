import { expect, test } from '@playwright/test';
import { createLargeFileFixture, createRepoFixture, startServer, stopServer } from './support/review-shell-fixture';

test('renders review shell and supports basic navigation', async ({ page }) => {
  const repo = createRepoFixture();
  const server = await startServer(repo.path);

  try {
    await page.goto(server.url);

    await expect(page.getByTestId('repo-path')).toContainText(repo.path);
    await expect(page.getByTestId('base-branch')).toHaveValue('origin/main');
    await expect(page.getByTestId('copy-button')).toBeDisabled();
    await expect(page.getByTestId('file-tree')).toContainText('src');
    await expect(page.getByTestId('file-tree')).toContainText('docs');
    await expect(page.getByRole('button', { name: 'lib.rs' })).toBeVisible();
    await expect(page.getByRole('button', { name: 'readme.md' })).toBeVisible();
    await expect(page.getByRole('button', { name: 'src/lib.rs' })).toHaveCount(0);
    await expect(page.getByRole('button', { name: 'docs/readme.md' })).toHaveCount(0);
    await expect(page.getByTestId('diff-view')).toContainText('feature line second');

    await page.getByRole('button', { name: 'Split' }).click();
    await expect(page.getByTestId('diff-view')).toHaveAttribute('data-mode', 'split');

    await page.getByRole('button', { name: 'Unified' }).click();
    await expect(page.getByTestId('diff-view')).toHaveAttribute('data-mode', 'unified');

    await page.getByRole('button', { name: 'readme.md' }).click();
    await expect(page.getByTestId('diff-view')).toContainText('changed docs');
  } finally {
    stopServer(server.process);
    repo.cleanup();
  }
});

test('keeps the latest base-branch selection when earlier responses finish later', async ({ page }) => {
  const repo = createRepoFixture();
  const server = await startServer(repo.path);
  let resolveReleaseReview;
  const releaseReviewSettled = new Promise((resolve) => {
    resolveReleaseReview = resolve;
  });

  await page.route(/\/api\/review\?base=origin(?:%2F|\/)release$/, async (route) => {
    try {
      await new Promise((resolve) => setTimeout(resolve, 600));
      await route.fulfill({ response: await route.fetch() });
    } finally {
      resolveReleaseReview();
    }
  });

  try {
    await page.goto(server.url);
    await expect(page.getByTestId('base-branch')).toHaveValue('origin/main');

    await page.getByTestId('base-branch').selectOption('origin/release');
    await page.getByTestId('base-branch').selectOption('origin/main');

    await releaseReviewSettled;

    await expect(page.getByTestId('base-branch')).toHaveValue('origin/main');
    await expect(page.getByTestId('diff-view')).toContainText('feature line second');
    await expect(page.getByTestId('diff-view')).not.toContainText('main baseline');
  } finally {
    stopServer(server.process);
    repo.cleanup();
  }
});

test('ignores delayed failures from older base-branch requests after a newer success', async ({ page }) => {
  const repo = createRepoFixture();
  const server = await startServer(repo.path);
  let resolveReleaseReview;
  const releaseReviewSettled = new Promise((resolve) => {
    resolveReleaseReview = resolve;
  });

  await page.route(/\/api\/review\?base=origin(?:%2F|\/)release$/, async (route) => {
    try {
      await new Promise((resolve) => setTimeout(resolve, 600));
      await route.fulfill({
        status: 500,
        contentType: 'application/json',
        body: JSON.stringify({ error: 'release branch failed' })
      });
    } finally {
      resolveReleaseReview();
    }
  });

  try {
    await page.goto(server.url);
    await expect(page.getByTestId('base-branch')).toHaveValue('origin/main');

    await page.getByTestId('base-branch').selectOption('origin/release');
    await page.getByTestId('base-branch').selectOption('origin/main');

    await expect(page.getByTestId('base-branch')).toHaveValue('origin/main');
    await expect(page.getByTestId('diff-view')).toContainText('feature line second');
    await expect(page.getByTestId('error-banner')).toBeHidden();

    await releaseReviewSettled;

    await expect(page.getByTestId('base-branch')).toHaveValue('origin/main');
    await expect(page.getByTestId('diff-view')).toContainText('feature line second');
    await expect(page.getByTestId('error-banner')).toBeHidden();
    await expect(page.getByTestId('error-banner')).not.toContainText('release branch failed');
  } finally {
    stopServer(server.process);
    repo.cleanup();
  }
});

test('keeps branch review usable when commit-list loading fails', async ({ page }) => {
  const repo = createRepoFixture();
  const server = await startServer(repo.path);

  await page.route(/\/api\/commits\?base=origin(?:%2F|\/)main$/, async (route) => {
    await route.fulfill({
      status: 500,
      contentType: 'application/json',
      body: JSON.stringify({ error: 'commit list unavailable' })
    });
  });

  try {
    await page.goto(server.url);

    await expect(page.getByTestId('base-branch')).toHaveValue('origin/main');
    await expect(page.getByTestId('diff-view')).toContainText('feature line second');
    await expect(page.getByTestId('error-banner')).toBeHidden();
    await expect(page.getByTestId('commit-select')).toBeHidden();
  } finally {
    stopServer(server.process);
    repo.cleanup();
  }
});

test('ignores delayed commit lists from an older base selection', async ({ page }) => {
  const repo = createRepoFixture();
  const server = await startServer(repo.path);
  let resolveReleaseCommits;
  const releaseCommitsSettled = new Promise((resolve) => {
    resolveReleaseCommits = resolve;
  });

  await page.route(/\/api\/commits\?base=origin(?:%2F|\/)release$/, async (route) => {
    try {
      await new Promise((resolve) => setTimeout(resolve, 600));
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: '[]'
      });
    } finally {
      resolveReleaseCommits();
    }
  });

  try {
    await page.goto(server.url);
    await expect(page.getByTestId('diff-view')).toContainText('feature line second');
    await page.getByTestId('review-mode').selectOption('commit');
    await expect(page.getByTestId('commit-select')).toContainText('second feature');

    const secondCommit = page.getByTestId('commit-select').locator('option').filter({ hasText: 'second feature' });
    await page.getByTestId('commit-select').selectOption(await secondCommit.getAttribute('value'));
    await expect(page.getByTestId('diff-view')).toContainText('feature line second');

    await page.getByTestId('base-branch').selectOption('origin/release');
    await page.getByTestId('base-branch').selectOption('origin/main');

    await expect(page.getByTestId('base-branch')).toHaveValue('origin/main');
    await expect(page.getByTestId('commit-select')).toContainText('second feature');
    await expect(page.getByTestId('commit-select')).toHaveValue('LOCAL_CHANGES');
    await expect(page.getByTestId('empty-state')).toContainText('No changed files.');

    await releaseCommitsSettled;

    await expect(page.getByTestId('base-branch')).toHaveValue('origin/main');
    await expect(page.getByTestId('commit-select')).toContainText('second feature');
    await expect(page.getByTestId('commit-select')).toHaveValue('LOCAL_CHANGES');
    await expect(page.getByTestId('empty-state')).toContainText('No changed files.');
  } finally {
    stopServer(server.process);
    repo.cleanup();
  }
});

test('keeps the previous branch review when switching to commit mode fails', async ({ page }) => {
  const repo = createRepoFixture();
  const server = await startServer(repo.path);

  await page.route(/\/api\/commits\?base=origin(?:%2F|\/)main$/, async (route) => {
    await route.fulfill({
      status: 500,
      contentType: 'application/json',
      body: JSON.stringify({ error: 'commit list unavailable' })
    });
  });

  try {
    await page.goto(server.url);

    await expect(page.getByTestId('review-mode')).toHaveValue('branch');
    await expect(page.getByTestId('diff-view')).toContainText('feature line second');

    await page.getByTestId('review-mode').selectOption('commit');

    await expect(page.getByTestId('error-banner')).toContainText('commit list unavailable');
    await expect(page.getByTestId('review-mode')).toHaveValue('branch');
    await expect(page.getByTestId('commit-select')).toBeHidden();
    await expect(page.getByTestId('diff-view')).toContainText('feature line second');
    await expect(page.getByTestId('file-tree')).toContainText('readme.md');
  } finally {
    stopServer(server.process);
    repo.cleanup();
  }
});

test('shows large-file fetch gate before loading a very large diff', async ({ page }) => {
  const repo = createLargeFileFixture();
  const server = await startServer(repo.path);

  try {
    await page.goto(server.url);

    await page.getByRole('button', { name: 'huge.ts' }).click();
    await expect(page.getByText('Large file')).toBeVisible();
    await expect(page.getByRole('button', { name: 'Fetch now' })).toBeVisible();
    await expect(page.getByTestId('diff-view')).not.toContainText('updated marker');

    await page.getByRole('button', { name: 'Fetch now' }).click();
    await expect(page.getByTestId('diff-view')).toContainText('updated marker');
  } finally {
    stopServer(server.process);
    repo.cleanup();
  }
});

test('switches to single-commit review and shows only the selected commit diff', async ({ page }) => {
  const repo = createRepoFixture();
  const server = await startServer(repo.path);

  try {
    await page.goto(server.url);

    await expect(page.getByTestId('review-mode')).toHaveValue('branch');
    await expect(page.getByTestId('commit-select')).toBeHidden();
    await expect(page.getByTestId('diff-view')).toContainText('feature line second');

    await page.getByTestId('review-mode').selectOption('commit');

    await expect(page.getByTestId('commit-select')).toBeVisible();
    await expect(page.getByTestId('commit-select').locator('option').first()).toContainText('LOCAL CHANGES');
    await expect(page.getByTestId('commit-select')).toHaveValue('LOCAL_CHANGES');
    await expect(page.getByTestId('empty-state')).toContainText('No changed files.');

    const secondCommit = page.getByTestId('commit-select').locator('option').filter({ hasText: 'second feature' });
    await page.getByTestId('commit-select').selectOption(await secondCommit.getAttribute('value'));

    await expect(page.getByTestId('commit-select')).toContainText('second feature');
    await expect(page.getByTestId('commit-select')).toContainText('first feature');
    await expect(page.getByTestId('diff-view')).toContainText('feature line second');
    await expect(page.getByTestId('diff-view')).not.toContainText('changed docs');
  } finally {
    stopServer(server.process);
    repo.cleanup();
  }
});

test('shows local worktree diff when LOCAL CHANGES is selected in commit mode', async ({ page }) => {
  const repo = createRepoFixture();
  repo.writeFile('src/lib.rs', 'line 1\nline 2\nline 3\nline 4\nline 5\nline 6\nline 7\nline 8\nline 9\nline 10\nline 11\nlocal worktree change\nline 13\n');
  const server = await startServer(repo.path);

  try {
    await page.goto(server.url);
    await expect(page.getByTestId('repo-path')).toContainText(repo.path);
    await expect(page.getByTestId('diff-view')).toContainText('local worktree change');

    await page.getByTestId('review-mode').selectOption('commit');

    await expect(page.getByTestId('commit-select')).toHaveValue('LOCAL_CHANGES');
    await expect(page.getByTestId('diff-view')).toContainText('local worktree change');
  } finally {
    stopServer(server.process);
    repo.cleanup();
  }
});

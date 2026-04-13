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
    await expect(page.getByTestId('diff-view')).toContainText('feature line');

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

  await page.route(/\/api\/review\?base=origin(?:%2F|\/)release$/, async (route) => {
    await new Promise((resolve) => setTimeout(resolve, 600));
    await route.fulfill({ response: await route.fetch() });
  });

  try {
    await page.goto(server.url);
    await expect(page.getByTestId('base-branch')).toHaveValue('origin/main');

    await page.getByTestId('base-branch').selectOption('origin/release');
    await page.getByTestId('base-branch').selectOption('origin/main');

    await page.waitForTimeout(800);

    await expect(page.getByTestId('base-branch')).toHaveValue('origin/main');
    await expect(page.getByTestId('diff-view')).toContainText('feature line');
    await expect(page.getByTestId('diff-view')).not.toContainText('main baseline');
  } finally {
    stopServer(server.process);
    repo.cleanup();
  }
});

test('ignores delayed failures from older base-branch requests after a newer success', async ({ page }) => {
  const repo = createRepoFixture();
  const server = await startServer(repo.path);

  await page.route(/\/api\/review\?base=origin(?:%2F|\/)release$/, async (route) => {
    await new Promise((resolve) => setTimeout(resolve, 600));
    await route.fulfill({
      status: 500,
      contentType: 'application/json',
      body: JSON.stringify({ error: 'release branch failed' })
    });
  });

  try {
    await page.goto(server.url);
    await expect(page.getByTestId('base-branch')).toHaveValue('origin/main');

    await page.getByTestId('base-branch').selectOption('origin/release');
    await page.getByTestId('base-branch').selectOption('origin/main');

    await expect(page.getByTestId('base-branch')).toHaveValue('origin/main');
    await expect(page.getByTestId('diff-view')).toContainText('feature line');
    await expect(page.getByTestId('error-banner')).toBeHidden();

    await page.waitForTimeout(800);

    await expect(page.getByTestId('base-branch')).toHaveValue('origin/main');
    await expect(page.getByTestId('diff-view')).toContainText('feature line');
    await expect(page.getByTestId('error-banner')).toBeHidden();
    await expect(page.getByTestId('error-banner')).not.toContainText('release branch failed');
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

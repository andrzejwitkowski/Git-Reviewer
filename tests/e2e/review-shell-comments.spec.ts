import { expect, test } from '@playwright/test';
import { createRepoFixture, sourceLines, startServer, stopServer } from './support/review-shell-fixture';

test('supports add edit delete and cancel in the comment modal', async ({ page }) => {
  const repo = createRepoFixture();
  const server = await startServer(repo.path);

  try {
    await page.goto(server.url);

    const targetLine = page.getByTestId('diff-line').filter({ hasText: 'feature line' });
    await targetLine.click();
    await expect(page.getByTestId('comment-modal')).toBeVisible();

    await page.getByTestId('comment-body').fill('line note');
    await page.getByRole('button', { name: 'Save comment' }).click();

    await expect(targetLine.getByTestId('comment-marker')).toContainText('1');

    await targetLine.click();
    await expect(page.getByTestId('comment-body')).toHaveValue('line note');
    await page.getByTestId('comment-body').fill('line note edited');
    await page.getByRole('button', { name: 'Cancel' }).click();

    await targetLine.click();
    await expect(page.getByTestId('comment-body')).toHaveValue('line note');
    await page.getByTestId('comment-body').fill('line note edited');
    await page.getByRole('button', { name: 'Save comment' }).click();

    await targetLine.click();
    await expect(page.getByTestId('comment-body')).toHaveValue('line note edited');
    await page.getByRole('button', { name: 'Delete comment' }).click();

    await expect(targetLine.getByTestId('comment-marker')).toBeHidden();
  } finally {
    stopServer(server.process);
    repo.cleanup();
  }
});

test('renders persistent inline comments directly under commented lines', async ({ page }) => {
  const repo = createRepoFixture();
  const server = await startServer(repo.path);

  try {
    await page.goto(server.url);

    const featureLine = page.getByTestId('diff-line').filter({ hasText: 'feature line' });
    await featureLine.click();
    await page.getByTestId('comment-body').fill('src comment that stays visible');
    await page.getByRole('button', { name: 'Save comment' }).click();

    await page.getByRole('button', { name: 'readme.md' }).click();
    const docsLine = page.getByTestId('diff-line').filter({ hasText: 'changed docs' });
    await docsLine.click();
    await page.getByTestId('comment-body').fill('docs comment in sidebar');
    await page.getByRole('button', { name: 'Save comment' }).click();

    await expect(page.getByTestId('file-tree').getByTestId('file-comment-count').filter({ hasText: '1' })).toHaveCount(2);
    await expect(page.getByTestId('file-tree')).toContainText('lib.rs');
    await expect(page.getByTestId('file-tree')).toContainText('readme.md');
    await expect(page.getByTestId('inline-comment')).toContainText('docs comment in sidebar');
    await expect(page.getByTestId('inline-comment-cell').first()).toHaveAttribute('colspan', '2');
    await expect(page.getByTestId('inline-comment-cell').first()).toHaveCSS('white-space', 'normal');
    await expect(page.getByTestId('inline-comment-header').first()).toContainText('Comment on line R1');
    await page.getByRole('button', { name: 'lib.rs' }).click();
    await expect(page.getByTestId('inline-comment').filter({ hasText: 'src comment that stays visible' })).toBeVisible();
    await expect(page.getByTestId('inline-comment').filter({ hasText: 'src comment that stays visible' })).toContainText('Comment on line R12');

    const toggle = page.getByTestId('inline-comment-toggle').first();
    await toggle.click();
    await expect(page.getByTestId('inline-comment-body').first()).toBeHidden();
    await toggle.click();
    await expect(page.getByTestId('inline-comment-body').first()).toBeVisible();
  } finally {
    stopServer(server.process);
    repo.cleanup();
  }
});

test('persists comments after reload and scopes them by base branch', async ({ page }) => {
  const repo = createRepoFixture();
  const server = await startServer(repo.path);

  try {
    await page.goto(server.url);

    const targetLine = page.getByTestId('diff-line').filter({ hasText: 'feature line' });
    await targetLine.click();
    await page.getByTestId('comment-body').fill('persisted note');
    await page.getByRole('button', { name: 'Save comment' }).click();

    await page.reload();

    const reloadedLine = page.getByTestId('diff-line').filter({ hasText: 'feature line' });
    await expect(reloadedLine.getByTestId('comment-marker')).toContainText('1');

    await reloadedLine.click();
    await expect(page.getByTestId('comment-body')).toHaveValue('persisted note');
    await page.getByRole('button', { name: 'Cancel' }).click();

    await page.getByTestId('base-branch').selectOption('origin/release');
    await expect(page.getByTestId('diff-view').getByTestId('comment-marker')).toHaveCount(0);

    await page.getByTestId('base-branch').selectOption('origin/main');
    await expect(page.getByTestId('diff-line').filter({ hasText: 'feature line' }).getByTestId('comment-marker')).toContainText('1');
  } finally {
    stopServer(server.process);
    repo.cleanup();
  }
});

test('keeps changed inline thread status visible in the compact header when collapsed in split mode', async ({ page }) => {
  const repo = createRepoFixture();
  const server = await startServer(repo.path);

  try {
    await page.goto(server.url);

    const featureLine = page.getByTestId('diff-line').filter({ hasText: 'feature line' });
    await featureLine.click();
    await page.getByTestId('comment-body').fill('changed inline comment');
    await page.getByRole('button', { name: 'Save comment' }).click();

    repo.writeFile('src/lib.rs', sourceLines('feature line updated'));
    repo.commit('refresh head');

    await page.evaluate(() => {
      window.dispatchEvent(new Event('focus'));
    });

     await expect(page.getByTestId('refresh-button')).toBeVisible();
     await page.getByTestId('refresh-button').click();
     await page.getByRole('button', { name: 'Split' }).click();

     const inlineComment = page.getByTestId('inline-comment').filter({ hasText: 'changed inline comment' });
     await expect(inlineComment.getByTestId('inline-comment-cell')).toHaveAttribute('colspan', '3');

     const toggle = inlineComment.getByTestId('inline-comment-toggle');
     await toggle.click();

    await expect(inlineComment.getByTestId('inline-comment-body')).toBeHidden();
    await expect(inlineComment.getByTestId('inline-comment-header')).toBeVisible();
    await expect(inlineComment.getByTestId('inline-comment-header')).toContainText('Comment on line R12');
    await expect(inlineComment.getByTestId('inline-comment-header')).toContainText('Changed');
  } finally {
    stopServer(server.process);
    repo.cleanup();
  }
});

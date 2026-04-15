import { expect, test } from '@playwright/test';
import { createRepoFixture, sourceLines, startServer, stopServer } from './support/review-shell-fixture';

test('supports add edit delete and cancel in the comment modal', async ({ page }) => {
  const repo = createRepoFixture();
  const server = await startServer(repo.path);

  try {
    await page.goto(server.url);

    const targetLine = page.getByTestId('diff-line').filter({ hasText: 'feature line' });
    await expect(targetLine.getByTestId('comment-add-trigger')).toBeHidden();
    await targetLine.click();
    await expect(page.getByTestId('comment-modal')).toBeHidden();

    await targetLine.hover();
    await expect(targetLine.getByTestId('comment-add-trigger')).toBeVisible();
    await targetLine.getByTestId('comment-add-trigger').click();
    await expect(page.getByTestId('comment-modal')).toBeVisible();

    await page.getByRole('button', { name: 'Cancel' }).click();
    await targetLine.getByTestId('comment-add-trigger').focus();
    await expect(targetLine.getByTestId('comment-add-trigger')).toBeVisible();
    await page.keyboard.press('Enter');
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

test('renders the add-comment trigger in a fixed gutter between line numbers and code', async ({ page }) => {
  const repo = createRepoFixture();
  const server = await startServer(repo.path);

  try {
    await page.goto(server.url);

    const line = page.getByTestId('diff-line').filter({ hasText: 'feature line' });
    await line.hover();

    const gutter = line.getByTestId('comment-add-gutter');
    await expect(gutter).toBeVisible();
    await expect(gutter.getByTestId('comment-add-trigger')).toBeVisible();

    const cells = line.locator('td');
    await expect(cells.nth(0)).toHaveClass(/line-number/);
    await expect(cells.nth(1)).toHaveClass(/line-number/);
    await expect(cells.nth(2)).toHaveAttribute('data-testid', 'comment-add-gutter');
    await expect(cells.nth(3)).toContainText('feature line');
  } finally {
    stopServer(server.process);
    repo.cleanup();
  }
});

test('keeps single-line diff rows compact with the fixed add-comment gutter', async ({ page }) => {
  const repo = createRepoFixture();
  const server = await startServer(repo.path);

  try {
    await page.goto(server.url);

    const line = page.getByTestId('diff-line').filter({ hasText: 'feature line' });
    await line.hover();

    const box = await line.boundingBox();
    expect(box).not.toBeNull();
    expect(box?.height ?? 0).toBeLessThan(40);
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
    await featureLine.hover();
    await featureLine.getByTestId('comment-add-trigger').click();
    await page.getByTestId('comment-body').fill('src comment that stays visible');
    await page.getByRole('button', { name: 'Save comment' }).click();

    await page.getByRole('button', { name: 'readme.md' }).click();
    const docsLine = page.getByTestId('diff-line').filter({ hasText: 'changed docs' });
    await docsLine.hover();
    await docsLine.getByTestId('comment-add-trigger').click();
    await page.getByTestId('comment-body').fill('docs comment in sidebar');
    await page.getByRole('button', { name: 'Save comment' }).click();

    await expect(page.getByTestId('file-tree').getByTestId('file-comment-count').filter({ hasText: '1' })).toHaveCount(2);
    await expect(page.getByTestId('file-tree')).toContainText('lib.rs');
    await expect(page.getByTestId('file-tree')).toContainText('readme.md');
    await expect(page.getByTestId('inline-comment')).toContainText('docs comment in sidebar');
    await expect(page.getByTestId('inline-comment-cell').first()).toHaveAttribute('colspan', '3');
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
    await targetLine.hover();
    await targetLine.getByTestId('comment-add-trigger').click();
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
    await featureLine.hover();
    await featureLine.getByTestId('comment-add-trigger').click();
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
      await expect(inlineComment.getByTestId('inline-comment-cell')).toHaveAttribute('colspan', '5');

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

test('supports creating and editing comments on removed lines in split mode', async ({ page }) => {
  const repo = createRepoFixture();
  repo.writeFile('src/lib.rs', sourceLines('feature line second').replace('feature line second\n', ''));
  const server = await startServer(repo.path);

  try {
    await page.goto(server.url);
    await page.getByRole('button', { name: 'Split' }).click();

    const removedLine = page.getByTestId('diff-line').filter({ hasText: 'base value' });
    await removedLine.hover();
    await expect(removedLine.getByTestId('comment-add-trigger')).toBeVisible();
    await removedLine.getByTestId('comment-add-trigger').click();
    await expect(page.getByTestId('comment-modal')).toBeVisible();

    await page.getByTestId('comment-body').fill('removed line note');
    await page.getByRole('button', { name: 'Save comment' }).click();

    await expect(removedLine.getByTestId('comment-marker')).toContainText('1');
    await removedLine.click();
    await expect(page.getByTestId('comment-body')).toHaveValue('removed line note');
  } finally {
    stopServer(server.process);
    repo.cleanup();
  }
});

test('keeps the add-comment trigger usable on touch layouts without hover', async ({ browser }) => {
  const context = await browser.newContext({
    hasTouch: true,
    viewport: { width: 390, height: 844 }
  });
  const page = await context.newPage();
  const repo = createRepoFixture();
  const server = await startServer(repo.path);

  try {
    await page.goto(server.url);

    const targetLine = page.getByTestId('diff-line').filter({ hasText: 'feature line' });
    const addTrigger = targetLine.getByTestId('comment-add-trigger');
    await expect(addTrigger).toBeVisible();
    await addTrigger.click();
    await expect(page.getByTestId('comment-modal')).toBeVisible();
  } finally {
    stopServer(server.process);
    repo.cleanup();
    await context.close();
  }
});

test('keeps the add-comment trigger reachable by keyboard focus', async ({ page }) => {
  const repo = createRepoFixture();
  const server = await startServer(repo.path);

  try {
    await page.goto(server.url);

    const addTrigger = page.getByTestId('diff-line').filter({ hasText: 'feature line' }).getByTestId('comment-add-trigger');
    for (let attempt = 0; attempt < 20; attempt += 1) {
      await page.keyboard.press('Tab');
      if (await addTrigger.evaluate((element) => element === document.activeElement)) {
        break;
      }
    }

    await expect(addTrigger).toBeFocused();
    await page.keyboard.press('Enter');
    await expect(page.getByTestId('comment-modal')).toBeVisible();
  } finally {
    stopServer(server.process);
    repo.cleanup();
  }
});

test('scopes comments separately for branch review and each selected commit', async ({ page }) => {
  const repo = createRepoFixture();
  const server = await startServer(repo.path);

  try {
    await page.goto(server.url);

    await addComment(page, 'feature line second', 'branch note');
    await expect(page.getByTestId('diff-line').filter({ hasText: 'feature line second' }).getByTestId('comment-marker')).toContainText('1');

    await page.getByTestId('review-mode').selectOption('commit');
    const secondCommit = page.getByTestId('commit-select').locator('option').filter({ hasText: 'second feature' });
    await page.getByTestId('commit-select').selectOption(await secondCommit.getAttribute('value'));
    await expect(page.getByTestId('diff-view')).toContainText('feature line second');
    await expect(page.getByTestId('diff-view').getByTestId('comment-marker')).toHaveCount(0);

    await addComment(page, 'feature line second', 'commit note second');
    await expect(page.getByTestId('diff-line').filter({ hasText: 'feature line second' }).getByTestId('comment-marker')).toContainText('1');

    const firstCommit = page.getByTestId('commit-select').locator('option').filter({ hasText: 'first feature' });
    await page.getByTestId('commit-select').selectOption(await firstCommit.getAttribute('value'));
    await expect(page.getByTestId('diff-view').getByTestId('comment-marker')).toHaveCount(0);

    await addComment(page, 'feature line first', 'commit note first');
    await expect(page.getByTestId('diff-line').filter({ hasText: 'feature line first' }).getByTestId('comment-marker')).toContainText('1');

    await page.getByTestId('commit-select').selectOption(await secondCommit.getAttribute('value'));
    await expect(page.getByTestId('diff-view')).toContainText('feature line second');
    await expect(page.getByTestId('diff-line').filter({ hasText: 'feature line second' }).getByTestId('comment-marker')).toContainText('1');
    await expect(page.getByTestId('diff-view')).not.toContainText('commit note first');

    await page.getByTestId('review-mode').selectOption('branch');
    await expect(page.getByTestId('file-tree')).toContainText('readme.md');
    await expect(page.getByTestId('diff-line').filter({ hasText: 'feature line second' }).getByTestId('comment-marker')).toContainText('1');
    await expect(page.getByTestId('inline-comment').filter({ hasText: 'branch note' })).toBeVisible();
    await expect(page.getByTestId('inline-comment').filter({ hasText: 'commit note second' })).toHaveCount(0);
  } finally {
    stopServer(server.process);
    repo.cleanup();
  }
});

test('scopes comments separately for LOCAL CHANGES and historical commits', async ({ page }) => {
  const repo = createRepoFixture();
  repo.writeFile('src/lib.rs', sourceLines('local worktree line'));
  const server = await startServer(repo.path);

  try {
    await page.goto(server.url);
    await expect(page.getByTestId('diff-view')).toContainText('local worktree line');

    await page.getByTestId('review-mode').selectOption('commit');
    await expect(page.getByTestId('commit-select')).toHaveValue('LOCAL_CHANGES');

    await addComment(page, 'local worktree line', 'local changes note');
    await expect(page.getByTestId('diff-line').filter({ hasText: 'local worktree line' }).getByTestId('comment-marker')).toContainText('1');

    const secondCommit = page.getByTestId('commit-select').locator('option').filter({ hasText: 'second feature' });
    await page.getByTestId('commit-select').selectOption(await secondCommit.getAttribute('value'));
    await expect(page.getByTestId('diff-view')).toContainText('feature line second');
    await expect(page.getByTestId('diff-view').getByTestId('comment-marker')).toHaveCount(0);

    await addComment(page, 'feature line second', 'historical commit note');
    await expect(page.getByTestId('diff-line').filter({ hasText: 'feature line second' }).getByTestId('comment-marker')).toContainText('1');

    await page.getByTestId('commit-select').selectOption('LOCAL_CHANGES');
    await expect(page.getByTestId('diff-view')).toContainText('local worktree line');
    await expect(page.getByTestId('diff-line').filter({ hasText: 'local worktree line' }).getByTestId('comment-marker')).toContainText('1');
    await expect(page.getByTestId('diff-view')).not.toContainText('historical commit note');
  } finally {
    stopServer(server.process);
    repo.cleanup();
  }
});

async function addComment(page: import('@playwright/test').Page, lineText: string, body: string) {
  const line = page.getByTestId('diff-line').filter({ hasText: lineText });
  await line.hover();
  await line.getByTestId('comment-add-trigger').click();
  await page.getByTestId('comment-body').fill(body);
  await page.getByRole('button', { name: 'Save comment' }).click();
}

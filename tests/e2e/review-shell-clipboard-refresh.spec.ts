import { expect, test } from '@playwright/test';
import { createRepoFixture, sourceLines, startServer, stopServer } from './support/review-shell-fixture';

test('copies grouped comment export to the clipboard', async ({ page }) => {
  const repo = createRepoFixture();
  const server = await startServer(repo.path);

  await page.addInitScript(() => {
    window.__copiedText = null;
    Object.defineProperty(navigator, 'clipboard', {
      configurable: true,
      value: {
        writeText: async (text: string) => {
          window.__copiedText = text;
        }
      }
    });
  });

  try {
    await page.goto(server.url);

    const copyButton = page.getByTestId('copy-button');
    await expect(copyButton).toHaveText('Copy to clipboard');

    await addComment(page, 'feature line', 'src comment');
    await page.getByRole('button', { name: 'readme.md' }).click();
    await addComment(page, 'changed docs', 'docs comment');

    await copyButton.click();

    await page.waitForFunction(() => window.__copiedText !== null);
    const copiedText = await page.evaluate(() => window.__copiedText);
    await expect(copyButton).toHaveText('Copied');
    expect(copiedText).toMatch(/src\/lib\.rs - [0-9a-f]{40}/);
    expect(copiedText).toContain('docs/readme.md - ');
    expect(copiedText).toContain('Comment: src comment');
    expect(copiedText).toContain('Comment: docs comment');
    expect(copiedText).toContain('line 2');
    expect(copiedText).toContain('line 22');
  } finally {
    stopServer(server.process);
    repo.cleanup();
  }
});

test('clears stale copied feedback before showing a fallback export modal on retry failure', async ({ page }) => {
  const repo = createRepoFixture();
  const server = await startServer(repo.path);

  await page.addInitScript(() => {
    window.__clipboardWriteAttempts = 0;
    window.__copyButtonTextDuringClipboardWrite = null;

    Object.defineProperty(navigator, 'clipboard', {
      configurable: true,
      value: {
        writeText: async () => {
          window.__clipboardWriteAttempts += 1;
          if (window.__clipboardWriteAttempts === 1) {
            return;
          }

          const copyButton = document.querySelector('[data-testid="copy-button"]');
          window.__copyButtonTextDuringClipboardWrite = copyButton instanceof HTMLElement
            ? copyButton.innerText.trim()
            : null;
          throw new Error('denied');
        }
      }
    });
  });

  try {
    await page.goto(server.url);

    await addComment(page, 'feature line', 'fallback note');
    const copyButton = page.getByTestId('copy-button');
    await copyButton.click();

    await expect(copyButton).toHaveText('Copied');

    await copyButton.click();

    await expect(page.getByTestId('clipboard-fallback-modal')).toBeVisible();
    expect(await page.evaluate(() => window.__copyButtonTextDuringClipboardWrite)).not.toBe('Copied');
    await expect(copyButton).not.toHaveText('Copied');
    await expect(page.getByTestId('clipboard-fallback-text')).toHaveValue(/fallback note/);
  } finally {
    stopServer(server.process);
    repo.cleanup();
  }
});

test('shows an error banner when clipboard export generation fails', async ({ page }) => {
  const repo = createRepoFixture();
  const server = await startServer(repo.path);
  const pageErrors: string[] = [];
  let releaseClipboardExportResponse = () => {};
  let resolveClipboardExportStarted = () => {};
  const clipboardExportStarted = new Promise<void>((resolve) => {
    resolveClipboardExportStarted = resolve;
  });

  await page.route('**/api/clipboard-export', async (route) => {
      resolveClipboardExportStarted();
      await new Promise<void>((release) => {
        releaseClipboardExportResponse = release;
      });
      await route.fulfill({
        status: 500,
        contentType: 'application/json',
        body: JSON.stringify({ error: 'clipboard export failed' })
      });
    });

  page.on('pageerror', (error) => {
    pageErrors.push(error.message);
  });

  try {
    await page.goto(server.url);

    await addComment(page, 'feature line', 'backend export note');
    const copyButton = page.getByTestId('copy-button');
    await copyButton.click();
    await clipboardExportStarted;

    await expect(copyButton).toHaveText('Copy to clipboard');
    await expect(copyButton).not.toHaveText('Copied');

    releaseClipboardExportResponse();

    await expect(page.getByTestId('error-banner')).toContainText('clipboard export failed');
    await expect(page.getByTestId('clipboard-fallback-modal')).toBeHidden();
    await expect(copyButton).not.toHaveText('Copied');
    expect(pageErrors).toEqual([]);
  } finally {
    stopServer(server.process);
    repo.cleanup();
  }
});

test('detects repo refreshes and remaps comments as changed or stale', async ({ page }) => {
  const repo = createRepoFixture();
  const server = await startServer(repo.path);

  try {
    await page.goto(server.url);

    await addComment(page, 'feature line', 'changed comment');
    await page.getByRole('button', { name: 'readme.md' }).click();
    await addComment(page, 'changed docs', 'stale comment');

    repo.writeFile('src/lib.rs', sourceLines('feature line updated'));
    repo.removeFile('docs/readme.md');
    repo.commit('refresh head');

    await page.evaluate(() => {
      window.dispatchEvent(new Event('focus'));
    });

    await expect(page.getByTestId('refresh-button')).toBeVisible();
    await page.getByTestId('refresh-button').click();
    await page.getByTestId('file-tree').getByRole('button', { name: 'lib.rs' }).click();

    const updatedLine = page.getByTestId('diff-line').filter({ hasText: 'feature line updated' });
    await expect(updatedLine.getByTestId('comment-marker')).toContainText('1');
    await updatedLine.click();
    await expect(page.getByTestId('comment-status')).toContainText('Changed');
    await page.getByRole('button', { name: 'Cancel' }).click();

    await expect(page.getByTestId('stale-comment-trigger')).toContainText('stale comment');
    await page.getByTestId('stale-comment-trigger').click();
    await expect(page.getByTestId('comment-status')).toContainText('Stale');
    await expect(page.getByRole('button', { name: 'Save comment' })).toBeHidden();
  } finally {
    stopServer(server.process);
    repo.cleanup();
  }
});

test('clears loading and keeps the shell usable when refresh reload fails', async ({ page }) => {
  const repo = createRepoFixture();
  const server = await startServer(repo.path);
  let reviewRequests = 0;

  await page.route(/\/api\/review(?:\?.*)?$/, async (route) => {
    reviewRequests += 1;
    if (reviewRequests === 1) {
      await route.fulfill({ response: await route.fetch() });
      return;
    }

    await route.fulfill({
      status: 500,
      contentType: 'application/json',
      body: JSON.stringify({ error: 'refresh failed' })
    });
  });

  try {
    await page.goto(server.url);

    await page.evaluate(() => {
      document.querySelector('[data-testid="refresh-button"]')?.click();
    });

    await expect(page.getByTestId('error-banner')).toContainText('refresh failed');
    await expect(page.getByTestId('loading-state')).toBeHidden();
    await page.getByRole('button', { name: 'readme.md' }).click();
    await expect(page.getByTestId('diff-view')).toContainText('changed docs');
  } finally {
    stopServer(server.process);
    repo.cleanup();
  }
});

test('keeps head-side comments across a rename when the anchored text still maps', async ({ page }) => {
  const repo = createRepoFixture();
  const server = await startServer(repo.path);

  try {
    await page.goto(server.url);

    await addComment(page, 'feature line', 'rename-safe comment');

    repo.writeFile('src/renamed.rs', sourceLines('feature line'));
    repo.removeFile('src/lib.rs');
    repo.commit('rename file');

    await page.evaluate(() => {
      window.dispatchEvent(new Event('focus'));
    });

    await expect(page.getByTestId('refresh-button')).toBeVisible();
    await page.getByTestId('refresh-button').click();

    await expect(page.getByTestId('file-tree').getByRole('button', { name: 'renamed.rs' })).toBeVisible();
    await page.getByTestId('file-tree').getByRole('button', { name: 'renamed.rs' }).click();

    const renamedLine = page.getByTestId('diff-line').filter({ hasText: 'feature line' });
    await expect(renamedLine.getByTestId('comment-marker')).toContainText('1');
    await renamedLine.click();
    await expect(page.getByTestId('comment-body')).toHaveValue('rename-safe comment');
  } finally {
    stopServer(server.process);
    repo.cleanup();
  }
});

async function addComment(page: import('@playwright/test').Page, lineText: string, body: string) {
  const line = page.getByTestId('diff-line').filter({ hasText: lineText });
  await line.click();
  await page.getByTestId('comment-body').fill(body);
  await page.getByRole('button', { name: 'Save comment' }).click();
}

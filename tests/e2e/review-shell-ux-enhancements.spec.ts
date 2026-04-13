import { expect, test } from '@playwright/test';
import { createRepoFixture, sourceLines, startServer, stopServer } from './support/review-shell-fixture';

test('collapses and expands directories in the file tree', async ({ page }) => {
  const repo = createRepoFixture();
  const server = await startServer(repo.path);

  try {
    await page.goto(server.url);

    const srcToggle = page.locator('.file-tree-node', { hasText: 'src' }).getByTestId('tree-toggle');
    await expect(srcToggle).toHaveAttribute('aria-expanded', 'true');

    await srcToggle.click();

    await expect(srcToggle).toHaveText('+');
    await expect(srcToggle).toHaveAttribute('aria-expanded', 'false');
    await expect(page.getByRole('button', { name: 'lib.rs' })).toBeHidden();
    await expect(page.getByRole('button', { name: 'readme.md' })).toBeVisible();

    await srcToggle.click();

    await expect(srcToggle).toHaveText('-');
    await expect(srcToggle).toHaveAttribute('aria-expanded', 'true');
    await expect(page.getByRole('button', { name: 'lib.rs' })).toBeVisible();
  } finally {
    stopServer(server.process);
    repo.cleanup();
  }
});

test('persists viewed state for the current review and clears it after a refresh to a new head', async ({ page }) => {
  const repo = createRepoFixture();
  const server = await startServer(repo.path);

  try {
    await page.goto(server.url);

    await page.getByRole('button', { name: 'lib.rs' }).click();
    const viewedCheckbox = page.getByTestId('file-viewed-checkbox');
    await viewedCheckbox.check();
    await expect(viewedCheckbox).toBeChecked();
    await expect(page.getByTestId('file-tree').getByTestId('file-viewed-badge')).toHaveCount(1);

    await page.reload();

    await page.getByRole('button', { name: 'lib.rs' }).click();
    await expect(page.getByTestId('file-viewed-checkbox')).toBeChecked();
    await expect(page.getByTestId('file-tree').getByTestId('file-viewed-badge')).toHaveCount(1);

    repo.writeFile('src/lib.rs', sourceLines('feature line updated'));
    repo.commit('refresh head');

    await page.evaluate(() => {
      window.dispatchEvent(new Event('focus'));
    });

    await page.getByTestId('refresh-button').click();
    await page.getByRole('button', { name: 'lib.rs' }).click();
    await expect(page.getByTestId('file-viewed-checkbox')).not.toBeChecked();
    await expect(page.getByTestId('file-tree').getByTestId('file-viewed-badge')).toHaveCount(0);
  } finally {
    stopServer(server.process);
    repo.cleanup();
  }
});

test('scopes viewed state by base branch', async ({ page }) => {
  const repo = createRepoFixture();
  const server = await startServer(repo.path);

  try {
    await page.goto(server.url);

    await page.getByRole('button', { name: 'lib.rs' }).click();
    await page.getByTestId('file-viewed-checkbox').check();
    await expect(page.getByTestId('file-tree').getByTestId('file-viewed-badge')).toHaveCount(1);

    await page.getByTestId('base-branch').selectOption('origin/release');
    await expect(page.getByTestId('file-viewed-checkbox')).not.toBeChecked();
    await expect(page.getByTestId('file-tree').getByTestId('file-viewed-badge')).toHaveCount(0);

    await page.getByTestId('base-branch').selectOption('origin/main');
    await expect(page.getByTestId('file-viewed-checkbox')).toBeChecked();
    await expect(page.getByTestId('file-tree').getByTestId('file-viewed-badge')).toHaveCount(1);
  } finally {
    stopServer(server.process);
    repo.cleanup();
  }
});

test('renders syntax highlighting for supported diff file types', async ({ page }) => {
  const repo = createRepoFixture();
  const server = await startServer(repo.path);

  try {
    await page.goto(server.url);

    await page.getByRole('button', { name: 'example.ts' }).click();
    await expect.poll(async () => page.locator('.diff-code .token.keyword').count()).toBeGreaterThan(0);

    await page.getByRole('button', { name: 'readme.md' }).click();
    await expect.poll(async () => page.locator('.diff-code .token').count()).toBeGreaterThan(0);

    await page.getByRole('button', { name: 'settings.json' }).click();
    await expect.poll(async () => page.locator('.diff-code .token.property').count()).toBeGreaterThan(0);

    await page.getByRole('button', { name: 'layout.xml' }).click();
    await expect.poll(async () => page.locator('.diff-code .token.tag').count()).toBeGreaterThan(0);
  } finally {
    stopServer(server.process);
    repo.cleanup();
  }
});

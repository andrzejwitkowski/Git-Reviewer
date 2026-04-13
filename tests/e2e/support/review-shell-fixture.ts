import { execFileSync, spawn, type ChildProcessWithoutNullStreams } from 'node:child_process';
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const CLEANUP_ATTEMPTS = 5;
const CLEANUP_RETRY_DELAY_MS = 100;

export function createRepoFixture() {
  const root = mkdtempSync(join(tmpdir(), 'git-reviewer-e2e-'));
  runGit(root, ['init', '-b', 'main']);
  runGit(root, ['config', 'user.name', 'Git Reviewer']);
  runGit(root, ['config', 'user.email', 'git-reviewer@example.com']);

  writeRepoFile(root, 'src/lib.rs', sourceLines('base value'));
  writeRepoFile(root, 'examples/example.ts', 'export const baseAnswer = { ok: false, value: 0 };\n');
  writeRepoFile(root, 'config/settings.json', '{\n  "ok": false,\n  "count": 0\n}\n');
  writeRepoFile(root, 'config/layout.xml', '<root attr="base">value</root>\n');
  writeRepoFile(root, 'docs/readme.md', '# base docs\nline 2\nline 3\n');
  runGit(root, ['add', '-A']);
  runGit(root, ['commit', '-m', 'base']);
  const baseSha = runGit(root, ['rev-parse', 'HEAD']).trim();

  runGit(root, ['update-ref', 'refs/remotes/origin/release', baseSha]);
  writeRepoFile(root, 'docs/readme.md', '# main baseline\nline 2\nline 3\n');
  runGit(root, ['add', '-A']);
  runGit(root, ['commit', '-m', 'main baseline']);
  const mainSha = runGit(root, ['rev-parse', 'HEAD']).trim();

  runGit(root, ['update-ref', 'refs/remotes/origin/main', mainSha]);
  runGit(root, ['checkout', '-b', 'feature/shell']);

  writeRepoFile(root, 'src/lib.rs', sourceLines('feature line first'));
  writeRepoFile(root, 'examples/example.ts', 'export const answer = { ok: true, value: 7 };\n');
  writeRepoFile(root, 'config/settings.json', '{\n  "ok": true,\n  "count": 7\n}\n');
  writeRepoFile(root, 'config/layout.xml', '<root attr="feature">value</root>\n');
  writeRepoFile(root, 'docs/readme.md', '# changed docs\nline 2\nline 3\n');
  runGit(root, ['add', '-A']);
  runGit(root, ['commit', '-m', 'first feature']);

  writeRepoFile(root, 'src/lib.rs', sourceLines('feature line second'));
  runGit(root, ['add', '-A']);
  runGit(root, ['commit', '-m', 'second feature']);

  return {
    path: root,
    root,
    cleanup() {
      cleanupDir(root);
    },
    writeFile(relativePath: string, content: string) {
      writeRepoFile(root, relativePath, content);
    },
    removeFile(relativePath: string) {
      rmSync(join(root, relativePath), { force: true });
    },
    commit(message: string) {
      runGit(root, ['add', '-A']);
      runGit(root, ['commit', '-m', message]);
      return runGit(root, ['rev-parse', 'HEAD']).trim();
    },
    headSha() {
      return runGit(root, ['rev-parse', 'HEAD']).trim();
    },
    updateRemoteBranch(name: string, target: string) {
      runGit(root, ['update-ref', `refs/remotes/${name}`, target]);
    }
  };
}

export function createLargeFileFixture() {
  const root = mkdtempSync(join(tmpdir(), 'git-reviewer-large-'));
  runGit(root, ['init', '-b', 'main']);
  runGit(root, ['config', 'user.name', 'Git Reviewer']);
  runGit(root, ['config', 'user.email', 'git-reviewer@example.com']);

  writeRepoFile(root, 'src/huge.ts', numberedLines(820, 'base marker'));
  runGit(root, ['add', '-A']);
  runGit(root, ['commit', '-m', 'base']);
  const baseSha = runGit(root, ['rev-parse', 'HEAD']).trim();

  runGit(root, ['update-ref', 'refs/remotes/origin/main', baseSha]);
  runGit(root, ['checkout', '-b', 'feature/large']);
  writeRepoFile(root, 'src/huge.ts', numberedLines(820, 'updated marker'));
  runGit(root, ['add', '-A']);
  runGit(root, ['commit', '-m', 'feature']);

  return {
    path: root,
    cleanup() {
      cleanupDir(root);
    }
  };
}

export async function startServer(repoPath: string) {
  const process = spawn('cargo', ['run', '--quiet', '--', repoPath, '--port', '0'], {
    cwd: processCwd(),
    stdio: ['ignore', 'pipe', 'pipe']
  });

  const url = await waitForServerUrl(process);
  return { process, url };
}

export function stopServer(process: ChildProcessWithoutNullStreams) {
  if (!process.killed) {
    process.kill('SIGTERM');
  }
}

export function sourceLines(target: string) {
  const lines = Array.from({ length: 25 }, (_, index) => `line ${index + 1}`);
  lines[11] = target;
  return `${lines.join('\n')}\n`;
}

function numberedLines(total: number, marker: string) {
  const lines = Array.from({ length: total }, (_, index) => `line ${index + 1}`);
  lines[410] = marker;
  return `${lines.join('\n')}\n`;
}

function writeRepoFile(root: string, relativePath: string, content: string) {
  const filePath = join(root, relativePath);
  mkdirSync(dirname(filePath), { recursive: true });
  writeFileSync(filePath, content);
}

function runGit(cwd: string, args: string[]) {
  return execFileSync('git', args, {
    cwd,
    encoding: 'utf8'
  });
}

function cleanupDir(path: string) {
  for (let attempt = 1; attempt <= CLEANUP_ATTEMPTS; attempt += 1) {
    try {
      rmSync(path, { force: true, recursive: true });
      return;
    } catch (error) {
      if (!(error instanceof Error) || !('code' in error) || error.code !== 'EBUSY' || attempt === CLEANUP_ATTEMPTS) {
        throw error;
      }
      sleep(CLEANUP_RETRY_DELAY_MS);
    }
  }
}

function sleep(ms: number) {
  const end = Date.now() + ms;
  while (Date.now() < end) {
    // Busy wait is fine here; cleanup only runs in test teardown.
  }
}

function waitForServerUrl(process: ChildProcessWithoutNullStreams) {
  return new Promise<string>((resolve, reject) => {
    let stdout = '';
    let stderr = '';

    const timer = setTimeout(() => {
      reject(new Error(`Timed out waiting for server URL.\nstdout:\n${stdout}\nstderr:\n${stderr}`));
    }, 30000);

    process.stdout.on('data', (chunk) => {
      stdout += chunk.toString();
      const match = stdout.match(/http:\/\/127\.0\.0\.1:\d+/);
      if (match) {
        clearTimeout(timer);
        resolve(match[0]);
      }
    });

    process.stderr.on('data', (chunk) => {
      stderr += chunk.toString();
    });

    process.on('exit', (code) => {
      clearTimeout(timer);
      reject(new Error(`Server exited early with code ${code}.\nstdout:\n${stdout}\nstderr:\n${stderr}`));
    });
  });
}

function processCwd() {
  return join(dirname(fileURLToPath(import.meta.url)), '..', '..', '..');
}

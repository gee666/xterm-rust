import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {mkdtempSync, mkdirSync, readFileSync, writeFileSync, existsSync, rmSync, symlinkSync, unlinkSync} from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import test from 'node:test';

const root = fileURLToPath(new URL('../', import.meta.url));
const workflow = readFileSync(path.join(root, '.github/workflows/rust.yml'), 'utf8');
function script(name) {
  const step = workflow.split(`      - name: ${name}\n`)[1]?.split(/\n      - name: |\n  [a-z].*:/)[0];
  const body = step?.split('        run: |\n')[1];
  assert.ok(body, `Missing PowerShell block ${name}`);
  return body.split('\n').map(line => line.startsWith('          ') ? line.slice(10) : line).join('\n');
}
const setup = script('Create a short physical Windows quality worktree');
const cleanup = script('Remove only the owned short Windows worktree');
const budget = script('Record the actual Windows formatting path budget');

test('native gates and artifacts use the selected physical root without changing gate commands', () => {
  assert.match(workflow, /XTERM_QUALITY_ROOT: \$\{\{ github\.workspace }}/);
  assert.equal((workflow.match(/working-directory: \$\{\{ env\.XTERM_QUALITY_ROOT }}/g) ?? []).length, 4);
  assert.match(workflow, /path: \$\{\{ env\.XTERM_QUALITY_ROOT }}\/results\/quality\//);
  assert.match(workflow, /run: node scripts\/quality-gates\.mjs --out results\/quality\/\$\{\{ matrix\.os }}-\$\{\{ matrix\.toolchain }} --timeout-ms 600000/);
  assert.ok(workflow.indexOf('Preserve command logs and native provenance') < workflow.indexOf('Remove only the owned short Windows worktree'));
});

test('the current target inventory fits the shortened hosted Windows command line', () => {
  const expression = "[IO.Path]::GetFullPath((Join-Path $env:RUNNER_TEMP '../../q'))";
  assert.ok(setup.includes(expression));
  assert.ok(cleanup.includes(expression));
  const result = spawnSync('cargo', ['metadata', '--no-deps', '--format-version', '1', '--locked'], {
    cwd: root, encoding: 'utf8', timeout: 30000
  });
  assert.equal(result.error, undefined);
  assert.equal(result.status, 0, result.stderr);
  const metadata = JSON.parse(result.stdout);
  const packages = metadata.packages.filter(pkg => metadata.workspace_members.includes(pkg.id));
  const groups = Object.groupBy(packages.flatMap(pkg => pkg.targets), target => target.edition);
  const hostedRoot = 'D:/q';
  const windowsCommandLineLimit = 32767;
  // Reserve the executable, --edition, --check and other non-file arguments.
  const nonFileArgumentReserve = 512;
  for (const [edition, targets] of Object.entries(groups)) {
    const characters = targets.reduce((total, target) => {
      const relative = path.relative(root, target.src_path);
      return total + hostedRoot.length + 1 + relative.length + 7;
    }, 0);
    assert.ok(characters <= windowsCommandLineLimit - nonFileArgumentReserve,
      `edition ${edition}: ${targets.length} roots need ${characters} file-argument characters`);
  }
});

test('Windows setup and cleanup preserve ownership, source identity and linked data', {skip: process.platform !== 'win32'}, async t => {
  mkdirSync(path.join(root, 'tmp'), {recursive: true});
  const fixture = mkdtempSync(path.join(root, 'tmp', 'windows-quality-'));
  const repo = path.join(fixture, 'repo');
  // Match the hosted two-level parent calculation without leaving our fixture.
  const temporary = path.join(fixture, 'runner', 'runner-temp');
  const short = path.join(fixture, 'q');
  const environment = path.join(fixture, 'github-env');
  const setupPath = path.join(fixture, 'setup.ps1');
  const cleanupPath = path.join(fixture, 'cleanup.ps1');
  const budgetPath = path.join(fixture, 'budget.ps1');
  const powershell = path.join(process.env.SystemRoot, 'System32/WindowsPowerShell/v1.0/powershell.exe');
  const env = {...process.env, RUNNER_TEMP: temporary, GITHUB_ENV: environment};
  delete env.XTERM_OWNED_QUALITY_ROOT;
  function git(args, cwd = repo) {
    const result = spawnSync('git', args, {cwd, encoding: 'utf8', timeout: 30000});
    assert.equal(result.error, undefined);
    assert.equal(result.status, 0, result.stderr);
    return result.stdout.trim();
  }
  function ps(file, owned, cwd = repo) {
    const childEnv = {...env};
    if (owned) childEnv.XTERM_OWNED_QUALITY_ROOT = owned;
    return spawnSync(powershell, ['-NoProfile', '-NonInteractive', '-ExecutionPolicy', 'Bypass', '-File', file], {
      cwd, env: childEnv, encoding: 'utf8', timeout: 30000
    });
  }
  function succeeded(result) {
    assert.equal(result.error, undefined);
    assert.equal(result.status, 0, result.stdout + result.stderr);
  }
  try {
    mkdirSync(repo);
    mkdirSync(temporary, {recursive: true});
    writeFileSync(path.join(temporary, 'keep.txt'), 'runner temporary data\n');
    writeFileSync(setupPath, setup);
    writeFileSync(cleanupPath, cleanup);
    writeFileSync(budgetPath, budget);
    writeFileSync(environment, '');
    git(['init', '--quiet']);
    git(['config', 'core.autocrlf', 'false']);
    writeFileSync(path.join(repo, 'source.txt'), 'immutable source\n');
    writeFileSync(path.join(repo, 'Cargo.toml'), '[package]\nname="quality-fixture"\nversion="0.1.0"\nedition="2021"\n[workspace]\n');
    writeFileSync(path.join(repo, 'Cargo.lock'), 'version=3\n[[package]]\nname="quality-fixture"\nversion="0.1.0"\n');
    mkdirSync(path.join(repo, 'src'));
    writeFileSync(path.join(repo, 'src/lib.rs'), '// Quality fixture.\n');
    git(['add', 'source.txt', 'Cargo.toml', 'Cargo.lock', 'src/lib.rs']);
    git(['-c', 'user.name=Quality test', '-c', 'user.email=quality@example.invalid', 'commit', '--quiet', '-m', 'fixture']);
    env.GITHUB_SHA = git(['rev-parse', 'HEAD']);

    await t.test('setup checks out the requested SHA and publishes only the owned root', () => {
      succeeded(ps(setupPath));
      assert.equal(git(['rev-parse', 'HEAD'], short), env.GITHUB_SHA);
      const published = readFileSync(environment, 'utf16le');
      assert.ok(published.includes(`XTERM_OWNED_QUALITY_ROOT=${short}`));
      assert.ok(published.includes(`XTERM_QUALITY_ROOT=${short}`));
    });
    await t.test('a trailing runner-temp separator resolves to the same protected checkout', () => {
      env.RUNNER_TEMP = temporary + path.sep;
      try {
        const result = ps(setupPath);
        assert.notEqual(result.status, 0);
        assert.match(result.stderr, /Refusing existing path/);
        assert.equal(git(['rev-parse', 'HEAD'], short), env.GITHUB_SHA);
      } finally { env.RUNNER_TEMP = temporary; }
    });
    await t.test('budget records the actual physical root and all metadata targets', () => {
      const result = ps(budgetPath, short, short);
      succeeded(result);
      const observed = JSON.parse(result.stdout);
      assert.equal(observed.physicalRoot, short);
      assert.equal(observed.groups.length, 1);
      assert.equal(observed.groups[0].roots, 1);
      assert.equal(observed.groups[0].edition, '2021');
      assert.equal(observed.groups[0].quotedFileArguments, path.join(short, 'src/lib.rs').length + 7);
    });
    await t.test('an existing path is preserved and setup fails visibly', () => {
      const result = ps(setupPath);
      assert.notEqual(result.status, 0);
      assert.match(result.stderr, /Refusing existing path/);
      assert.equal(readFileSync(path.join(short, 'source.txt'), 'utf8'), 'immutable source\n');
    });
    await t.test('cleanup rejects an unexpected ownership marker', () => {
      const result = ps(cleanupPath, repo);
      assert.notEqual(result.status, 0);
      assert.match(result.stderr, /Unexpected owned quality path/);
      assert.ok(existsSync(short));
    });
    await t.test('cleanup refuses a child junction and preserves its target', () => {
      const sentinel = path.join(fixture, 'sentinel');
      const junction = path.join(short, 'linked-data');
      mkdirSync(sentinel);
      writeFileSync(path.join(sentinel, 'keep.txt'), 'preserve\n');
      symlinkSync(sentinel, junction, 'junction');
      try {
        const result = ps(cleanupPath, short);
        assert.notEqual(result.status, 0);
        assert.match(result.stderr, /Refusing linked quality worktree cleanup/);
        assert.equal(readFileSync(path.join(sentinel, 'keep.txt'), 'utf8'), 'preserve\n');
        assert.ok(existsSync(short));
      } finally { unlinkSync(junction); }
    });
    await t.test('cleanup removes owned generated files after an earlier failed step', () => {
      mkdirSync(path.join(short, 'target'));
      writeFileSync(path.join(short, 'target', 'generated.txt'), 'generated\n');
      succeeded(ps(cleanupPath, short));
      assert.equal(existsSync(short), false);
      assert.equal(git(['status', '--porcelain']), '');
      assert.equal(git(['worktree', 'list', '--porcelain']).includes(short.replaceAll('\\', '/')), false);
      assert.equal(readFileSync(path.join(temporary, 'keep.txt'), 'utf8'), 'runner temporary data\n');
    });
    await t.test('cleanup refuses a root junction before inspecting descendants', () => {
      symlinkSync(path.join(fixture, 'sentinel'), short, 'junction');
      try {
        const result = ps(cleanupPath, short);
        assert.notEqual(result.status, 0);
        assert.match(result.stderr, /Refusing non-directory or linked quality root/);
        assert.equal(readFileSync(path.join(short, 'keep.txt'), 'utf8'), 'preserve\n');
      } finally { unlinkSync(short); }
    });
    await t.test('a failed checkout remains failed and cleanup preserves the source repository', () => {
      env.GITHUB_SHA = 'does-not-exist';
      const result = ps(setupPath);
      assert.notEqual(result.status, 0);
      assert.match(result.stderr, /Cannot create the short quality worktree/);
      succeeded(ps(cleanupPath, short));
      assert.equal(existsSync(short), false);
      assert.equal(readFileSync(path.join(repo, 'source.txt'), 'utf8'), 'immutable source\n');
      assert.equal(git(['status', '--porcelain']), '');
    });
  } finally {
    if (existsSync(short)) succeeded(ps(cleanupPath, short));
    rmSync(fixture, {recursive: true});
  }
});

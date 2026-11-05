// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
// Fresh unchanged-source execution. Fixtures are updated only with --write.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {createRequire} from 'node:module';
import {readFile, writeFile, mkdir} from 'node:fs/promises';
import path from 'node:path';
import vm from 'node:vm';

const root = path.resolve(import.meta.dirname, '..');
const source = path.resolve(process.env.XT_SOURCE_ROOT ?? path.join(root, 'source_repo'));
const dependencies = path.resolve(process.env.XT_NODE_MODULES ?? path.join(source, 'node_modules'));
const {build} = createRequire(path.join(dependencies, '../package.json'))(path.join(dependencies, 'esbuild'));
const out = path.join(root, 'results/buffer-cleanup');
await mkdir(out, {recursive: true});
const hashes = {};
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
const built = await build({
  absWorkingDir: root,
  entryPoints: [path.join(source, 'src/headless/Terminal.ts')],
  bundle: true, platform: 'browser', format: 'iife', globalName: 'CleanupSource',
  write: false, metafile: true,
  tsconfig: path.join(source, 'src/headless/tsconfig.json'),
  nodePaths: [dependencies], logLevel: 'silent'
});
for (const file of Object.keys(built.metafile.inputs)) {
  const absolute = path.resolve(root, file);
  hashes[path.relative(source, absolute).replaceAll('\\', '/')] = hash(await readFile(absolute));
}
const code = built.outputFiles[0].text;

function run(idle, cols, scrollback) {
  let token = 0;
  const callbacks = new Map();
  const request = callback => { callbacks.set(++token, callback); return token; };
  const cancel = id => callbacks.delete(id);
  const context = vm.createContext({
    structuredClone,
    console: {warn() {}, error() {}, log() {}},
    performance: {now: () => 0},
    navigator: {userAgent: 'cleanup-source-probe', platform: 'Linux'},
    setTimeout: request, clearTimeout: cancel,
    ...(idle ? {requestIdleCallback: request, cancelIdleCallback: cancel} : {})
  });
  vm.runInContext(code, context, {timeout: 5000});
  const term = new context.CleanupSource.Terminal({cols, rows: 4, scrollback, logLevel: 'off'});
  const rows = [];
  const narrow = cols / 4;
  function snapshot(tag) {
    const fields = [tag];
    for (const b of [term.buffers.normal, term.buffers.alt]) {
      const lengths = Array.from({length: b.lines.length}, (_, i) => b.lines.get(i).length);
      fields.push(lengths.join(','), String(b._memoryCleanupQueue._tasks.length > 0));
    }
    rows.push(fields.join('|'));
  }
  try {
    term.writeSync('normal\x1b[?1049halt');
    snapshot('initial');
    term.resize(narrow, 4);
    snapshot('shrink');
    assert.equal(term.buffers.alt._memoryCleanupQueue._tasks.length, 0);
    assert.equal(term.buffers.alt.lines.get(0).length, cols);
    term.resize(narrow + 1, 4);
    snapshot('grow');
    for (const b of [term.buffers.normal, term.buffers.alt]) {
      assert.equal(b._memoryCleanupQueue._tasks.length, 1);
      b._memoryCleanupQueue.flush();
    }
    snapshot('cleaned');
    return [idle ? 1 : 0, cols, scrollback, rows.join('^')].join('\t');
  } finally {
    term.dispose();
    callbacks.clear();
  }
}

const rows = [];
for (const idle of [false, true]) {
  for (const cols of [12, 80]) {
    for (const scrollback of [0, 10]) rows.push(run(idle, cols, scrollback));
  }
}
const fixture = rows.join('\n') + '\n';
const file = path.join(root, 'crates/xterm-core/tests/fixtures/buffer-cleanup.tsv');
if (process.argv.includes('--write')) await writeFile(file, fixture);
else assert.equal(await readFile(file, 'utf8'), fixture, 'source cleanup fixture drift');
const report = {
  status: 'passed', node: process.version, platform: process.platform, cases: rows.length,
  sourceHashes: hashes, fixtureSha256: hash(fixture), snapshots: rows,
  scope: 'Unchanged source headless terminal, real parser/buffers/queues, controlled idle and priority timer hosts. Not full source suite or browser proof.'
};
await writeFile(path.join(out, 'source.json'), JSON.stringify(report, null, 2) + '\n');
console.log(JSON.stringify({status: report.status, cases: report.cases, fixtureSha256: report.fixtureSha256}));

// Generate the Unicode boundary expectations from the unchanged source service.
import { readFile, mkdir, writeFile } from 'node:fs/promises';
import { stripTypeScriptTypes } from 'node:module';
import { runInNewContext } from 'node:vm';
import { createHash } from 'node:crypto';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { verifyOrGenerate } from './oracle-fixture.mjs';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const paths = ['Lifecycle.ts', 'Event.ts', 'services/UnicodeService.ts', 'input/UnicodeV6.ts'];
const sources = await Promise.all(paths.map(path => readFile(resolve(root, 'source_repo/src/common', path), 'utf8')));
const runtime = sources.map(source => stripTypeScriptTypes(
  source.replace(/^import .*;\r?\n/gm, '').replace(/^export /gm, ''), {mode: 'transform'})).join('\n');
const traceSource = await readFile(resolve(root, 'crates/xterm-web/tests/support/unicode_api.js'), 'utf8');
const {UnicodeService, UnicodeV6, unicodeTrace} = runInNewContext(
  runtime + '\n' + traceSource.replace(/^export /gm, '') + '\n({UnicodeService, UnicodeV6, unicodeTrace});');
const service = new UnicodeService();
service.register(new UnicodeV6());
const expected = JSON.stringify(unicodeTrace(service), null, 2) + '\n';
const fixture = resolve(root, 'crates/xterm-web/tests/fixtures/unicode-api.json');
await mkdir(dirname(fixture), {recursive: true});
await verifyOrGenerate(fixture, expected, process.argv.includes('--generate'));
const results = resolve(root, 'results/unicode-browser');
await mkdir(results, {recursive: true});
await writeFile(resolve(results, 'oracle.json'), JSON.stringify({
  status: 'passed', command: 'node scripts/unicode-browser-oracle.mjs',
  node: process.version, source: paths.map((path, index) => ({
    path: 'source_repo/src/common/' + path,
    sha256: createHash('sha256').update(sources[index]).digest('hex')
  })), snapshots: JSON.parse(expected).snapshots.length,
  note: 'Source-only expected values, not Rust/browser runtime evidence.'
}, null, 2) + '\n');

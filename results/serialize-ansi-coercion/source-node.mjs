// Supplemental source-only proof. DOM/browser equivalence remains separate.
import assert from 'node:assert/strict';
import { readFile, writeFile } from 'node:fs/promises';
import { createRequire } from 'node:module';
import { join } from 'node:path';
import { pathToFileURL } from 'node:url';
const root = process.cwd(), reference = process.env.SOURCE_ROOT;
assert(reference);
const require = createRequire(import.meta.url);
const { Terminal: Headless } = require(join(reference, 'out-esbuild/headless/public/Terminal.js'));
const { SerializeAddon } = await import(pathToFileURL(join(reference, 'addons/addon-serialize/lib/addon-serialize.mjs')));
class Terminal extends Headless { constructor(options) { super({ ...options, allowProposedApi: true }); } }
const document = { createElement() { return { style: {}, remove() {} }; }, body: { append() {} } };
const fixture = await readFile(join(root, 'scripts/serialize-browser-cases.mjs'), 'utf8');
const selected = fixture.replace('export async function serializationCases()', 'async function serializationCases()')
  .replace('async function run(name, body, opened = false) {', `async function run(name, body, opened = false) {
    if (!name.startsWith('ansi-scrollback-coercion-') && name !== 'ansi-scrollback-captured-missing-line') return;`);
const traces = await new Function('Terminal', 'SerializeAddon', 'document', `${selected}; return serializationCases();`)(Terminal, SerializeAddon, document);
assert.equal(traces.length, 6);
for (const entry of traces) assert.equal(entry.thrown, undefined);
assert(traces.find(entry => entry.name.endsWith('throws')).result.every(entry => entry.same));
const missing = traces.find(entry => entry.name === 'ansi-scrollback-captured-missing-line');
assert.deepEqual(missing.result.ansi, { error: 'TypeError', message: "Cannot read properties of undefined (reading 'isWrapped')" });
await writeFile(join(root, 'results/serialize-ansi-coercion/source-node.json'), JSON.stringify({
  platform: process.platform, node: process.version, scope: 'Original headless engine and original compiled serializer. Not Rust or browser proof.', traces
}, null, 2) + '\n');
console.log('Original headless five directed cases plus before-load passed; seven sentinel identities exact. Missing-line failure retained.');

// Run the unchanged original addon lifecycle, not a replacement Unicode host.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { createHash } from 'node:crypto';
const root = fileURLToPath(new URL('../../', import.meta.url));
const reference = path.resolve(process.env.SOURCE_ROOT ?? path.join(root, 'source_repo'));
const load = file => import(pathToFileURL(path.join(reference, file)).href);
const { Terminal } = await load('lib/xterm.mjs');
const { Unicode11Addon } = await load('addons/addon-unicode11/lib/addon-unicode11.mjs');
const { UnicodeGraphemesAddon } = await load('addons/addon-unicode-graphemes/lib/addon-unicode-graphemes.mjs');
const write = (terminal, text) => new Promise(resolve => terminal.write(text, resolve));
const results = [];
for (const [name, Addon, version] of [['unicode11', Unicode11Addon, '11'],
  ['unicode-graphemes', UnicodeGraphemesAddon, '15-graphemes']]) {
  const terminal = new Terminal({ allowProposedApi: true, cols: 12, rows: 4 });
  const previous = terminal.unicode.activeVersion;
  const cycles = [];
  try {
    for (let cycle = 0; cycle < 2; cycle++) {
      const addon = new Addon();
      terminal.loadAddon(addon);
      terminal.unicode.activeVersion = version;
      addon.dispose();
      const active = terminal.unicode.activeVersion;
      assert.equal(active, name === 'unicode11' ? '11' : previous);
      assert(terminal.unicode.versions.includes(active));
      terminal.reset();
      await write(terminal, '🧐');
      const width = [terminal.buffer.active.cursorX, terminal.buffer.active.getLine(0).getCell(0).getWidth()];
      assert.deepEqual(width, name === 'unicode11' ? [2, 2] : [1, 1]);
      cycles.push({ cycle, active, width, versions: terminal.unicode.versions });
    }
    let oldAssertion;
    try {
      assert.equal(terminal.unicode.activeVersion, previous);
      oldAssertion = { status: 'passed' };
    } catch (error) {
      assert.equal(name, 'unicode11');
      oldAssertion = { status: 'rejected', name: error.name, message: error.message };
    }
    results.push({ name, previous, cycles, oldAssertion });
  } finally { terminal.dispose(); }
}
const files = ['addons/addon-unicode11/src/Unicode11Addon.ts',
  'addons/addon-unicode-graphemes/src/UnicodeGraphemesAddon.ts', 'lib/xterm.mjs',
  'addons/addon-unicode11/lib/addon-unicode11.mjs',
  'addons/addon-unicode-graphemes/lib/addon-unicode-graphemes.mjs'];
const pins = [];
for (const file of files) {
  const bytes = await fs.readFile(path.join(reference, file));
  pins.push({ path: file, bytes: bytes.length, sha256: createHash('sha256').update(bytes).digest('hex') });
}
const report = { status: 'passed', node: process.version, platform: process.platform, results, pins,
  scope: 'Actual original browser Terminal in Node, unopened; not Rust/browser E2E',
  oldAssertionRejectedOnlyUnicode11: true };
await fs.writeFile(new URL('unicode-disposal-source-assertions.json', import.meta.url), JSON.stringify(report, null, 2) + '\n');
console.log(JSON.stringify(report));

// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
// Public reset inside marker disposal, executed by the unchanged source parser.
import { createRequire } from 'node:module';
import { mkdir, writeFile } from 'node:fs/promises';
import { resolve } from 'node:path';
const root = resolve(import.meta.dirname, '..');
const source = resolve(process.env.SOURCE_REPO ?? resolve(root, 'source_repo'));
const require = createRequire(resolve(source, 'package.json'));
const { build } = require('esbuild');
const temporary = resolve(root, 'tmp/marker-csi');
await mkdir(temporary, { recursive: true });
const bundle = resolve(temporary, 'Terminal.cjs');
await build({ entryPoints: [resolve(source, 'src/headless/public/Terminal.ts')],
  outfile: bundle, bundle: true, platform: 'node', format: 'cjs',
  tsconfig: resolve(source, 'src/headless/tsconfig.json') });
const { Terminal } = require(bundle);
const cases = [['0J', 1], ['1J', -1], ['2J', 0], ['?2J', 0], ['3J', -5],
  ['L', 0], ['M', 0], ['S', -2], ['T', 0], ['^', 0]];
const results = [];
for (const [command, offset] of cases) {
  const terminal = new Terminal({ cols: 5, rows: 3, scrollback: 10, allowProposedApi: true });
  const parse = input => terminal._core._inputHandler.parse(input);
  parse('0\r\n1\r\n2\r\n3\r\n4\r\n5\r\n6\r\n7');
  if (command === '0J') parse('\x1b[H');
  const marker = terminal.registerMarker(offset);
  let calls = 0;
  marker.onDispose(() => { calls++; terminal.reset(); });
  parse('\x1b[' + command);
  const buffer = terminal._core._bufferService.buffer;
  results.push({ command, calls, x: buffer.x, y: buffer.y, ybase: buffer.ybase,
    length: buffer.lines.length, rows: Array.from({ length: buffer.lines.length },
      (_, index) => buffer.lines.get(index)?.translateToString(true) ?? null) });
  terminal.dispose();
}
await mkdir(resolve(root, 'results/csi-marker'), { recursive: true });
await writeFile(resolve(root, 'results/csi-marker/source.json'), JSON.stringify(results, null, 2) + '\n');
console.log(`Source marker reset: ${results.length} cases`);

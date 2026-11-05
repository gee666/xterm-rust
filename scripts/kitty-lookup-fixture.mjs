// Licensed under MIT. Unchanged source bodies are test-only, never binding glue.
import assert from 'node:assert/strict';
import { readFileSync, writeFileSync } from 'node:fs';
import { stripTypeScriptTypes } from 'node:module';
import { resolve } from 'node:path';
const root = resolve(import.meta.dirname, '..');
const read = name => readFileSync(resolve(root, name), 'utf8');
const erase = text => stripTypeScriptTypes(text.replace(/^import .*;\r?$/gm, '').replace(/^export /gm, ''), { mode: 'transform' });
const resultEnum = read('source_repo/src/common/Types.ts').match(/export const enum KeyboardResultType\s*\{[^}]*\}/)?.[0];
assert.ok(resultEnum);
const output = '// Copyright (c) 2025 The xterm.js authors. All rights reserved.\n' +
  '// Licensed under MIT. Generated test-only unchanged upstream body; exceeds\n' +
  '// 350 lines to preserve source auditability. Regenerate with kitty-lookup-fixture.mjs.\n' +
  erase(resultEnum) + '\n' + erase(read('source_repo/src/common/data/EscapeSequences.ts')) + '\n' +
  erase(read('source_repo/src/common/input/KittyKeyboard.ts')) + '\n' +
  read('crates/xterm-web/tests/fixtures/kitty_lookup_cases.mjs').replace(/^export /gm, '') + '\n' +
  read('scripts/kitty-lookup-driver.js');
const file = resolve(root, 'crates/xterm-web/tests/fixtures/kitty_lookup_driver.js');
if (process.argv.includes('--write')) writeFileSync(file, output);
else assert.equal(readFileSync(file, 'utf8'), output, 'Kitty source fixture drift.');
console.log('Kitty fixture matches unchanged source bodies.');

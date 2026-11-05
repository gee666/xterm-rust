// Licensed under MIT. Original source is test-only, never production glue.
import assert from 'node:assert/strict';
import { readFileSync, writeFileSync } from 'node:fs';
import { stripTypeScriptTypes } from 'node:module';
import { resolve } from 'node:path';
const root = resolve(import.meta.dirname, '..');
const read = name => readFileSync(resolve(root, name), 'utf8');
const erase = text => stripTypeScriptTypes(text.replace(/^import .*;\r?$/gm, '').replace(/^export /gm, ''), { mode: 'transform' });
const resultEnum = read('source_repo/src/common/Types.ts').match(/export const enum KeyboardResultType\s*\{[^}]*\}/)?.[0];
assert.ok(resultEnum);
const source = '// Copyright (c) 2026 The xterm.js authors. All rights reserved.\n' +
  '// Licensed under MIT. Generated test-only upstream fixture. The cohesive\n' +
  '// source body exceeds 350 lines so its assertions stay auditable.\n' +
  read('crates/xterm-web/tests/fixtures/win32-prototype-cases.mjs').replace(/^export /gm, '') + '\n' +
  erase(resultEnum) + '\n' + erase(read('source_repo/src/common/data/EscapeSequences.ts')) + '\n' +
  erase(read('source_repo/src/common/input/Win32InputMode.ts')) + '\n' +
  read('crates/xterm-web/tests/fixtures/win32_independent_cases.mjs').replace(/^export /gm, '') + '\n' +
  read('scripts/win32-prototype-driver.js');
const file = resolve(root, 'crates/xterm-web/tests/fixtures/win32_prototype_driver.js');
if (process.argv.includes('--write')) writeFileSync(file, source);
else assert.equal(readFileSync(file, 'utf8'), source, 'Win32 source fixture is stale.');
console.log('Win32 prototype fixture matches unchanged source bodies.');

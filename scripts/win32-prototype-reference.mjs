#!/usr/bin/env node
// Licensed under MIT. Reference execution only; task184 still supplies Rust behavior.
import assert from 'node:assert/strict';
import { readFileSync, writeFileSync, mkdirSync } from 'node:fs';
import { stripTypeScriptTypes } from 'node:module';
import { resolve } from 'node:path';
import vm from 'node:vm';
import { prototypeCases, capturePrototypeCase } from './win32-prototype-cases.mjs';

const root = resolve(import.meta.dirname, '..');
const flags = new Set(process.argv.slice(2));
assert.ok(flags.has('--generate-reference'), 'Use --generate-reference. This script does not implement a Rust comparison.');
assert.ok([...flags].every(flag => ['--generate-reference', '--browsers'].includes(flag)), 'Unknown flag');
const read = name => readFileSync(resolve(root, name), 'utf8');
const erase = text => stripTypeScriptTypes(text.replace(/^import .*;\r?$/gm, '').replace(/^export /gm, ''), { mode: 'transform' });
const resultEnum = read('source_repo/src/common/Types.ts').match(/export const enum KeyboardResultType\s*\{[^}]*\}/)?.[0];
assert.ok(resultEnum);
const program = erase(resultEnum) + '\n' + erase(read('source_repo/src/common/data/EscapeSequences.ts')) + '\n' +
  erase(read('source_repo/src/common/input/Win32InputMode.ts')) +
  `\n(${capturePrototypeCase.toString()})(Win32InputMode,globalThis.spec)`;
const output = resolve(root, 'results/win32-prototype-reference');
mkdirSync(output, { recursive: true });
const captures = [];
for (const spec of prototypeCases) captures.push(vm.runInNewContext(program, { spec }, { timeout: 5000 }));
assert.equal(captures.length, prototypeCases.length);
const text = id => String.fromCharCode(...captures.find(row => row.caseId === id).results[0].returned.key.units);
assert.equal(text('primitive-undefined-1-0'), '\x1b[65;0;0;1;0;1_');
assert.equal(text('primitive-false-1-0'), '\x1b[false;0;false;1;0;1_');
assert.equal(text('late-object-0'), '\x1b[late-vk;late-sc;late-uc;1;0;1_');
const isolated = captures.find(row => row.caseId === 'second-encoder-isolation-0');
assert.equal(String.fromCharCode(...isolated.results[1].returned.key.units), '\x1b[65;30;97;1;0;1_');
writeFileSync(resolve(output, 'node.json'), JSON.stringify({ scope: 'Original JS reference only. Rust/native/browser comparison remains task184/185.',
  node: process.version, cases: captures.length, captures }, null, 2) + '\n');
console.log(`Node ${process.version}: ${captures.length} original Win32 prototype reference cases; Rust comparison unrun.`);
if (flags.has('--browsers')) {
  const { pathToFileURL } = await import('node:url');
  const playwright = await import(pathToFileURL(resolve(root, 'source_repo/node_modules/playwright/index.mjs')));
  for (const name of ['chromium', 'firefox', 'webkit']) {
    const browser = await playwright[name].launch({ headless: true, timeout: 30000 });
    try {
      const page = await browser.newPage();
      const rows = [];
      for (const spec of prototypeCases) {
        rows.push(await page.evaluate(({ program, spec }) => {
          globalThis.spec = spec;
          try { return (0, eval)(program); }
          finally { delete globalThis.spec; }
        }, { program, spec }));
      }
      writeFileSync(resolve(output, `${name}.json`), JSON.stringify({ scope: 'Original JS browser reference only; no Rust implementation executes here.',
        engine: name, version: browser.version(), cases: rows.length, captures: rows }, null, 2) + '\n');
      console.log(`${name} ${browser.version()}: ${rows.length} original reference cases; Rust comparison unrun.`);
    } finally { await browser.close(); }
  }
}

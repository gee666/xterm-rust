// Copyright (c) 2018 The xterm.js authors. All rights reserved. Licensed under MIT.
// Retain every literal expectation in the upstream escape sequence examples.
import assert from 'node:assert/strict';
import { readFile, writeFile, mkdir } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { stripTypeScriptTypes } from 'node:module';
import { spawnSync } from 'node:child_process';
import { homedir } from 'node:os';
import path from 'node:path';

const sourcePath = 'source_repo/src/common/parser/EscapeSequenceParser.test.ts';
const source = await readFile(sourcePath, 'utf8');
const start = source.indexOf("  describe('escape sequence examples'");
const end = source.indexOf("  describe('coverage tests'", start);
assert(start > 0 && end > start);
const section = stripTypeScriptTypes(source.slice(start, end), { mode: 'transform' });
const cases = [];
let current;
let clear = false;
const describe = (_name, callback) => callback();
const it = (name, callback) => { current = { name, steps: [] }; cases.push(current); callback(); };
const test = (input, expected, noReset) => {
  current.steps.push({ input, expected, reset: !noReset, clear: !noReset || clear });
  clear = false;
};
const testTerminal = { clear() { clear = true; } };
// Execute only literal expected-value declarations and the capture shim, never
// a substitute terminal. Actual upstream assertions execute in official baseline.
Function('describe', 'it', 'test', 'testTerminal', section)(describe, it, test, testTerminal);
function rs(text) {
  let result = '"';
  for (const char of text) {
    const code = char.codePointAt(0);
    if (char === '"' || char === '\\') result += '\\' + char;
    else if (code < 32 || code === 127) result += `\\u{${code.toString(16)}}`;
    else result += char;
  }
  return result + '"';
}
function params(values) {
  return 'vec![' + values.map(value => Array.isArray(value)
    ? `V::SubParams(vec![${value.join(',')}])` : `V::Param(${value})`).join(',') + ']';
}
function event(value) {
  const [type, a, b, c] = value;
  switch (type) {
    case 'print': return `Event::Print(${rs(a)}.into())`;
    case 'exe': return `Event::Execute(${a.charCodeAt(0)})`;
    case 'esc': return `Event::Esc(${rs(a + b)}.into())`;
    case 'csi': return `Event::Csi(${rs(a + c)}.into(),${params(b)})`;
    case 'osc': return `Event::Osc(${rs(a)}.into())`;
    case 'dcs hook': return `Event::DcsHook(${params(a)})`;
    case 'dcs put': return `Event::DcsPut(${rs(a)}.into())`;
    case 'dcs unhook': return `Event::DcsEnd(${a})`;
    case 'apc start': return 'Event::ApcStart';
    case 'apc put': return `Event::ApcPut(${rs(a)}.into())`;
    case 'apc end': return `Event::ApcEnd(${a})`;
    default: throw new Error(`unrecognized expected event ${type}`);
  }
}
const lines = ['// Copyright (c) 2018 The xterm.js authors. All rights reserved.',
  '// Licensed under MIT. Generated from original literal example assertions.', 'use super::*;', ''];
for (const [index, testCase] of cases.entries()) {
  lines.push(`// Original example: ${testCase.name}`, '#[test]', `fn example_${index.toString().padStart(2, '0')}() {`, '    let mut fixture = Fixture::new();');
  for (const step of testCase.steps) {
    if (step.reset) lines.push('    fixture.parser.reset().unwrap();');
    if (step.clear) lines.push('    fixture.clear();');
    lines.push(`    fixture.parse(${rs(step.input)});`, `    assert_eq!(fixture.events(), vec![${step.expected.map(event).join(',')}]);`);
  }
  lines.push('}', '');
}
const rustfmt = process.env.RUSTFMT ?? path.join(homedir(), '.cargo', 'bin', process.platform === 'win32' ? 'rustfmt.exe' : 'rustfmt');
const formatted = spawnSync(rustfmt, ['--edition', '2021', '--emit', 'stdout'], { input: lines.join('\n'), encoding: 'utf8', timeout: 30000 });
if (formatted.error) throw formatted.error;
assert.equal(formatted.status, 0, formatted.stderr);
const generated = formatted.stdout.replace(/\r\n/g, '\n');
const destination = 'crates/xterm-core/tests/escape_upstream_support/examples.rs';
if (process.argv.includes('--generate')) {
  await mkdir('crates/xterm-core/tests/escape_upstream_support', { recursive: true });
  await writeFile(destination, generated);
} else {
  assert.equal((await readFile(destination, 'utf8')).replace(/\r\n/g, '\n'), generated);
}
await mkdir('results/escape-upstream', { recursive: true });
const summary = { sourcePath, sourceSha256: createHash('sha256').update(source).digest('hex'),
  generatedTests: cases.length, exampleSteps: cases.reduce((sum, c) => sum + c.steps.length, 0),
  expectedEvents: cases.reduce((sum, c) => sum + c.steps.reduce((n, s) => n + s.expected.length, 0), 0),
  status: process.argv.includes('--generate') ? 'generated' : 'matched' };
await writeFile('results/escape-upstream/examples.json', JSON.stringify(summary, null, 2) + '\n');
console.log(JSON.stringify(summary));

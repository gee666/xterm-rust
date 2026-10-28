// Copyright (c) 2018-2025 The xterm.js authors. All rights reserved.
// Licensed under MIT. APC traces through the unchanged complete escape parser.
import assert from 'node:assert/strict';
import { readFile, writeFile, mkdir } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { escapeReference } from './escape-reference.mjs';

const reference = await escapeReference();
const { EscapeSequenceParser } = reference.context;
const hex = values => Array.from(values, value => value.toString(16)).join(',');
const units = text => hex(Array.from({ length: text.length }, (_, i) => text.charCodeAt(i)));
const params = value => value.toArray().map(v => Array.isArray(v) ? `s${v.join(',')}` : `p${v}`).join(':');
const text = value => Array.from(value, char => char.codePointAt(0));
const sequences = [
  text('A\x1b_Gpayload\x1b\\Z'), text('\x9fGpayload\x9cZ'),
  text('\x1b_+Gpayload\x1b\\Z'), text('\x1b_ +Gpayload\x9cZ'),
  text('\x1b_0payload\x9cZ'), text('\x1b_Hunhandled\x9cZ'),
  text('\x1b_G\x9cZ'), text('\x1b_Gpartial\x18Z'), text('\x1b_Gpartial\x1aZ'),
  text('\x1b_Gpartial\x1b[31mZ'),
  [...text('\x1b_G'), 0, 7, 8, 9, 10, 11, 12, 13, 14, 31, 32, 127, 65, 0x9c, 90],
  [...text('\x1b_G'), 0x1f600, 0xd800, 0x10ffff, 0xffffffff, 0x9c, 90],
  text('\x1b_Gunterminated'), text('\x1b_'),
];
const configurations = ['-', 'h:0:1', 'a:1:1,b:0:0,c:1:1', 'a:0:0,b:1:0,c:0:0'];
const rows = [];
let parseCalls = 0;
let inputChunks = 0;
let pauses = 0;
for (let sequence = 0; sequence < sequences.length; sequence++) {
  for (const width of [0, 1, 2, 3, 7]) {
    for (const configuration of configurations) {
      const parser = new EscapeSequenceParser();
      const log = [];
      const operations = [];
      const snapshots = [];
      parser.setPrintHandler((data, start, end) => log.push(`P:${hex(data.slice(start, end))}`));
      parser.setExecuteHandlerFallback(code => log.push(`X:${code}`));
      parser.setEscHandlerFallback(id => log.push(`E:${id >>> 0}`));
      parser.setCsiHandlerFallback((id, value) => log.push(`C:${id >>> 0}:${params(value)}`));
      parser.setApcHandlerFallback((id, action, data) => log.push(`F:${id >>> 0}:${action === 'START' ? 's' : action === 'PUT' ? `p:${units(data)}` : `e:${+data}`}`));
      if (configuration !== '-') {
        for (const [intermediates, final] of [[undefined, 'G'], ['+', 'G'], [' +', 'G'], [undefined, '0']]) {
          const id = [...(intermediates ?? ''), final].reduce((value, char) => value * 256 + char.charCodeAt(0), 0);
          for (const entry of configuration.split(',')) {
            const [name, asynchronous, handled] = entry.split(':');
            parser.registerApcHandler({ intermediates, final }, {
              start() { log.push(`H:${id}:${name}`); },
              put(data, start, end) { log.push(`D:${id}:${name}:${units(reference.utf32String(Array.from(data.slice(start, end))))}`); },
              end(success) {
                const complete = () => { log.push(`U:${id}:${name}:${+success}`); return handled === '1'; };
                return asynchronous === '1' ? reference.resolvedPromise(complete) : complete();
              },
            });
          }
        }
      }
      const data = sequences[sequence];
      const step = width || data.length;
      const chunks = [];
      for (let offset = 0; offset < data.length; offset += step) chunks.push(data.slice(offset, offset + step));
      for (const chunk of chunks) {
        operations.push(`p:${hex(chunk)}`);
        const input = Uint32Array.from(chunk);
        let previous;
        let pending;
        const parse = () => { parseCalls++; return parser.parse(input, input.length, previous); };
        while ((pending = parse())) {
          log.push(`Y:${parser.currentState}:${parser._parseStack.chunkPos}`);
          pauses++;
          previous = await pending;
        }
        inputChunks++;
        snapshots.push(`${parser.currentState}/${parser._collect >>> 0}/${params(parser._params)}/${parser.precedingJoinState}`);
      }
      operations.push('x');
      parser.reset();
      // Reset initiates JS promises eagerly. Explicitly flush their jobs before
      // comparing with Rust's executor-drained cleanup queue.
      await reference.resolvedPromise(() => {});
      snapshots.push(`${parser.currentState}/${parser._collect >>> 0}/${params(parser._params)}/${parser.precedingJoinState}`);
      rows.push([`s${sequence}-w${width}`, configuration, operations.join(';'), snapshots.join('~'), log.join('|')].join('\t'));
    }
  }
}
const corpus = rows.join('\n') + '\n';
const fixture = 'crates/xterm-core/tests/fixtures/apc-escape.tsv';
await mkdir('crates/xterm-core/tests/fixtures', { recursive: true });
if (process.argv.includes('--generate')) await writeFile(fixture, corpus);
else assert.equal((await readFile(fixture, 'utf8')).replace(/\r\n/g, '\n'), corpus, 'APC escape integration oracle drift');
const controls = [];
for (const kind of ['throw-sync', 'throw-async', 'reset-pending']) {
  const parser = new EscapeSequenceParser();
  const log = [];
  parser.setPrintHandler((data, start, end) => log.push(`P:${hex(data.slice(start, end))}`));
  for (const name of ['a', 'b']) {
    parser.registerApcHandler({ final: 'G' }, {
      start() { log.push(`${name}:S`); },
      put(data, start, end) { log.push(`${name}:P:${units(reference.utf32String(Array.from(data.slice(start, end))))}`); },
      end(success) {
        const complete = () => {
          log.push(`${name}:E:${+success}`);
          if (name === 'b' && kind.startsWith('throw')) throw new Error('callback failure');
          return true;
        };
        return kind === 'throw-sync' ? complete() : reference.resolvedPromise(complete);
      },
    });
  }
  const input = Uint32Array.from(text('\x1b_Gdata\x9cZ'));
  let error;
  try {
    const pending = parser.parse(input, input.length);
    if (pending) {
      log.push(`Y:${parser.currentState}:${parser._parseStack.chunkPos}`);
      if (kind === 'reset-pending') parser.reset();
      const previous = await pending;
      await reference.resolvedPromise(() => {});
      assert.equal(parser.parse(input, input.length, previous), undefined);
    }
  } catch (caught) { error = caught.name; }
  assert.equal(error, kind === 'reset-pending' ? undefined : 'Error');
  const state = `${parser.currentState}/${parser._collect >>> 0}/${params(parser._params)}/${parser.precedingJoinState}/${parser._parseStack.state ? 'paused' : 'none'}`;
  controls.push([kind, state, log.join('|'), error ?? 'none'].join('\t'));
}
const controlCorpus = controls.join('\n') + '\n';
const controlFixture = 'crates/xterm-core/tests/fixtures/apc-escape-control.tsv';
if (process.argv.includes('--generate')) await writeFile(controlFixture, controlCorpus);
else assert.equal((await readFile(controlFixture, 'utf8')).replace(/\r\n/g, '\n'), controlCorpus, 'APC control/error oracle drift');
const report = { node: process.version, platform: process.platform, sources: reference.sources,
  scenarios: sequences.length, configurations: configurations.length, chunkWidths: [0, 1, 2, 3, 7],
  traces: rows.length, inputChunks, parseCalls, pauses, fixtureSha256: createHash('sha256').update(corpus).digest('hex'),
  controlCases: controls.length, controlFixtureSha256: createHash('sha256').update(controlCorpus).digest('hex'),
  status: process.argv.includes('--generate') ? 'generated' : 'matched',
  scope: 'actual escape parser, APC registrations/fallback, control/Unicode chunks, ST/CAN/SUB, async resume and reset cleanup; no terminal/browser claim' };
await mkdir('results/apc-escape', { recursive: true });
await writeFile('results/apc-escape/oracle.json', JSON.stringify(report, null, 2) + '\n');
console.log(JSON.stringify(report, null, 2));

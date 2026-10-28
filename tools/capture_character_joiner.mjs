// Execute the original joiner assertions without changing their expectations.
import fs from 'node:fs';
import path from 'node:path';
import { createRequire } from 'node:module';
import { createHash } from 'node:crypto';
import { captureCells, utf16 } from './render_fixture_helpers.mjs';
const require = createRequire(path.resolve('source_repo/package.json'));
const Mocha = require('mocha');
const base = path.resolve('source_repo/out-esbuild');
const { CharacterJoinerService } = require(path.join(base, 'browser/services/CharacterJoinerService.js'));
const { CellData } = require(path.join(base, 'common/buffer/CellData.js'));
const calls = [];
let active;
const register = CharacterJoinerService.prototype.register;
CharacterJoinerService.prototype.register = function(handler) {
  let id;
  id = register.call(this, text => {
    const result = handler(text);
    if (active) active.handlerCalls.push({ id, text: utf16(text), ranges: result.map(range => [...range]) });
    return result;
  });
  return id;
};
const get = CharacterJoinerService.prototype.getJoinedCharacters;
CharacterJoinerService.prototype.getJoinedCharacters = function(row) {
  const line = this._bufferService.buffer.lines.get(row);
  const previous = active;
  const record = { cells: line ? captureCells(line, CellData) : null,
    handlers: this._characterJoiners.map(joiner => joiner.id), handlerCalls: [] };
  active = record;
  try { record.expected = get.call(this, row); }
  finally { active = previous; }
  calls.push(record);
  return record.expected;
};
const mocha = new Mocha({ reporter: 'dot', grep: /CharacterJoinerService/ });
mocha.addFile(path.join(base, 'browser/services/CharacterJoinerService.test.js'));
const failed = await new Promise(resolve => mocha.run(resolve));
if (failed || !calls.length) throw new Error(`Joiner baseline failed=${failed}, calls=${calls.length}`);
const source = 'source_repo/src/browser/services/CharacterJoinerService.test.ts';
const sha256 = createHash('sha256').update(fs.readFileSync(source)).digest('hex');
fs.mkdirSync('crates/xterm-web/tests/fixtures', { recursive: true });
fs.writeFileSync('crates/xterm-web/tests/fixtures/character-joiner.json', JSON.stringify({ source, sha256, calls }) + '\n');
fs.mkdirSync('results/renderer', { recursive: true });
fs.writeFileSync('results/renderer/joiner-original.json', JSON.stringify({ source, sha256, calls: calls.length, failed }, null, 2) + '\n');
console.log(JSON.stringify({ source, calls: calls.length, failed }));

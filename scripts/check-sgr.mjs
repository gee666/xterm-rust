// Scoped SGR assertion bodies and state oracle. Does not execute a terminal demo.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { createRequire } from 'node:module';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';

process.chdir(resolve(import.meta.dirname, '..'));
const require = createRequire(resolve('source_repo/package.json'));
const esbuild = require('esbuild');
const ts = require('typescript');
mkdirSync('tmp/sgr', { recursive: true });
mkdirSync('results/sgr', { recursive: true });
const entry = `export { InputHandler } from '../../source_repo/src/common/InputHandler';
export { Params } from '../../source_repo/src/common/parser/Params';
export { AttributeData, ExtendedAttrs } from '../../source_repo/src/common/buffer/AttributeData';
export { Attributes, BgFlags, UnderlineStyle } from '../../source_repo/src/common/buffer/Constants';`;
writeFileSync('tmp/sgr/entry.ts', entry);
await esbuild.build({ entryPoints: ['tmp/sgr/entry.ts'], outfile: 'tmp/sgr/original.mjs',
  bundle: true, platform: 'node', format: 'esm', tsconfig: 'source_repo/src/common/tsconfig.json',
  nodePaths: [resolve('source_repo/src')] });
const { InputHandler, Params, AttributeData, ExtendedAttrs, Attributes, BgFlags, UnderlineStyle } =
  await import(pathToFileURL(resolve('tmp/sgr/original.mjs')));
const rows = [];
let activeCase = 'directed';
const caseCounts = {};
function paramsFor(body) {
  assert.match(body, /^[0-9;:]*$/);
  const params = new Params();
  params.addParam(0);
  for (const char of body) {
    if (char === ';') params.addParam(0);
    else if (char === ':') params.addSubParam(-1);
    else params.addDigit(Number(char));
  }
  return params;
}
function bits(attr) {
  return [attr.fg >>> 0, attr.bg >>> 0, attr.extended._ext >>> 0,
    attr.extended.urlId, Number(attr.extended.payload !== undefined)];
}
function step(handler, body) {
  const before = bits(handler._curAttrData);
  const identity = handler._curAttrData.extended;
  const unknown = [];
  handler._logService = { debug: (_format, value) => unknown.push(value) };
  const kitty = handler._optionsService.rawOptions.vtExtensions.kittySgrBoldFaintControl;
  assert.equal(handler.charAttributes(paramsFor(body)), true);
  rows.push([...before, Number(kitty), body, ...bits(handler._curAttrData),
    Number(identity === handler._curAttrData.extended), unknown.join(',')].join('\t'));
  caseCounts[activeCase] = (caseCounts[activeCase] ?? 0) + 1;
}
function makeHandler() {
  // Constructor-free state is limited to the fields these unchanged methods use.
  // CSI tokenization here is test plumbing, not an original parser baseline run.
  const handler = Object.create(InputHandler.prototype);
  handler._curAttrData = new AttributeData();
  Object.defineProperty(handler, 'curAttrData', { get: () => handler._curAttrData });
  handler._optionsService = { rawOptions: { vtExtensions: { kittySgrBoldFaintControl: true } } };
  handler.parseP = async text => {
    const matches = [...text.matchAll(/\x1b\[([0-9;:]*)m/g)];
    assert.equal(matches.map(match => match[0]).join(''), text, 'adapter accepts SGR only');
    for (const match of matches) step(handler, match[1]);
  };
  return handler;
}
const originalTests = readFileSync('source_repo/src/common/InputHandler.test.ts', 'utf8');
const start = originalTests.indexOf("  describe('text attributes',");
const end = originalTests.indexOf("  describe('cursor positioning',", start);
assert(start >= 0 && end > start, 'exact scoped assertion sections must exist');
const selected = originalTests.slice(start, end);
const cases = [];
const contexts = [];
let assertionCount = 0;
const countedAssert = {
  equal: (...args) => { assertionCount++; assert.equal(...args); },
  deepEqual: (...args) => { assertionCount++; assert.deepEqual(...args); }
};
const describe = (name, callback) => {
  contexts.push({ name, hooks: [] }); callback(); contexts.pop();
};
const beforeEach = callback => contexts.at(-1).hooks.push(callback);
const it = (name, callback) => cases.push({ name: [...contexts.map(c => c.name), name].join(' / '),
  hooks: contexts.flatMap(c => c.hooks), callback });
const inputHandler = makeHandler();
class TestInputHandler { constructor() { return makeHandler(); } }
class UnusedServiceArgument {}
const arguments_ = { describe, beforeEach, it, assert: countedAssert, inputHandler, TestInputHandler,
  bufferService: {}, coreService: {}, optionsService: {}, AttributeData, Attributes,
  MockCharsetService: UnusedServiceArgument, MockLogService: UnusedServiceArgument,
  MockOscLinkService: UnusedServiceArgument, MockMouseStateService: UnusedServiceArgument,
  MockUnicodeService: UnusedServiceArgument };
const body = ts.transpileModule(selected, { compilerOptions: { target: ts.ScriptTarget.ES2022 } }).outputText;
new Function(...Object.keys(arguments_), body)(...Object.values(arguments_));
for (const item of cases) {
  activeCase = item.name;
  inputHandler._curAttrData = new AttributeData();
  for (const hook of item.hooks) await hook();
  await item.callback();
}
assert.equal(cases.length, 35, 'scoped original case count changed and needs review');
assert.equal(assertionCount, 1162, 'scoped original assertion count changed and needs review');
const originalAssertionRows = rows.length;
assert.equal(originalAssertionRows, 332, 'scoped original step count changed and needs review');
activeCase = 'directed boundaries';
for (const kitty of [false, true]) {
  for (const body of ['', '1;2;221;222', '6;999;221;222', '38;0;0;0;0;0:5',
    '38:2:1:2:3', '38:2::256:257:511;4', '48:5:511;53', '58:2::1:2:3',
    '58;0', '59', '0;4:0;21;24']) {
    const handler = makeHandler();
    handler._optionsService.rawOptions.vtExtensions.kittySgrBoldFaintControl = kitty;
    handler._curAttrData.fg = 0xffffffff;
    handler._curAttrData.bg = 0xffffffff;
    handler._curAttrData.extended = new ExtendedAttrs(0xffffffff, 9);
    handler._curAttrData.extended.payload = {};
    step(handler, body);
  }
  for (let value = 0; value < 256; value++) {
    const handler = makeHandler();
    handler._optionsService.rawOptions.vtExtensions.kittySgrBoldFaintControl = kitty;
    step(handler, `${value}`);
    step(handler, `38;5;${value};48:5:${value};4:${value};58;2;${value};${255-value};${value}`);
  }
}
const seed = 0x53475231;
let state = seed;
const random = () => { state ^= state << 13; state ^= state >>> 17; state ^= state << 5; return state >>> 0; };
const choices = [0,1,2,3,4,5,6,7,8,9,21,22,23,24,25,27,28,29,30,37,38,39,40,47,48,49,53,55,58,59,90,97,100,107,221,222,255,256,511,2147483647];
activeCase = 'seeded state transitions';
for (let index = 0; index < 12000; index++) {
  const handler = makeHandler();
  handler._curAttrData.fg = random(); handler._curAttrData.bg = random();
  handler._curAttrData.extended = new ExtendedAttrs(random(), random() % 4);
  if (random() & 1) handler._curAttrData.extended.payload = {};
  handler._optionsService.rawOptions.vtExtensions.kittySgrBoldFaintControl = Boolean(random() & 1);
  const length = random() % 32;
  let body = '';
  for (let position = 0; position < length; position++) {
    if (position) body += (random() % 3 ? ';' : ':');
    if (random() % 5) body += choices[random() % choices.length];
  }
  step(handler, body);
}
const fixture = rows.join('\n') + '\n';
const path = 'crates/xterm-core/tests/fixtures/sgr-oracle.tsv';
if (process.argv.includes('--write-fixtures')) writeFileSync(path, fixture);
else assert.equal(readFileSync(path, 'utf8').replaceAll('\r\n', '\n'), fixture);
const sha256 = data => createHash('sha256').update(data).digest('hex');
const summary = { scope: 'SGR methods and scoped source assertions only, not complete InputHandler or parser baseline',
  originalCases: cases.length, originalAssertions: assertionCount, originalAssertionRows,
  cases: caseCounts, comparedRows: rows.length, seed, node: process.version, esbuild: esbuild.version,
  sourceSha256: sha256(readFileSync('source_repo/src/common/InputHandler.ts')),
  assertionSourceSha256: sha256(selected), fixtureSha256: sha256(fixture), status: 'passed' };
writeFileSync('results/sgr/oracle.json', JSON.stringify(summary, null, 2) + '\n');
console.log(JSON.stringify(summary, null, 2));

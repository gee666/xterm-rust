// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
// Execute unchanged upstream method bodies. This is not browser-suite evidence.
import assert from 'node:assert/strict';
import { readFile, writeFile, mkdir } from 'node:fs/promises';
import { stripTypeScriptTypes } from 'node:module';
import { createHash } from 'node:crypto';
const hashes = {};
async function load(path, replacements = [], suffix = '') {
  const original = await readFile(path, 'utf8');
  hashes[path] = createHash('sha256').update(original).digest('hex');
  let source = original;
  for (const [pattern, replacement] of replacements) {
    assert.match(source, pattern);
    source = source.replace(pattern, replacement);
  }
  const js = stripTypeScriptTypes(source + suffix, { mode: 'transform' });
  return import(`data:text/javascript;base64,${Buffer.from(js).toString('base64')}`);
}
const root = 'source_repo/addons/';
const { Base64, ClipboardAddon } = await load(root + 'addon-clipboard/src/ClipboardAddon.ts', [[/^import \{ type IClipboardProvider, type IBase64 \} from '@xterm\/addon-clipboard';\r?$/m, '']]);
const { FitAddon } = await load(root + 'addon-fit/src/FitAddon.ts', [[/^import \{ ViewportConstants \} from 'browser\/shared\/Constants';\r?$/m, 'const ViewportConstants = { DEFAULT_SCROLL_BAR_WIDTH: 14 };']]);
const { LinkComputer, isUrl } = await load(root + 'addon-web-links/src/WebLinkProvider.ts', [[/^import \{ ILinkProvider, ILink, Terminal, IViewportRange, IBufferLine \} from '@xterm\/xterm';\r?$/m, '']], '\nexport { isUrl };');
const addonSource = await readFile(root + 'addon-web-links/src/WebLinksAddon.ts', 'utf8');
hashes[root + 'addon-web-links/src/WebLinksAddon.ts'] = createHash('sha256').update(addonSource).digest('hex');
const regexLiteral = addonSource.match(/^const strictUrlRegex = (.+);\r?$/m)[1];
const regex = Function(`return ${regexLiteral}`)();
const u16 = text => Array.from({ length: text.length }, (_, i) => text.charCodeAt(i));
const text = units => String.fromCharCode(...units);
const seed = 0x42555449;
let rng = seed;
function next() { rng ^= rng << 13; rng ^= rng >>> 17; rng ^= rng << 5; return rng >>> 0; }
const codec = new Base64();
const base64 = [];
const enc = ['', 'hello world', '€mläütß', '\ufeffa', '\ud800', '\udc00', '🦀', '\0'];
for (let i = 0; i < 256; i++) enc.push(String.fromCharCode(i));
for (let i = 0; i < 512; i++) enc.push(text(Array.from({ length: next() % 80 }, () => next() & 65535)));
for (const input of enc) base64.push({ op: 'encode', input: u16(input), output: u16(codec.encodeText(input)) });
const dec = ['','Zg==','Zg','Zh==','Zm8=','Zm8','aGVsbG8gd29ybGQ=', '4oKsbWzDpMO8dMOf', '77u/YQ==', '77u/77u/YQ==', 'Z g\r\n==', 'A', 'Zg=', 'Zg===', '=Zg=', 'Zg==\u00a0', '!!!!', '/w==', '8ICAAA=='];
for (let i = 0; i < 512; i++) {
  const bytes = Uint8Array.from({ length: next() % 60 }, () => next() & 255);
  dec.push(Buffer.from(bytes).toString('base64'));
}
for (const input of dec) base64.push({ op: 'decode', input: u16(input), output: u16(codec.decodeText(input)) });
const css = ['800px', '450.9px', '1px', '-10px', '', 'auto', '  +34.99em', '1e3', '\ufeff12px', '0x10', '9'.repeat(310), '-0', '\u008512px'];
const fit = [];
for (let i = 0; i < 513; i++) {
  const m = i===512 ? { cell_width:1,cell_height:1,parent_width:'18014398509481984px',parent_height:'18014398509481984px',padding:['1px','1px','1px','1px'],scrollback:0,show_scrollbar:false,scrollbar_width:null } : { cell_width: [0, 1, 9.2, 18.3, -1][next() % 5], cell_height: [0, 1, 17, 30.2][next() % 4], parent_width: css[next() % css.length], parent_height: css[next() % css.length], padding: Array.from({ length: 4 }, () => css[next() % css.length]), scrollback: next() % 2 ? 1000 : 0, show_scrollbar: [null, true, false][next() % 3], scrollbar_width: [null, 0, 14, 25][next() % 4] };
  const addon = new FitAddon();
  const style = values => ({ getPropertyValue: name => values[name] ?? '' });
  const view = { getComputedStyle: element => element.style };
  const parent = { ownerDocument: { defaultView: view }, style: style({ height: m.parent_height, width: m.parent_width }) };
  const element = { parentElement: parent, ownerDocument: { defaultView: view }, style: style(Object.fromEntries(['padding-top', 'padding-bottom', 'padding-right', 'padding-left'].map((v, j) => [v, m.padding[j]]))) };
  addon.activate({ element, dimensions: { css: { cell: { width: m.cell_width, height: m.cell_height } } }, options: { scrollback: m.scrollback, scrollbar: { showScrollbar: m.show_scrollbar ?? undefined, width: m.scrollbar_width ?? undefined } } });
  const result = addon.proposeDimensions();
  const number = v => Number.isNaN(v) ? 'NaN' : v === Infinity ? 'Infinity' : v === -Infinity ? '-Infinity' : v;
  fit.push({ m, output: result ? { cols: number(result.cols), rows: number(result.rows) } : null });
}
const clipboard = [];
for (const data of ['c;aGVsbG8gd29ybGQ=', 'p;aGVsbG8gd29ybGQ=', ';aGVsbG8gd29ybGQ=', 'c;aGVsbG8gd29ybGQ=invalid', 'c;', 'c;?', 'p;?', 'c;!', 'c;4oKsbWzDpMO8dMOf', 'missing', 'c;?;ignored', ';?']) {
  for (const async of [false, true]) {
    const events = [];
    const provider = { readText: selection => { events.push(['read', u16(selection)]); return async ? Promise.resolve('€mläütß') : '€mläütß'; }, writeText: (selection, value) => { events.push(['write', u16(selection), u16(value)]); return async ? Promise.resolve() : undefined; } };
    let handler;
    const addon = new ClipboardAddon(undefined, provider);
    addon.activate({ parser: { registerOscHandler: (id, callback) => { assert.equal(id, 52); handler = callback; return { dispose() {} }; } }, input: (reply, user) => events.push(['input', u16(reply), user]) });
    const result = handler(data);
    const immediate = structuredClone(events);
    assert.equal(await result, true);
    clipboard.push({ data: u16(data), async, immediate, events });
  }
}
function bufferFor(lines) {
  const wrappers = lines.map(line => ({
    isWrapped: line.wrapped,
    length: line.cells.length,
    translateToString: () => line.cells.filter(c => c.width).map(c => text(c.chars) || ' ').join('').replace(/ +$/, ''),
    getCell: (i, target) => { const cell = line.cells[i]; target.getChars = () => text(cell.chars); target.getWidth = () => cell.width; return target; }
  }));
  return { active: { getLine: i => wrappers[i], getNullCell: () => ({}) } };
}
function makeLines(value, width) {
  const lines = [{ wrapped: false, cells: [] }];
  for (const char of value) {
    let line = lines.at(-1);
    if (/\p{Mark}/u.test(char) && line.cells.length) {
      const cell = [...line.cells].reverse().find(cell => cell.width);
      cell.chars.push(...u16(char));
      continue;
    }
    const cp = char.codePointAt(0);
    const wide = cp === 0xffe5 || (cp >= 0xac00 && cp <= 0xd7af) || cp > 0xffff;
    const size = wide ? 2 : 1;
    if (line.cells.length + size > width) {
      while (line.cells.length < width) line.cells.push({ chars: [], width: 1 });
      line = { wrapped: true, cells: [] }; lines.push(line);
    }
    line.cells.push({ chars: u16(char), width: size });
    if (wide) line.cells.push({ chars: [], width: 0 });
  }
  while (lines.at(-1).cells.length < width) lines.at(-1).cells.push({ chars: [], width: 1 });
  return lines;
}
const links = [];
const examples = ['aaa http://example.com aaa http://example.com aaa', 'HTTP://EXAMPLE.COM HTTPS://Example.com HTTP://Ab:xY@abc.com:80/staysUpper', 'http://example.com/colon:test: "http://example.com/" http://example.com/a~b#c~d?e~f', '￥￥￥cafe\u0301 http://test:password@example.com/some_path', 'https://ko.wikipedia.org/wiki/위키백과:대문', 'http://[::1]/ http://not a url ftp://host/ http://example.com:999999/'];
for (let i = 0; i < 256; i++) examples.push('a'.repeat(next() % 50) + ' http://example.com/' + 'x'.repeat(next() % 100));
for (const value of examples) {
  const width = [8, 20, 40][next() % 3];
  const lines = makeLines(value, width);
  const terminal = { buffer: bufferFor(lines) };
  for (let y = 1; y <= lines.length; y++) {
    const [parts] = LinkComputer._getWindowedLineStrings(y - 1, terminal);
    const joined = parts.join('');
    const candidates = [...joined.matchAll(new RegExp(regex.source, 'g'))].filter(m => isUrl(m[0])).map(m => ({ index: m.index, text: u16(m[0]) }));
    const expected = LinkComputer.computeLink(y, regex, terminal, () => {}).map(link => ({ text: u16(link.text), range: link.range }));
    links.push({ lines, y, candidates, expected });
  }
}
const linkTestPath = root + 'addon-web-links/test/WebLinksAddon.test.ts';
const linkTest = await readFile(linkTestPath, 'utf8');
hashes[linkTestPath] = createHash('sha256').update(linkTest).digest('hex');
const tldBlock = linkTest.match(/const countryTlds = \[([\s\S]*?)\n  \];/)[1];
const tlds = [...tldBlock.matchAll(/'(\.[a-z]+)'/g)].map(match => match[1]);
const hostnames = [...tlds.map(tld => `foo${tld}`), 'foo.com', ...tlds.map(tld => `foo.com${tld}`)];
const hostCorpus = JSON.stringify({ hosts: hostnames, source: linkTestPath, sourceSha256: hashes[linkTestPath] }) + '\n';
const hostPath = 'crates/xterm-web/tests/fixtures/utility-hostnames.json';
await mkdir('crates/xterm-web/tests/fixtures', { recursive: true });
if (process.argv.includes('--generate')) await writeFile(hostPath, hostCorpus);
else assert.equal(await readFile(hostPath, 'utf8'), hostCorpus);
const corpus = JSON.stringify({ base64, fit, clipboard, links, regex: regex.source });
const path = 'crates/xterm-addons/tests/fixtures/utility-addons.json';
await mkdir('crates/xterm-addons/tests/fixtures', { recursive: true });
const sparseLinks=[];
for(const row of [3000000000,4294967294]) {
  const line=bufferFor(makeLines('http://example.com',18)).active.getLine(0);
  const terminal={buffer:{active:{getLine:index=>index===row?line:undefined,getNullCell:()=>({})}}};
  const found=LinkComputer.computeLink(row+1,regex,terminal,()=>{});
  assert.equal(found.length,1);
  sparseLinks.push({row,top:LinkComputer._getWindowedLineStrings(row,terminal)[1],range:found[0].range});
}
const sparsePath='crates/xterm-addons/tests/fixtures/utility-sparse-links.json';
const sparseCorpus=JSON.stringify(sparseLinks)+'\n';
if(process.argv.includes('--generate'))await writeFile(sparsePath,sparseCorpus);
else assert.equal(await readFile(sparsePath,'utf8'),sparseCorpus);
if (process.argv.includes('--generate')) await writeFile(path, corpus + '\n');
else assert.equal((await readFile(path, 'utf8')).trim(), corpus);
const report = { node: process.version, seed, hashes, counts: { base64: base64.length, fit: fit.length, clipboard: clipboard.length, links: links.length, countryTlds: tlds.length, hostnameCases: hostnames.length, hostnameAssertions: hostnames.length * 7, sparseLinks: sparseLinks.length }, corpusSha256: createHash('sha256').update(corpus).digest('hex'), status: 'source methods executed; portable corpus only; browser suites unrun' };
await mkdir('results/utility-addons', { recursive: true });
await writeFile('results/utility-addons/oracle.json', JSON.stringify(report, null, 2) + '\n');
console.log(JSON.stringify(report, null, 2));

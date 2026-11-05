// Compare unchanged source callback attachment to the production binding factory.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFile, mkdir, writeFile } from 'node:fs/promises';
import { stripTypeScriptTypes } from 'node:module';
import { resolve } from 'node:path';
const sourcePath = resolve(process.env.SOURCE_ROOT ?? 'source_repo', 'addons/addon-web-links/src/WebLinkProvider.ts');
const productionPath = 'crates/xterm-web/src/utility_addons/raw_link_bridge.js';
const source = await readFile(sourcePath, 'utf8');
const production = await readFile(productionPath, 'utf8');
const code = stripTypeScriptTypes(source.replace(/^import \{ ILinkProvider, ILink, Terminal, IViewportRange, IBufferLine \} from '@xterm\/xterm';\r?$/m, ''), { mode: 'transform' });
const load = js => import(`data:text/javascript;base64,${Buffer.from(js).toString('base64')}`);
const { WebLinkProvider } = await load(code);
const { webBrowserLink } = await load(production);
const marker = { thrown: 'marker' };
const isMarker = error => error === marker;
function run(factory, setup) {
  const calls = [], reads = [];
  let link;
  const options = setup({ calls, reads, getLink: () => link });
  const handler = function(event, text) { calls.push(['activate', this === link, text]); };
  const invoke = operation => {
    try { operation(); return 'ok'; }
    catch (error) { return isMarker(error) ? 'marker' : error.name; }
  };
  const created = invoke(() => { link = factory(options, handler); });
  const result = { created, reads, calls };
  if (link) {
    result.activation = invoke(() => link.activate({}, link.text));
    result.firstHover = invoke(() => link.hover({}, link.text));
    result.range = link.range && [link.range.start.x, link.range.start.y, link.range.end.x, link.range.end.y];
    const current = link.range;
    result.leave = invoke(() => { if (link.leave) link.leave({}, link.text); });
    link.range = { start: { x: 8, y: 2 }, end: { x: 10, y: 2 } };
    result.replacedRange = invoke(() => link.hover({}, link.text));
    result.priorMutation = current && current.start.x;
  }
  return result;
}
const sourceFactory = (options, handler) => {
  const provider = new WebLinkProvider(undefined, undefined, handler, options);
  return provider._addCallbacks([{ text: 'http://x.co', range: { start: { x: 1, y: 1 }, end: { x: 11, y: 1 } }, activate: handler }])[0];
};
const productionFactory = (options, handler) => webBrowserLink('http://x.co', [1, 1, 11, 1], handler, options);
const cases = [];
// Undefined options are normalized by the public constructor before this factory.
for (const value of [null, false, 0, '', true, 3, 'str', {}, [], { hover: false }, { hover: 3 }, { leave: 3 }]) {
  cases.push({ name: `raw ${String(value)}`, setup: () => value });
}
cases.push({ name: 'receivers and live range replacement', setup: ({ calls, getLink }) => {
  const options = {
    leave(event, text) { calls.push(['leave', this === getLink(), text]); },
    hover(event, text, range) { calls.push(['hover', this === options, range === getLink().range, text]); range.start.x += 1; }
  };
  return options;
} });
for (const field of ['leave', 'hover']) {
  cases.push({ name: `${field} thrown identity`, setup: ({ reads }) => Object.defineProperty({}, field, { get() { reads.push(field); throw marker; } }) });
}
cases.push({ name: 'hover reread and range sampled between reads', setup: ({ calls, reads, getLink }) => {
  let count = 0;
  const options = Object.create({ get leave() { reads.push('leave'); return undefined; } });
  Object.defineProperty(options, 'hover', { get() {
    reads.push('hover');
    if (++count % 2) return true;
    const old = getLink().range;
    getLink().range = { start: { x: 20, y: 2 }, end: { x: 22, y: 2 } };
    return function(event, text, range) { calls.push(['ordered', this === options, range === old, range === getLink().range]); };
  } });
  return options;
} });
cases.push({ name: 'hover truthiness getter replaces leave after capture', setup: ({ calls, reads }) => {
  const options = { leave() { calls.push(['original leave']); } };
  Object.defineProperty(options, 'hover', { get() { reads.push('hover'); options.leave = () => calls.push(['replacement leave']); return false; } });
  return options;
} });
cases.push({ name: 'second hover getter throws exact object', setup: ({ reads }) => {
  let count = 0;
  return Object.defineProperty({}, 'hover', { get() { reads.push('hover'); if (++count % 2 === 0) throw marker; return true; } });
} });
const results = [];
for (const test of cases) {
  const expected = run(sourceFactory, test.setup);
  const actual = run(productionFactory, test.setup);
  assert.deepEqual(actual, expected, test.name);
  results.push({ name: test.name, expected, actual, status: 'passed' });
}
// Direct factory activation retains arbitrary handler values rather than a
// truthiness fallback. The controller always invokes activate with link receiver.
for (const handler of [null, false, 0, '', {}, () => { throw marker; }]) {
  const fail = factory => { try { const link = factory({}, handler); link.activate({}, link.text); return 'ok'; } catch (error) { return isMarker(error) ? 'marker' : error.name; } };
  assert.equal(fail(productionFactory), fail(sourceFactory));
  results.push({ name: `raw activate ${typeof handler}`, status: 'passed', result: fail(productionFactory) });
}
const sha256 = value => createHash('sha256').update(value).digest('hex');
await mkdir('results/web-links-live-range', { recursive: true });
await writeFile('results/web-links-live-range/callbacks.json', JSON.stringify({ platform: process.platform, node: process.version,
  source: { path: sourcePath, sha256: sha256(source) }, production: { path: productionPath, sha256: sha256(production) },
  scope: 'Callback attachment only. No terminal discovery or browser claim.', results }, null, 2) + '\n');
console.log(`${results.length} source/production callback cases passed`);

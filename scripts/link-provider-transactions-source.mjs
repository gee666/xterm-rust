// Execute the original methods in memory, with no source or assertion changes.
import assert from 'node:assert/strict';
import { mkdir, writeFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { sourceLoader } from './source-ts-loader.mjs';

const root = process.env.SOURCE_ROOT ? resolve(process.env.SOURCE_ROOT, '..') : process.cwd();
const { source, load, hashes, typescript } = sourceLoader(root);
const { Linkifier } = load(resolve(source, 'src/browser/Linkifier.ts'));
const cases = [];
for (const mode of ['reentry', 'throw', 'caught nested throw', 'cache']) {
  const target = Object.create(Linkifier.prototype);
  const callbacks = [];
  const calls = [];
  const marker = { marker: mode };
  let caught;
  let hovers = 0;
  target._isMouseOut = false;
  target._bufferService = { cols: 20 };
  target._linkProviderService = { linkProviders: [0, 1].map(index => ({
    provideLinks(y, callback) { callbacks[index] = callback; }
  })) };
  const link = { text: 'high', range: { start: { x: 2, y: 1 }, end: { x: 6, y: 1 } } };
  target._handleNewLink = () => {
    calls.push(`hover${++hovers}`);
    if (mode === 'throw' && hovers === 1 || mode === 'caught nested throw' && hovers === 2) throw marker;
    if (hovers === 1 && mode !== 'throw') {
      try { callbacks[1](undefined); } catch (error) { caught = error; calls.push('nested caught'); }
    }
  };
  if (mode === 'cache') {
    target._activeProviderReplies = new Map([[0, [{ link }]]]);
    // The cached request owns a fresh outer linkProvided local. A callback here
    // models reentry into that same local before _checkLinkProviderResult returns.
    let provided = false;
    callbacks[1] = links => {
      target._activeProviderReplies.set(1, links);
      provided = target._checkLinkProviderResult(1, { x: 4, y: 1 }, provided);
    };
    target._askForLink({ x: 4, y: 1 }, true);
  } else {
    target._askForLink({ x: 4, y: 1 }, false);
    try { callbacks[0]([link]); } catch (error) { caught = error; calls.push('outer caught'); }
    if (mode === 'throw') callbacks[1](undefined);
    const before = hovers;
    callbacks[1](undefined);
    assert.equal(hovers, before, 'successful outer selection must eventually commit');
  }
  assert.equal(hovers, 2, mode);
  assert.equal(caught === marker, mode.includes('throw'), mode);
  cases.push({ mode, hovers, calls, caughtMarker: caught === marker });
}
const report = { node: process.version, typescript, cases, sources: Object.fromEntries(hashes),
  scope: 'Four unchanged Linkifier method traces. Cache case isolates method transaction, not a live provider request.' };
await mkdir('results/link-provider-reentry', { recursive: true });
await writeFile('results/link-provider-reentry/methods.json', JSON.stringify(report, null, 2) + '\n');
console.log(JSON.stringify(cases));

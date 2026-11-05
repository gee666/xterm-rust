// Execute unchanged Linkifier._handleMouseMove with only geometry/provider observation replaced.
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { sourceLoader } from './source-ts-loader.mjs';
const root = process.env.SOURCE_ROOT ? resolve(process.env.SOURCE_ROOT, '..') : process.cwd();
const { source, load, hashes, typescript } = sourceLoader(root);
const { Linkifier } = load(resolve(source, 'src/browser/Linkifier.ts'));
const cases = Function(await readFile('scripts/link-mouse-path-cases.js', 'utf8'))();
const trace = cases((event, marker) => {
  const target = Object.create(Linkifier.prototype);
  let provided = false, error = 'none';
  target._positionFromMouseEvent = () => ({ x: 1, y: 1 });
  target._handleHover = () => { provided = true; };
  try { target._handleMouseMove(event); }
  catch (caught) { error = caught === marker ? 'marker' : caught.name; }
  return { provided, error };
}, () => ({}));
await mkdir('results/link-mouse-path', { recursive: true });
await writeFile('results/link-mouse-path/methods.json', JSON.stringify({ typescript, trace, sources: Object.fromEntries(hashes) }, null, 2) + '\n');
console.log(`Unchanged source mouse path: ${trace.length} cases passed`);

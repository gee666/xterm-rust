// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
// Export the unchanged private AtlasPage only in this in-memory test bundle.
import {readFile, writeFile} from 'node:fs/promises';
import {createRequire} from 'node:module';
import {createHash} from 'node:crypto';
import path from 'node:path';
import vm from 'node:vm';
const root = path.resolve(process.env.SOURCE_ROOT ?? 'source_repo');
const file = path.join(root, 'addons/addon-webgl/src/TextureAtlas.ts');
const source = await readFile(file, 'utf8');
const wrapper = '\nexport {AtlasPage};\n';
const require = createRequire(path.join(root, 'package.json'));
const built = await require('esbuild').build({stdin: {contents: source + wrapper,
  sourcefile: file, resolveDir: path.dirname(file), loader: 'ts'}, bundle: true,
  write: false, platform: 'node', format: 'cjs',
  alias: {browser: path.join(root, 'src/browser'), common: path.join(root, 'src/common')}});
const module = {exports: {}};
vm.runInNewContext(built.outputFiles[0].text, {module, exports: module.exports, process, console});
const {AtlasPage} = module.exports;
const document = {createElement() {return {getContext() {return {clearRect() {}};}};}};
const cases = [];
for (const indices of [[], [0], [0, 1], [0, 1, 2, 3], [3, 1, 3, 0, 2]]) {
  AtlasPage.nextVersion = 0;
  const pages = Array.from({length: 4}, () => new AtlasPage(document, 16));
  const glyphs = Array.from({length: 4}, (_,i) => ({size: {x: i + 1, y: i + 2}}));
  pages.forEach((page, i) => page.addGlyph(glyphs[i]));
  const merged = new AtlasPage(document, 32, indices.map(i => pages[i]));
  const row = merged.currentRow;
  Object.assign(row, {x: 9, y: 8, height: 7});
  merged.fixedRows.push(row);
  const snapshot = () => ({version: merged.version, percentageUsed: merged.percentageUsed,
    glyphs: merged.glyphs.map(g => glyphs.indexOf(g)), row: {...merged.currentRow},
    sameRow: merged.currentRow === row, fixedCount: merged.fixedRows.length});
  const before = snapshot();
  merged.clear();
  const after = snapshot();
  pages[0].clear();
  cases.push({indices, before, after, sourceVersionAfterClear: pages[0].version});
}
const output = path.resolve('crates/xterm-web/tests/fixtures/webgl-atlas-pages.json');
await writeFile(output, JSON.stringify({source: {path: 'addons/addon-webgl/src/TextureAtlas.ts',
  sha256: createHash('sha256').update(source).digest('hex')}, wrapper, cases}) + '\n');
console.log(`${cases.length} unchanged-source page metadata cases -> ${output}`);

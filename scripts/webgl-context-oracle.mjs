// Copyright (c) 2018-2026 The xterm.js authors. All rights reserved. MIT license.
// Build the unchanged source class plus a separately hashed lifecycle test wrapper.
import {readFile, writeFile, mkdir} from 'node:fs/promises';
import {createRequire} from 'node:module';
import {createHash} from 'node:crypto';
import path from 'node:path';
const root = path.resolve(process.env.SOURCE_ROOT ?? 'source_repo');
const wrapper = path.resolve('scripts/support/webgl-context-source.ts');
const output = path.resolve(process.env.WEBGL_CONTEXT_PRELUDE ?? 'tmp/webgl-context/source.js');
const result = path.resolve('results/webgl-context/oracle.json');
const require = createRequire(path.join(root, 'package.json'));
const hash = value => createHash('sha256').update(value).digest('hex');
const contents = await readFile(wrapper, 'utf8');
const build = await require('esbuild').build({
  stdin: {contents, sourcefile: 'context-test-wrapper.ts', loader: 'ts', resolveDir: root},
  bundle: true, write: false, platform: 'browser', format: 'iife', globalName: '__webglContextSource', metafile: true,
  alias: {browser: path.join(root, 'src/browser'), common: path.join(root, 'src/common'),
    'source-webgl': path.join(root, 'addons/addon-webgl/src')},
});
const inputs = [];
for (const input of Object.keys(build.metafile.inputs).sort()) {
  if (path.resolve(input) === path.join(root, 'context-test-wrapper.ts')) continue;
  inputs.push({path: path.relative(root, input).replaceAll('\\', '/'), sha256: hash(await readFile(input))});
}
const prelude = build.outputFiles[0].text + '\nglobalThis.sourceContextFactory = __webglContextSource.sourceContextFactory;\n';
await mkdir(path.dirname(output), {recursive: true});
await mkdir(path.dirname(result), {recursive: true});
await writeFile(output, prelude);
await writeFile(result, JSON.stringify({sourceRoot: root, inputs,
  node: process.version, esbuild: require('esbuild').version,
  generatorSha256: hash(await readFile(new URL(import.meta.url))),
  wrapper: {path: 'scripts/support/webgl-context-source.ts', sha256: hash(contents)},
  supportSha256: hash(await readFile('crates/xterm-web/tests/support/webgl_context.js')),
  nativeSupportSha256: hash(await readFile('crates/xterm-web/tests/support/webgl_context_native.js')),
  prelude: {path: output, sha256: hash(prelude)}}, null, 2) + '\n');
console.log(`Unchanged WebglRenderer lifecycle prelude: ${output}; ${inputs.length} original inputs, one separate wrapper`);

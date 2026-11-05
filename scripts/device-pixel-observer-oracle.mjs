// Copyright (c) 2022-2026 The xterm.js authors. All rights reserved. MIT license.
// Bundle unchanged source for live browser differential tests, never Rust execution.
import {readFile, writeFile, mkdir} from 'node:fs/promises';
import {createHash} from 'node:crypto';
import {createRequire} from 'node:module';
import path from 'node:path';
const root = path.resolve(process.env.SOURCE_ROOT ?? 'source_repo');
const file = path.join(root, 'addons/addon-webgl/src/DevicePixelObserver.ts');
const output = path.resolve(process.env.DEVICE_PIXEL_PRELUDE ?? 'tmp/device-pixel-observer/prelude.js');
const result = path.resolve('results/device-pixel-observer/oracle.json');
const require = createRequire(path.join(root, 'package.json'));
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
const build = await require('esbuild').build({entryPoints: [file], bundle: true, write: false,
  platform: 'browser', format: 'iife', globalName: '__devicePixelSource', metafile: true,
  alias: {common: path.join(root, 'src/common')}});
const inputs = [];
for (const input of Object.keys(build.metafile.inputs).sort()) {
  inputs.push({path: path.relative(root, input).replaceAll('\\', '/'), sha256: hash(await readFile(input))});
}
const prelude = build.outputFiles[0].text + '\nglobalThis.sourceObserveDevicePixelDimensions = __devicePixelSource.observeDevicePixelDimensions;\n';
await mkdir(path.dirname(output), {recursive: true});
await mkdir(path.dirname(result), {recursive: true});
await writeFile(output, prelude);
await writeFile(result, JSON.stringify({sourceRoot: root, inputs,
  generatorSha256: hash(await readFile(new URL(import.meta.url))),
  prelude: {path: output, sha256: hash(prelude)},
  supportSha256: hash(await readFile('crates/xterm-web/tests/support/device_pixel_observer.js'))}, null, 2) + '\n');
console.log(`Unchanged-source observer prelude: ${output}; ${inputs.length} input hashes`);

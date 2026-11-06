// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
// Bundle unchanged reference inputs in memory; do not modify source_repo.
import {readFile,writeFile,mkdir} from 'node:fs/promises';
import {createHash} from 'node:crypto';
import {createRequire} from 'node:module';
import path from 'node:path';
const root=path.resolve(process.env.SOURCE_ROOT??'source_repo');
const require=createRequire(path.join(root,'package.json'));
const hash=bytes=>createHash('sha256').update(bytes).digest('hex');
const entry='crates/xterm-web/tests/reference/canvas-layer.mjs';
const build=await require('esbuild').build({entryPoints:[entry],bundle:true,write:false,platform:'browser',format:'iife',globalName:'__canvasLayerSource',metafile:true,
 alias:{webgl:path.join(root,'addons/addon-webgl/src'),common:path.join(root,'src/common'),browser:path.join(root,'src/browser')}});
const inputs=[];
for(const file of Object.keys(build.metafile.inputs).sort()){
 const absolute=path.resolve(file),relative=path.relative(root,absolute);
 const reference=!relative.startsWith('..')&&!path.isAbsolute(relative);
 inputs.push({reference,path:(reference?relative:path.relative(process.cwd(),absolute)).replaceAll('\\\\','/'),sha256:hash(await readFile(file))});
}
const prelude=build.outputFiles[0].text+'\nglobalThis.sourceCanvasLayer=__canvasLayerSource.sourceCanvasLayer;\n';
const output=path.resolve('tmp/canvas-layer/source.js');await mkdir(path.dirname(output),{recursive:true});await writeFile(output,prelude);
await mkdir('results/canvas-layer',{recursive:true});
await writeFile('results/canvas-layer/source-inputs.json',JSON.stringify({inputs,output,sha256:hash(prelude)},null,2)+'\n');
console.log(output);

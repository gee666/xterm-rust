// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
// Bundle the unchanged source cache and TextureAtlas dependency in memory.
import {readFile,writeFile,mkdir} from 'node:fs/promises';
import {createRequire} from 'node:module';
import {createHash} from 'node:crypto';
import path from 'node:path';
const root=path.resolve(process.env.SOURCE_ROOT??'source_repo');
const file=path.join(root,'addons/addon-webgl/src/CharAtlasCache.ts');
const wrapper=path.resolve('crates/xterm-web/tests/reference/browser-atlas-cache.mjs');
const source=await readFile(file,'utf8');
const require=createRequire(path.join(root,'package.json'));
const built=await require('esbuild').build({stdin:{contents:source+'\n'+await readFile(wrapper,'utf8'),
  sourcefile:file,resolveDir:path.dirname(file),loader:'ts'},bundle:true,write:false,platform:'browser',format:'iife',metafile:true,
  alias:{browser:path.join(root,'src/browser'),common:path.join(root,'src/common')}});
const output=path.resolve(process.env.ATLAS_ORACLE_OUTPUT??'tmp/browser-atlas-cache/source.js');
await mkdir(path.dirname(output),{recursive:true});
await writeFile(output,built.outputFiles[0].text);
const hash=bytes=>createHash('sha256').update(bytes).digest('hex');
const inputs=[{path:path.relative(root,file).replaceAll('\\','/'),sha256:hash(await readFile(file))}];
for(const input of Object.keys(built.metafile.inputs).sort()) {
  if(input==='<stdin>'||path.resolve(input)===file)continue;
  inputs.push({path:path.relative(root,input).replaceAll('\\','/'),sha256:hash(await readFile(input))});
}
await writeFile(output+'.json',JSON.stringify({inputs,wrapper:{path:path.relative(process.cwd(),wrapper).replaceAll('\\','/'),
  sha256:hash(await readFile(wrapper))},bundleSha256:hash(built.outputFiles[0].contents)},null,2)+'\n');
console.log(`Unchanged-source CharAtlasCache/TextureAtlas probe -> ${output}`);

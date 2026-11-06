// Copyright (c) 2018-2026 The xterm.js authors. All rights reserved. MIT license.
// Bundle immutable upstream code with a separately hashed test wrapper.
import {readFile,writeFile,mkdir} from 'node:fs/promises';
import {createRequire} from 'node:module';
import {createHash} from 'node:crypto';
import path from 'node:path';
const source=path.resolve(process.env.SOURCE_ROOT??'source_repo');
const require=createRequire(path.join(source,'package.json'));
const wrapper='scripts/support/webgl-frame-source.ts';
const bytes=await readFile(wrapper);
const build=await require('esbuild').build({stdin:{contents:bytes.toString(),sourcefile:'frame-test-wrapper.ts',loader:'ts',resolveDir:source},
  bundle:true,write:false,platform:'browser',format:'iife',globalName:'__webglFrameSource',metafile:true,
  alias:{browser:path.join(source,'src/browser'),common:path.join(source,'src/common'),'source-webgl':path.join(source,'addons/addon-webgl/src')}});
const sha256=b=>createHash('sha256').update(b).digest('hex');
const inputs=[];
for(const file of Object.keys(build.metafile.inputs).sort())if(!file.endsWith('frame-test-wrapper.ts'))
  inputs.push({path:path.relative(source,file).replaceAll('\\','/'),sha256:sha256(await readFile(file))});
const output=path.resolve('tmp/webgl-frame/source.js');await mkdir(path.dirname(output),{recursive:true});
await mkdir('results/webgl-frame',{recursive:true});await writeFile(output,build.outputFiles[0].contents);
await writeFile('results/webgl-frame/oracle.json',JSON.stringify({node:process.version,esbuild:require('esbuild').version,
  inputs,wrapper:{path:wrapper,sha256:sha256(bytes)},generatorSha256:sha256(await readFile(new URL(import.meta.url))),
  output:{path:output,sha256:sha256(build.outputFiles[0].contents)}},null,2)+'\n');
console.log(`${inputs.length} unchanged inputs; actual WebglRenderer/Terminal/GPU/atlas oracle -> ${output}`);

// Copyright (c) 2014-2026 The xterm.js authors. All rights reserved. MIT license.
// Read-only shared source loader for real headless terminal differential tests.
import { createRequire } from 'node:module';
import { createHash } from 'node:crypto';
import { readFile } from 'node:fs/promises';
const require=createRequire(import.meta.url);
export async function loadTerminalReference() {
  const {build}=require('../source_repo/node_modules/esbuild');
  const compiled=await build({entryPoints:['source_repo/src/headless/public/Terminal.ts'],bundle:true,write:false,platform:'node',format:'esm',target:'node24',tsconfig:'source_repo/src/headless/tsconfig.json',metafile:true});
  const {Terminal}=await import(`data:text/javascript;base64,${Buffer.from(compiled.outputFiles[0].contents).toString('base64')}`);
  const sources=await Promise.all(Object.keys(compiled.metafile.inputs).sort().map(async file=>({file,sha256:createHash('sha256').update(await readFile(file)).digest('hex')})));
  return {Terminal,sources};
}

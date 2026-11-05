// Copyright (c) 2019-2023 The xterm.js authors. All rights reserved. MIT license.
// Signed numeric glyph codes reach providers before String.fromCharCode drawing.
import {createRequire} from 'node:module';
import {readFile,writeFile} from 'node:fs/promises';
import {createHash} from 'node:crypto';
import path from 'node:path';
import vm from 'node:vm';
const root=path.resolve(process.env.SOURCE_ROOT??'source_repo');
const require=createRequire(path.join(root,'package.json'));
const entry=`export {UnicodeV6} from './src/common/input/UnicodeV6';
export {UnicodeV11} from './addons/addon-unicode11/src/UnicodeV11';
export {UnicodeGraphemeProvider} from './addons/addon-unicode-graphemes/src/UnicodeGraphemeProvider';`;
const built=await require('esbuild').build({stdin:{contents:entry,resolveDir:root,sourcefile:'signed-glyph-width.ts',loader:'ts'},
  bundle:true,write:false,platform:'node',format:'cjs',metafile:true,alias:{common:path.join(root,'src/common')}});
// Match the established unicode-addons oracle's browser atob branch. Node's
// pooled Buffer offsets are incompatible with the source trie DataView loader.
const module={exports:{}};vm.runInNewContext(built.outputFiles[0].text,{module,exports:module.exports,process,console,atob,btoa},{filename:'signed-glyph-width.cjs'});
const {UnicodeV6,UnicodeV11,UnicodeGraphemeProvider}=module.exports;
const providers=[new UnicodeV6(),new UnicodeV11(),new UnicodeGraphemeProvider(false),new UnicodeGraphemeProvider(true)];
const values=[-2147483648,-2147483583,-1,0,1,31,32,65,0x301,0xd800,0x4e00,0x1f680,0x110000,2147483647];
const cases=providers.flatMap(p=>[false,true].flatMap(ambiguous=>{p.ambiguousCharsAreWide=ambiguous;return values.map(code=>({version:p.version,ambiguous,code,width:p.wcwidth(code)}));}));
const inputs=[];for(const input of Object.keys(built.metafile.inputs).sort()){if(input==='<stdin>'||path.resolve(input)===path.join(root,'signed-glyph-width.ts'))continue;
 inputs.push({path:path.relative(root,input).replaceAll('\\','/'),sha256:createHash('sha256').update(await readFile(input)).digest('hex')});}
const output=path.resolve('crates/xterm-web/tests/fixtures/webgl-unicode-number.json');
await writeFile(output,JSON.stringify({entry,inputs,cases})+'\n');console.log(cases.length+' source signed-width cases -> '+output);

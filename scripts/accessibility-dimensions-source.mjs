// MIT. Verify test-only methods copied from the unchanged accessibility source.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {readFile, writeFile} from 'node:fs/promises';
import {resolve} from 'node:path';
const root=resolve(import.meta.dirname,'..');
const raw=await readFile(resolve(root,'source_repo/src/browser/AccessibilityManager.ts'),'utf8');
const source=raw.replaceAll('\r\n','\n');
const methods=[];
for(const signature of [
  'private _refreshRowsDimensions(): void {',
  'private _refreshRowDimensions(element: HTMLElement): void {',
  'private _alignRowWidth(element: HTMLElement): void {'
]) {
  const offset=source.indexOf(signature);
  assert.ok(offset>=0,`Missing source signature ${signature}`);
  const start=offset+signature.length;
  const end=source.indexOf('\n  }',start);
  assert.ok(end>start);
  methods.push(signature.replace('private ','').replace(': HTMLElement','').replace(': void','')+source.slice(start,end)+'\n  }');
}
const hash=createHash('sha256').update(raw).digest('hex');
const fixture=`// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.\n// Generated unchanged method bodies. Test-only, never imported by the library.\n// AccessibilityManager.ts SHA-256: ${hash}\nexport class SourceDimensions {\n${methods.join('\n')}\n}\n`;
const output=resolve(root,'crates/xterm-web/tests/fixtures/accessibility-dimensions-source.js');
if(process.argv.includes('--write'))await writeFile(output,fixture);
else assert.equal((await readFile(output,'utf8')).replaceAll('\r\n','\n'),fixture);
console.log(`Accessibility dimension methods match unchanged source ${hash}`);

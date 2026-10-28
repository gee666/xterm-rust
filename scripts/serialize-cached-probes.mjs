// Tiny source-only check reusing the last verified reference bundle. No build.
import { createRequire } from 'node:module';
import { readFile, writeFile } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import assert from 'node:assert/strict';
import path from 'node:path';
const root=path.resolve(import.meta.dirname,'..');
const require=createRequire(path.join(root,'source_repo/package.json'));
const reference='tmp/serialize-live-oracle/reference.cjs';
const manifest=JSON.parse(await readFile(path.join(root,'results/serialize/live-source.json'),'utf8'));
const hash=bytes=>createHash('sha256').update(bytes).digest('hex');
for(const [file,expected] of Object.entries(manifest.sourceHashes)) {
  assert.equal(hash(await readFile(path.join(root,file))),expected,`Reference source changed: ${file}`);
}
const {Terminal,SerializeAddon}=require(path.join(root,reference));
const terminal=new Terminal({cols:10,rows:2});
const addon=new SerializeAddon();terminal.loadAddon(addon);
try {
  await new Promise(resolve=>terminal.write('normal',resolve));
  const before=addon.serialize();
  terminal.options.theme={get foreground(){terminal.reset();return undefined;}};
  const resetHtml=addon.serializeAsHTML({includeGlobalBackground:true});
  const report={command:'node scripts/serialize-cached-probes.mjs',node:process.version,
    scope:'Actual original browser-terminal class under Node, not a browser runtime or Rust check',
    bundle:reference,bundleSha256:hash(await readFile(path.join(root,reference))),
    referenceSourcesVerified:true,allowProposedApi:terminal.options.allowProposedApi,
    before,resetHtml,blankRows:(resetHtml.match(/<div><span> {10}<\/span><\/div>/g)||[]).length};
  assert.equal(report.allowProposedApi,false);assert.equal(before,'normal');
  assert.equal(report.blankRows,2);assert.ok(!resetHtml.includes('normal'));
  await writeFile(path.join(root,'results/serialize/source-reset-ungated-cached.json'),JSON.stringify(report,null,2)+'\n');
  console.log(JSON.stringify(report));
} finally {terminal.dispose();}

// Case-level accounting is not assertion-level equivalence. Keep that distinction
// explicit when joining original Mocha names to native Rust test registrations.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync, writeFileSync, mkdirSync, existsSync, readdirSync } from 'node:fs';
import { resolve } from 'node:path';
import { spawnSync } from 'node:child_process';
import { captureRustInputs } from './public-api-evidence.mjs';
process.chdir(resolve(import.meta.dirname, '..'));
const source = 'source_repo/src/headless/public/Terminal.test.ts';
const baseline = JSON.parse(readFileSync('results/headless/upstream.json', 'utf8'));
const hash = path => createHash('sha256').update(readFileSync(path)).digest('hex');
assert.equal(hash(source), baseline.sourceHashes[source], 'baseline source hash is stale');
const groups = {
  writes: [
    ['Default options','default_options',232], ['Proposed API check','proposed_api_check',232],
    ['write','write',233], ['write with callback','write_with_callback',233],
    ['write - bytes (UTF8)','write_bytes_utf8',233],
    ['write - bytes (UTF8) with callback','write_bytes_utf8_with_callback',233],
    ['writeln','writeln',233], ['writeln with callback','writeln_with_callback',233],
    ['writeln - bytes (UTF8)','writeln_bytes_utf8',233], ['clear','clear',236],
    ['clear disposes markers','clear_disposes_markers',236], ['dispose','dispose',234],
    ['options get options','get_options',232], ['options set options','set_options',232],
    ['loadAddon constructor','load_addon_constructor',236],
    ['loadAddon dispose (addon)','load_addon_dispose_addon',236],
    ['loadAddon dispose (terminal)','load_addon_dispose_terminal',236]
  ],
  events: ['onCursorMove','onData','onLineFeed','onRender','onScroll','onResize','onTitleChange','onBell']
    .map(name => [`Events ${name}`, name.replace(/[A-Z]/g, letter => `_${letter.toLowerCase()}`),234]),
  buffers: [
    ['cursorX, cursorY','cursor_x_cursor_y'], ['viewportY','viewport_y'], ['baseY','base_y'],
    ['length','length'], ['active, normal, alternate','active_normal_alternate'],
    ['registerMarker on alternate buffer','register_marker_on_alternate_buffer'],
    ['getLine invalid index','get_line_invalid_index'], ['getLine isWrapped','get_line_is_wrapped'],
    ['getLine translateToString','get_line_translate_to_string'], ['getLine getCell','get_line_get_cell']
  ].map(([name,native]) => [`buffer ${name}`,native,236]),
  modes: ['defaults','applicationCursorKeysMode','applicationKeypadMode','bracketedPasteMode','insertMode',
    'mouseTrackingMode','originMode','reverseWraparoundMode','sendFocusMode','wraparoundMode']
    .map(name => [`modes ${name}`, name.replace(/[A-Z]/g, letter => `_${letter.toLowerCase()}`),235])
};
const logPath = 'results/public-api/headless-upstream-native.log';
const runPath = 'results/public-api/headless-native-run.json';
const inputPaths = ['crates/xterm-core/tests/headless_upstream.rs',
  ...readdirSync('crates/xterm-core/tests/headless_cases').filter(path=>path.endsWith('.rs')).map(path=>`crates/xterm-core/tests/headless_cases/${path}`)];
const snapshot = () => captureRustInputs(inputPaths);
let nativeExit = 0;
if (process.argv.includes('--run-native')) {
  const attempt = `headless-native-${Date.now()}`;
  const before = snapshot();
  writeFileSync(runPath,JSON.stringify({status:'running',inputs:before})+'\n');
  const cargo = process.env.CARGO ?? (process.platform === 'win32' ? `${process.env.USERPROFILE}/.cargo/bin/cargo.exe` : 'cargo');
  const run = spawnSync(cargo,['test','-p','xterm-core','--test','headless_upstream'],{encoding:'utf8',timeout:180000,maxBuffer:8*1024*1024});
  const logText = (run.stdout??'')+'\n'+(run.stderr??'');
  const changedDuringRun = JSON.stringify(before)!==JSON.stringify(snapshot());
  const evidence = {attempt, command:'cargo test -p xterm-core --test headless_upstream',
    exitCode:run.status,error:run.error?.message??null,inputs:before,changedDuringRun,
    logSha256:createHash('sha256').update(logText).digest('hex')};
  for (const path of [logPath, `results/public-api/${attempt}.log`]) writeFileSync(path,logText);
  for (const path of [runPath, `results/public-api/${attempt}.json`]) writeFileSync(path,JSON.stringify(evidence,null,2)+'\n');
  nativeExit = run.status===0 && !changedDuringRun ? 0 : 1;
}
const log = existsSync(logPath) ? readFileSync(logPath,'utf8') : '';
const evidence = existsSync(runPath) ? JSON.parse(readFileSync(runPath,'utf8')) : null;
const evidenceCurrent = existsSync(logPath) && evidence && !evidence.changedDuringRun && evidence.logSha256===hash(logPath)
  && JSON.stringify(evidence.inputs)===JSON.stringify(snapshot());
const executed = new Map(evidenceCurrent ? [...log.matchAll(/^test ([a-z0-9_:]+) \.\.\. (ok|FAILED)$/gm)]
  .map(([,name,status]) => [name,status === 'ok' ? 'passed' : 'failed']) : []);
const entries = [];
for (const [module, mappings] of Object.entries(groups)) {
  const path = `crates/xterm-core/tests/headless_cases/${module}.rs`;
  const contents = readFileSync(path,'utf8');
  for (const [title,name,criterion] of mappings) {
    assert.ok(new RegExp(`\\bfn\\s+${name}\\s*\\(`).test(contents), `missing native function ${module}::${name}`);
    const sourceName = `Headless API Tests ${title}`;
    const original = baseline.outcomes.find(test => test.name === sourceName);
    assert.ok(original, `unknown original case ${sourceName}`);
    const native = `${module}::${name}`;
    entries.push({ sourceName, source, originalStatus:original.status, nativeTest:native, nativePath:path,
      nativeSha256:hash(path), criterion, nativeExecution:executed.get(native) ?? 'unrun',
      assertionReview:'case mapping only; per-assertion fidelity and ABI exclusions require independent review' });
  }
}
assert.equal(new Set(entries.map(row=>row.sourceName)).size,45);
assert.equal(entries.length,baseline.upstream.discovered);
assert.equal(entries.length,45);
const report = {
  scope:'Original45 public headless case-name accounting, not proof of every assertion or asynchronous/browser equivalence',
  command:'node scripts/check-headless-mapping.mjs', sourceSha256:hash(source),
  originalDiscovered:baseline.upstream.discovered, mappedCases:entries.length,
  nativeExecuted:entries.filter(row=>row.nativeExecution!=='unrun').length,
  nativePassed:entries.filter(row=>row.nativeExecution==='passed').length,
  nativeFailed:entries.filter(row=>row.nativeExecution==='failed').length,
  nativeUnrun:entries.filter(row=>row.nativeExecution==='unrun').length,
  nativeLogSha256:existsSync(logPath) ? hash(logPath) : null, nativeEvidenceCurrent:Boolean(evidenceCurrent),
  nativeEvidenceNote:evidenceCurrent ? 'log and source snapshot match current files' : 'unrun or stale source/log evidence; use --run-native', entries
};
mkdirSync('results/public-api',{recursive:true});
writeFileSync('results/public-api/headless-case-map.json',JSON.stringify(report,null,2)+'\n');
console.log(JSON.stringify({...report,entries:undefined},null,2));
process.exitCode = nativeExit;

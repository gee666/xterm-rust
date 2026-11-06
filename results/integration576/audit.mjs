// Producer unions come from immutable Git objects, never the assembly being checked.
// The live compatibility hash list is independently regenerated from audited inputs.
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import {execFileSync, spawnSync} from 'node:child_process';
const out = 'results/integration576';
const git = (...args) => execFileSync(process.env.GIT ?? 'git', args,
  {timeout:30000, maxBuffer:128 * 1024 * 1024});
const text = (...args) => git(...args).toString().trim();
const sha = bytes => crypto.createHash('sha256').update(bytes).digest('hex');
const plan = JSON.parse(fs.readFileSync(`${out}/plan.json`));
const originals = JSON.parse(fs.readFileSync(`${out}/originals.json`));
const source = file => !file.startsWith('results/');
const production = file => /^(crates|scripts|tools|\.github|\.pi\/swarm\/skills)\//.test(file)
  || ['Cargo.toml', 'Cargo.lock', '.gitattributes'].includes(file);
const cache = new Map();
const trees = new Map();
function exists(ref, file) {
  if (!trees.has(ref)) trees.set(ref,new Set(text('ls-tree','-r','--name-only',ref).split('\n')));
  return trees.get(ref).has(file);
}
function blob(ref, file) {
  const spec = `${ref}:${file}`;
  if (!cache.has(spec)) cache.set(spec, git('show', spec));
  return cache.get(spec);
}
function prime(specs) {
  const unique = [...new Set(specs)].filter(spec => !cache.has(spec));
  const sizes = execFileSync(process.env.GIT ?? 'git',['cat-file','--batch-check=%(objectsize)'],
    {input:unique.join('\n')+'\n',timeout:30000,maxBuffer:4 * 1024 * 1024})
    .toString().trim().split('\n').map(Number);
  if (sizes.length !== unique.length || sizes.some(size => !Number.isSafeInteger(size) || size < 0)) throw Error('Invalid immutable object sizes');
  for (let offset = 0; offset < unique.length;) {
    let end = offset, total = 0;
    do { total += sizes[end++]; } while (end < unique.length && end - offset < 32 && total + sizes[end] < 32 * 1024 * 1024);
    const batch = unique.slice(offset,end);
    offset = end;
    const bytes = execFileSync(process.env.GIT ?? 'git',['cat-file','--batch'],
      {input:batch.join('\n')+'\n',timeout:30000,maxBuffer:Math.max(128 * 1024 * 1024,total + 1024 * 1024)});
    let cursor = 0;
    for (const spec of batch) {
      const end = bytes.indexOf(10,cursor), match = /^\S+ blob (\d+)$/.exec(bytes.subarray(cursor,end).toString());
      if (!match) throw Error(`Invalid immutable object ${spec}`);
      const size = Number(match[1]);
      cache.set(spec,bytes.subarray(end + 1,end + 1 + size));
      cursor = end + size + 2;
    }
    if (cursor !== bytes.length) throw Error('Unexpected immutable object suffix');
  }
}
function replaceOnce(value, before, after) {
  if (value.split(before).length !== 2) throw Error(`Ambiguous union anchor ${before}`);
  return value.replace(before, after);
}
function resolveConflict(file, output) {
  return output.replace(/<<<<<<< current\n([\s\S]*?)\n=======\n([\s\S]*?)\n?>>>>>>> producer/g, (_, ours, theirs) => {
    if (file === 'crates/xterm-web/src/terminal/marker_api.rs') {
      if (!ours.includes('pub(super) fn native') || theirs !== '') throw Error('Unexpected marker conflict');
      return '}\nimpl BrowserMarker {\n    pub(super) fn native(&self) -> Marker {\n        self.marker.clone()\n    }';
    }
    if (file === 'scripts/search-call-args-source.mjs') return replaceOnce(ours, "'search-delayed-expiry': 6", "'search-delayed-expiry': 6, 'search-highlight-bounds': 8");
    throw Error('Undeclared source union ' + file + '\n' + ours + '\n===\n' + theirs);
  });
}
const scratch = fs.mkdtempSync(path.resolve('tmp', 'audit576-'));
function merge(file, ours, base, theirs) {
  if (ours.equals(base)) return {bytes:theirs, composed:false};
  if (theirs.equals(base) || ours.equals(theirs)) return {bytes:ours, composed:false};
  const files = ['current','base','producer'].map(name => path.join(scratch, name));
  [ours,base,theirs].forEach((bytes,i) => fs.writeFileSync(files[i],bytes));
  const result = spawnSync(process.env.GIT ?? 'git', ['merge-file','-p','-L','current','-L','base','-L','producer',...files],
    {timeout:30000, maxBuffer:16 * 1024 * 1024});
  if (result.error || result.status < 0 || result.status >= 128 || result.status === null) throw Error(`Cannot merge expected ${file}`);
  let bytes = result.status === 0 ? result.stdout : Buffer.from(resolveConflict(file, result.stdout.toString()));
  if (/^(<<<<<<<|=======|>>>>>>>)/m.test(bytes.toString())) throw Error(`Unresolved expected ${file}`);
  return {bytes, composed:true};
}
const expected = new Map(), owners = new Map(), composed = new Set();
const report = {base:plan.base, assembly:text('rev-parse','HEAD'), sourcePins:[], receiptPins:[], preservedBasePins:[]};
try {
  const baseline = JSON.parse(blob(plan.base,'results/integration554/manifest.json'));
  const oldFiles = [...new Set([...baseline.sourcePins,...baseline.preservedBasePins].map(pin => pin.path))];
  const specs = oldFiles.flatMap(file => [`${plan.base}:${file}`,`HEAD:${file}`]);
  for (const batch of originals) {
    for (const file of new Set(batch.commits.flatMap(commit => commit.paths))) {
      if (!source(file) && !file.startsWith('results/')) continue;
      const first = batch.commits.find(commit => commit.paths.includes(file));
      for (const ref of [batch.commits.at(-1).ref,'HEAD',...(source(file) ? [plan.base,first.parents.split(' ')[1]] : [])]) {
        if (exists(ref,file)) specs.push(`${ref}:${file}`);
      }
    }
  }
  prime(specs);
  for (const batch of originals) {
    const files = [...new Set(batch.commits.flatMap(commit => commit.paths))];
    for (const file of files) {
      if (!source(file) && !file.startsWith('results/')) continue;
      if (file === 'results/demos/build/latest.json') continue;
      const first = batch.commits.find(commit => commit.paths.includes(file));
      const parent = first.parents.split(' ')[1], tip = batch.commits.at(-1).ref;
      const present = exists(tip,file);
      if (!source(file)) {
        expected.set(file, {task:batch.task, ref:tip, bytes:present ? blob(tip,file) : null});
        continue;
      }
      const current = expected.get(file)?.bytes ?? (exists(plan.base,file) ? blob(plan.base,file) : null);
      const old = exists(parent,file) ? blob(parent,file) : null;
      const producer = present ? blob(tip,file) : null;
      let value;
      if (!old) {
        if (current && !current.equals(producer)) throw Error(`Different added source ${file}`);
        value = {bytes:producer,composed:false};
      } else if (!producer) {
        if (!current?.equals(old)) throw Error(`Conflicting deletion ${file}`);
        value = {bytes:null,composed:false};
      } else {
        if (!current) throw Error(`Missing current source ${file}`);
        value = merge(file,current,old,producer);
      }
      if (value.composed) composed.add(file);
      expected.set(file,{task:batch.task,ref:tip,bytes:value.bytes});
      owners.set(file,[...(owners.get(file) ?? []),batch.task]);
    }
  }
  // A producer's live evidence hashes necessarily stale when other accepted
  // source enters the union. Keep its immutable bytes and derive the new list
  // with the unchanged, itself-audited generator in fresh owned scratch.
  const derivedPath = 'results/compatibility/evidence-manifest.json';
  const producerManifest = expected.get(derivedPath);
  if (producerManifest) {
    const generatedDir = path.relative(process.cwd(), scratch).split(path.sep).join('/');
    const generated = spawnSync(process.execPath, ['scripts/compatibility-inventory.mjs', `--out=${generatedDir}`],
      {timeout:60000, maxBuffer:8 * 1024 * 1024, encoding:'utf8'});
    if (generated.error || generated.status !== 0) throw Error(`Cannot derive live compatibility hashes: ${generated.error?.message ?? generated.stderr}`);
    const bytes = fs.readFileSync(path.join(scratch, 'evidence-manifest.json'));
    report.derivedCompatibility = {path:derivedPath, producerSha256:sha(producerManifest.bytes),
      derivedSha256:sha(bytes), command:generated.spawnargs ?? ['node','scripts/compatibility-inventory.mjs',`--out=${generatedDir}`]};
    expected.set(derivedPath, {...producerManifest, bytes});
  }
  for (const [file, entry] of expected) {
    const actual = exists('HEAD',file) ? blob('HEAD',file) : null;
    const matches = entry.bytes === null ? actual === null : actual !== null && entry.bytes.equals(actual);
    const pin = {task:entry.task,path:file,expectedSha256:entry.bytes === null ? null : sha(entry.bytes),
      headSha256:actual === null ? null : sha(actual),matches,composed:composed.has(file)};
    if (source(file)) {
      const working = fs.readFileSync(file);
      pin.checkoutSha256 = sha(working);
      pin.checkoutEolOnly = !entry.bytes.equals(working) && entry.bytes.toString().replaceAll('\r\n','\n') === working.toString().replaceAll('\r\n','\n');
      pin.matches &&= entry.bytes.equals(working) || pin.checkoutEolOnly;
      pin.owners = owners.get(file);
      report.sourcePins.push(pin);
    } else report.receiptPins.push(pin);
  }
  for (const file of oldFiles) {
    const wanted = expected.get(file)?.bytes ?? blob(plan.base,file);
    const actual = blob('HEAD',file);
    report.preservedBasePins.push({path:file,expectedSha256:sha(wanted),headSha256:sha(actual),matches:wanted.equals(actual)});
  }
  report.receiptPins.push({path:'results/demos/build/latest.json',preservedMainPointer:true,
    matches:blob(plan.base,'results/demos/build/latest.json').equals(blob('HEAD','results/demos/build/latest.json'))});
  report.unexpectedSource = text('diff','--name-only',plan.base,'HEAD').split('\n').filter(file => production(file) && !expected.has(file));
  const mismatches = [...report.sourcePins,...report.receiptPins,...report.preservedBasePins].filter(pin => !pin.matches);
  report.status = mismatches.length || report.unexpectedSource.length ? 'failed' : 'passed';
  report.composedFiles = [...composed];
  if (process.argv.includes('--write')) fs.writeFileSync(`${out}/manifest.json`,JSON.stringify(report,null,2)+'\n');
  console.log(JSON.stringify({status:report.status,sources:report.sourcePins.length,receipts:report.receiptPins.length,
    preserved:report.preservedBasePins.length,composed:report.composedFiles,unexpected:report.unexpectedSource,mismatches}));
  if (report.status !== 'passed') process.exitCode = 1;
} finally { fs.rmSync(scratch,{recursive:true}); }

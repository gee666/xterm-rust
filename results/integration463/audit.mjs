// Reuse integration448's exact-blob audit. Combined files are derived from
// preserved producer objects, never from the assembled checkout.
import fs from 'node:fs';
import crypto from 'node:crypto';
import {execFileSync} from 'node:child_process';
const cache = new Map();
function git(...args) {
  const key = JSON.stringify(args);
  if (!cache.has(key)) cache.set(key, execFileSync(process.env.GIT ?? 'git', args,
    {timeout: 30000, maxBuffer: 32 * 1024 * 1024}));
  return cache.get(key);
}
const text = (...args) => git(...args).toString().trim();
const trees = new Map();
function exists(ref, file) {
  if (!trees.has(ref)) trees.set(ref, new Set(text('ls-tree', '-r', '--name-only', ref).split('\n')));
  return trees.get(ref).has(file);
}
function prime(specs) {
  const unique = [...new Set(specs)];
  const output = execFileSync(process.env.GIT ?? 'git', ['cat-file', '--batch'],
    {input: unique.join('\n') + '\n', timeout: 30000, maxBuffer: 128 * 1024 * 1024});
  let cursor = 0;
  for (const spec of unique) {
    const end = output.indexOf(10, cursor), header = output.subarray(cursor, end).toString();
    const match = /^\S+ blob (\d+)$/.exec(header);
    if (!match) throw Error(`Invalid object ${spec}: ${header}`);
    const size = Number(match[1]), start = end + 1;
    cache.set(JSON.stringify(['show', spec]), output.subarray(start, start + size));
    cursor = start + size + 1;
  }
  if (cursor !== output.length) throw Error('Unexpected batch object suffix');
}
const sha = bytes => crypto.createHash('sha256').update(bytes).digest('hex');
const plan = JSON.parse(fs.readFileSync('results/integration463/plan.json'));
const blob = (ref, file) => git('show', `${ref}:${file}`).toString();
const combined = new Map();
function replace(value, before, after) {
  if (value.split(before).length !== 2) throw Error(`Ambiguous replacement: ${before}`);
  return value.replace(before, after);
}
function segment(value, start, end) {
  const from = value.indexOf(start), to = value.indexOf(end, from + start.length);
  if (from < 0 || to < 0) throw Error(`Missing producer segment ${start}`);
  return value.slice(from, to);
}
const font = 'crates/xterm-addons/src/ligatures/';
const gpos = blob('5dd73902', font + 'font_bytes.rs');
let bytes = blob('f9a29280', font + 'font_bytes.rs');
bytes = replace(bytes, '    KernVersion(u16),', '    KernVersion(u16),\n' +
  segment(gpos, '    GposVersion {', '    CoverageFormat {').trimEnd());
bytes = replace(bytes, '            Self::CoverageFormat {',
  segment(gpos, '            Self::GposVersion {', '            Self::CoverageFormat {') +
  '            Self::CoverageFormat {');
bytes = replace(bytes, '    pub gdef: Option<DeferredTable<\'a>>,',
  '    pub gdef: Option<DeferredTable<\'a>>,\n    pub gpos: Option<DeferredTable<\'a>>,');
bytes = replace(bytes, '        let mut gdef = None;',
  '        let mut gdef = None;\n        let mut gpos = None;');
// Task457 already contains420's hint hook using442's refreshed payload.
bytes = replace(bytes,
  '                b"name" | b"GSUB" | b"GDEF" | b"hmtx" | b"kern" | b"meta" | b"fvar" | b"avar" => {',
  '                b"name" | b"GSUB" | b"GDEF" | b"GPOS" | b"hmtx" | b"kern" | b"meta" | b"fvar"\n                | b"avar" => {');
bytes = replace(bytes, '                        b"GDEF" => gdef = Some(record),',
  '                        b"GDEF" => gdef = Some(record),\n                        b"GPOS" => gpos = Some(record),');
bytes = replace(bytes, '        Ok((\n            Self {',
  '        let deferred = |record| DeferredTable { record, woff };\n        Ok((\n            Self {');
bytes = bytes.replaceAll('.map(|record| DeferredTable { record, woff })', '.map(deferred)');
bytes = replace(bytes, '                gdef: gdef.map(deferred),',
  '                gdef: gdef.map(deferred),\n                gpos: gpos.map(deferred),');
const error = segment(bytes, '#[derive(Clone, Debug, Eq, PartialEq)]', '#[derive(Clone, Copy)]');
combined.set(font + 'font_error.rs', Buffer.from(
  '// Copyright (c) 2018 The xterm.js authors. All rights reserved. MIT license.\nuse std::fmt;\n\n' + error.trimEnd() + '\n'));
bytes = replace(bytes, error, '');
bytes = replace(bytes, 'use super::{font_cmap::Cmap, font_warning::FontLoadError, FontWarning};\nuse std::{borrow::Cow, fmt};',
  'pub use super::font_error::FontError;\nuse super::{font_cmap::Cmap, font_warning::FontLoadError, FontWarning};\nuse std::borrow::Cow;');
combined.set(font + 'font_bytes.rs', Buffer.from(bytes));
let loader = blob('f9a29280', font + 'font.rs');
loader = replace(loader, '#[path = "font_fvar.rs"]', '#[path = "font_error.rs"]\nmod font_error;\n#[path = "font_fvar.rs"]');
loader = replace(loader, '#[path = "font_gsub.rs"]',
  '#[path = "font_gpos.rs"]\nmod font_gpos;\n#[path = "font_gsub.rs"]');
loader = replace(loader, '        let current = refreshed.as_deref().unwrap_or(bytes);\n        let gsub = tables',
  '        if let Some(table) = tables.gpos {\n            font_gpos::validate(&table, bytes, &mut refreshed, &mut warning)?;\n        }\n        let current = refreshed.as_deref().unwrap_or(bytes);\n        let gsub = tables');
combined.set(font + 'font.rs', Buffer.from(loader));
let cases = blob('76635fde', 'scripts/serialize-browser-cases.mjs');
cases = replace(cases,
  '// This cohesive fixture exceeds350 lines because Playwright serializes this\n// function, including its helpers, into each isolated engine page.',
  '// Cohesive browser fixture exceeds 350 lines because Playwright serializes this\n// function into each realm; local helpers and directed cases must travel with it.');
for (const [ref, start, end] of [
  ['4ba540d4', "  await run('html-reversed-numeric-row-bounds'", "  await run('html-nan-row-bounds'"],
  [plan.base, "  for (const phase of ['header', 'endLine'])", "  await run('html-header-alt-view'"]
]) {
  cases = replace(cases, end, segment(blob(ref, 'scripts/serialize-browser-cases.mjs'), start, end) + end);
}
combined.set('scripts/serialize-browser-cases.mjs', Buffer.from(cases));
combined.set('scripts/serialize-browser-check.mjs', Buffer.from(replace(
  blob('76635fde', 'scripts/serialize-browser-check.mjs'),
  "66, 'Expected34 directed and32 seeded cases'", "70, 'Expected38 directed and32 seeded cases'")));
const source = file => ['crates/', 'scripts/', '.github/', '.pi/swarm/skills/'].some(p => file.startsWith(p));
const expected = new Map(), owners = new Map(), report = {base: plan.base,
  assembly: text('rev-parse', 'HEAD'), batches: [], sourcePins: [], receiptPins: [],
  preservedBasePins: [], pointerExceptions: ['results/demos/build/latest.json']};
for (const batch of plan.batches) {
  const chain = batch.chain.map(ref => text('rev-parse', ref));
  const paths = [...new Set(chain.flatMap(ref => text('diff-tree', '--root', '--no-commit-id',
    '--name-only', '-r', ref).split('\n').filter(Boolean)))];
  if (batch.receiptOnly && paths.some(source)) throw Error(`Receipt-only task${batch.task} changes source`);
  report.batches.push({...batch, chain, paths: paths.length});
  for (const file of paths) {
    if (file === 'results/demos/build/latest.json') continue;
    const present = exists(chain.at(-1), file);
    expected.set(file, {task: batch.task, ref: chain.at(-1), present});
    if (source(file)) owners.set(file, [...new Set([...(owners.get(file) ?? []), batch.task])]);
  }
}
expected.set(font + 'font_error.rs', {task: 463, ref: null, present: true});
const baseline = JSON.parse(git('show', `${plan.base}:results/integration448/manifest.json`));
const oldPins = [...baseline.sourcePins, ...baseline.preservedBasePins];
prime([...expected].filter(([file, entry]) => entry.present && (source(file) || file.startsWith('results/')))
  .flatMap(([file, entry]) => [...(entry.ref ? [`${entry.ref}:${file}`] : []), `HEAD:${file}`])
  .concat(oldPins.flatMap(({path: file}) => [`${plan.base}:${file}`, `HEAD:${file}`])));
for (const [file, entry] of expected) {
  if (!source(file) && !file.startsWith('results/')) continue;
  const actualPresent = exists('HEAD', file);
  if (!entry.present) {
    report.receiptPins.push({path: file, deleted: true, matches: !actualPresent});
    continue;
  }
  const producer = entry.ref ? git('show', `${entry.ref}:${file}`) : combined.get(file);
  const wanted = combined.get(file) ?? producer;
  const actual = git('show', `HEAD:${file}`);
  const pin = {task: entry.task, path: file, producerSha256: sha(producer), expectedSha256: sha(wanted),
    headSha256: sha(actual), composed: combined.has(file), matches: wanted.equals(actual)};
  if (source(file)) {
    const working = fs.readFileSync(file);
    pin.checkoutEolOnly = !wanted.equals(working) && file.endsWith('.md') &&
      wanted.toString().replaceAll('\r\n', '\n') === working.toString().replaceAll('\r\n', '\n');
    pin.matches &&= wanted.equals(working) || pin.checkoutEolOnly;
    pin.owners = owners.get(file);
    report.sourcePins.push(pin);
  } else report.receiptPins.push(pin);
}
for (const {path: file} of oldPins) {
  const updated = report.sourcePins.find(pin => pin.path === file);
  const wanted = updated?.expectedSha256 ?? sha(git('show', `${plan.base}:${file}`));
  const actual = sha(git('show', `HEAD:${file}`));
  report.preservedBasePins.push({path: file, updated: updated?.task ?? null, expectedSha256: wanted,
    headSha256: actual, matches: wanted === actual});
}
report.receiptPins.push({path: 'results/demos/build/latest.json', preservedMainPointer: true,
  matches: git('show', `${plan.base}:results/demos/build/latest.json`).equals(git('show', 'HEAD:results/demos/build/latest.json'))});
report.unexpectedSource = text('diff', '--name-only', plan.base, 'HEAD').split('\n')
  .filter(file => source(file) && !expected.has(file));
if (report.unexpectedSource.length) throw Error(`Unexpected source files: ${report.unexpectedSource.join(', ')}`);
const mismatches = [...report.sourcePins, ...report.receiptPins, ...report.preservedBasePins].filter(p => !p.matches);
report.status = mismatches.length ? 'failed' : 'passed';
if (process.argv.includes('--write')) fs.writeFileSync('results/integration463/manifest.json', JSON.stringify(report, null, 2) + '\n');
console.log(JSON.stringify({status: report.status, sources: report.sourcePins.length,
  composed: combined.size, receipts: report.receiptPins.length, preserved: report.preservedBasePins.length, mismatches}));
if (mismatches.length) process.exitCode = 1;

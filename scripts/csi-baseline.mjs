// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
// Execute all applicable original assertion bodies without changing the reference.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
const root = resolve(import.meta.dirname, '..');
const output = resolve(root, 'results/csi');
mkdirSync(output, { recursive: true });
writeFileSync(resolve(output, 'baseline.json'), JSON.stringify({ status: 'incomplete', started: new Date().toISOString() }) + '\n');
const temporary = resolve(root, 'tmp/csi-baseline');
mkdirSync(temporary, { recursive: true });
const deps = resolve(root, 'source_repo/node_modules');
const { build } = await import(pathToFileURL(resolve(deps, 'esbuild/lib/main.js')));
const { default: Mocha } = await import(pathToFileURL(resolve(deps, 'mocha/index.js')));
const bundled = resolve(temporary, 'InputHandler.test.cjs');
const built = await build({ absWorkingDir: root,
  entryPoints: ['source_repo/src/common/InputHandler.test.ts'], outfile: bundled, bundle: true,
  platform: 'node', format: 'cjs', tsconfig: 'source_repo/src/common/tsconfig.json', metafile: true });
const grep = /^InputHandler (?:SL\/SR\/DECIC\/DECDC |regression tests (?:insertChars|deleteChars)$|print should (?:repeat preceding grapheme cluster via REP|not repeat when REP has no preceding join state|not repeat after an intervening escape sequence)$|cursor positioning |DECSTBM - scroll margins |scroll margins |should correctly reset cells taken by wide chars (?:ICH|DCH)$)/;
const mocha = new Mocha({ reporter: 'spec', grep });
mocha.addFile(bundled);
await mocha.loadFilesAsync();
const outcomes = [];
let runner;
const failed = await new Promise(done => {
  runner = mocha.run(done);
  runner.on('pass', test => outcomes.push({ name: test.fullTitle(), status: 'passed' }));
  runner.on('pending', test => outcomes.push({ name: test.fullTitle(), status: 'skipped' }));
  runner.on('fail', (test, error) => outcomes.push({ name: test.fullTitle(), status: 'failed', error: String(error.stack) }));
});
const names = {
  'SL (scrollLeft)': 'scroll_left', 'SR (scrollRight)': 'scroll_right',
  'insertColumns (DECIC)': 'insert_columns', 'deleteColumns (DECDC)': 'delete_columns',
  'insertChars': 'insert_and_delete_characters_regression', 'deleteChars': 'insert_and_delete_characters_regression',
  'should repeat preceding grapheme cluster via REP': 'repeat_preceding_grapheme',
  'should not repeat when REP has no preceding join state': 'repeat_without_predecessor',
  'should not repeat after an intervening escape sequence': 'intervening_sequence_resets_repeat',
  'cursor forward (CUF)': 'cursor_forward_and_horizontal_relative',
  'cursor backward (CUB)': 'cursor_backward', 'cursor down (CUD)': 'cursor_down_and_vertical_relative',
  'cursor up (CUU)': 'cursor_up', 'cursor next line (CNL)': 'cursor_next_line',
  'cursor previous line (CPL)': 'cursor_previous_line', 'cursor character absolute (CHA)': 'horizontal_absolute_aliases',
  'cursor position (CUP)': 'cursor_position', 'cursor position (CUP) with DECOM and scroll margins': 'cursor_position_origin_and_scroll_margins',
  'horizontal position absolute (HPA)': 'horizontal_absolute_aliases',
  'horizontal position relative (HPR)': 'cursor_forward_and_horizontal_relative',
  'vertical position absolute (VPA)': 'vertical_absolute', 'vertical position relative (VPR)': 'cursor_down_and_vertical_relative',
  'should default to whole viewport': 'scroll_margin_defaults', 'should clamp bottom': 'scroll_margin_clamps_bottom',
  'should only apply for top < bottom': 'scroll_margin_rejects_top_at_or_below_bottom', 'should home cursor': 'scroll_margin_homes_cursor',
  'scrollUp': 'scroll_up', 'scrollDown': 'scroll_down',
  'insertLines - out of margins': 'insert_and_delete_lines_outside_margins',
  'deleteLines - out of margins': 'insert_and_delete_lines_outside_margins',
  'insertLines - within margins': 'insert_lines_within_margins', 'deleteLines - within margins': 'delete_lines_within_margins',
};
const mapping = outcomes.map(test => {
  const clamp = test.name.includes('should clamp cursor into addressable range');
  const lastCell = test.name.includes('should delete last cell');
  let rustTest;
  if (clamp) rustTest = lastCell ? 'pending_wrap_edit_deletes_last_cell' : 'clamp_positive_out_of_range_cursor';
  else if (test.name.includes('should correctly reset cells taken by wide chars')) {
    rustTest = test.name.endsWith(' ICH') ? 'insert_characters_repairs_wide_boundaries' : 'delete_characters_repairs_wide_boundaries';
  } else rustTest = Object.entries(names).find(([suffix]) => test.name.endsWith(' ' + suffix))?.[1];
  assert(rustTest, `No Rust mapping for ${test.name}`);
  const partial = clamp && !lastCell;
  return { source: test.name, rustTest, status: partial ? 'partial' : 'mapped',
    gap: partial ? 'Positive out-of-range assertion is ported. Negative direct Buffer cursor mutation is unrepresentable in current Rust usize fields; negative precondition/assertion is explicitly unrun.' : null };
});
const report = { command: 'node scripts/csi-baseline.mjs', node: process.version, platform: process.platform,
  status: failed || runner.stats.pending ? 'failed' : 'passed', overallEquivalence: 'NOT PROVEN',
  sourceHashes: Object.fromEntries(Object.keys(built.metafile.inputs).filter(path => path.startsWith('source_repo/src/'))
    .map(path => [path, createHash('sha256').update(readFileSync(resolve(root, path))).digest('hex')])),
  original: { discovered: runner.total, executed: runner.stats.tests - runner.stats.pending,
    passed: runner.stats.passes, failed, skipped: runner.stats.pending, unrun: runner.total - runner.stats.tests },
  rustMigration: { mapped: mapping.filter(x => x.status === 'mapped').length,
    partial: mapping.filter(x => x.status === 'partial').length, unmapped: 0,
    negativeInternalCursorAssertionsUnrun: mapping.filter(x => x.status === 'partial').length }, mapping, outcomes,
  limits: ['This is scoped original JavaScript execution and mapping, not a claim Rust passed.',
    'All other InputHandler families belong to engine/SGR/modes/erase/OSC owners.',
    'Typed Rust internal-cursor negative-state gap is retained rather than weakening source assertions.'] };
writeFileSync(resolve(output, 'baseline.json'), JSON.stringify(report, null, 2) + '\n');
console.log(JSON.stringify({ original: report.original, migration: report.rustMigration }));
assert.equal(report.status, 'passed');

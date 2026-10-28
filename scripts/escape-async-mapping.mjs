// Copyright (c) 2018 The xterm.js authors. All rights reserved. MIT license.
// Verify an explicit migration for every original outer-parser async test.
import assert from 'node:assert/strict';
import { readFile, mkdir, writeFile } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import ts from '../source_repo/node_modules/typescript/lib/typescript.js';
const sourcePath = 'source_repo/src/common/parser/EscapeSequenceParser.test.ts';
const source = await readFile(sourcePath, 'utf8');
const syntax = ts.createSourceFile(sourcePath, source, ts.ScriptTarget.Latest, true);
const rustPath = 'crates/xterm-core/tests/escape_async.rs';
const rust = await readFile(rustPath, 'utf8');
const rules = new Map([
  ['sync handlers keep being parsed in sync mode', 'sync_handlers_keep_sync_mode_and_produce_original_result'],
  ['correct result on sync parse call', 'sync_handlers_keep_sync_mode_and_produce_original_result'],
  ['correct result on async parse call', 'sync_handlers_keep_sync_mode_and_produce_original_result'],
  ['sync parse call does not work anymore', 'async_handlers_require_continuation_and_remain_failed_until_reset'],
  ['improper continuation should throw', 'async_handlers_require_continuation_and_remain_failed_until_reset'],
  ['reset during async pause continues at next codepoint in chunk', 'reset_during_async_pause_continues_at_next_codepoint_in_same_chunk'],
  ['correct result on awaited parse call', 'awaited_parse_preserves_original_results_and_exact_stack_positions'],
  ['correct result on chunked awaited parse calls', 'one_codepoint_chunks_preserve_original_callback_splitting'],
  ['multiple async SGR handlers', 'multiple_async_handlers_bubble_suppress_and_dispose_in_all_five_families'],
  ['multiple async ESC handlers', 'multiple_async_handlers_bubble_suppress_and_dispose_in_all_five_families'],
  ['sync/async SGR mixed', 'mixed_sync_async_sgr_handlers_keep_original_order_and_stack_positions'],
  ['multiple async OSC handlers', 'multiple_async_handlers_bubble_suppress_and_dispose_in_all_five_families'],
  ['multiple async DCS handlers', 'multiple_async_handlers_bubble_suppress_and_dispose_in_all_five_families'],
  ['multiple async APC handlers', 'multiple_async_handlers_bubble_suppress_and_dispose_in_all_five_families'],
]);
const mapping = [];
function visit(node, active = false) {
  if (ts.isCallExpression(node) && node.expression.getText(syntax) === 'describe' && node.arguments[0]?.text === 'EscapeSequenceParser - async') active = true;
  if (active && ts.isCallExpression(node) && node.expression.getText(syntax) === 'it') {
    const name = node.arguments[0].text;
    const rustTest = rules.get(name);
    assert(rustTest, `unmapped async source test ${name}`);
    assert(rust.includes(`fn ${rustTest}(`), `missing Rust test ${rustTest}`);
    mapping.push({ name, sourceLine: syntax.getLineAndCharacterOfPosition(node.getStart(syntax)).line + 1, rustPath, rustTest });
  }
  ts.forEachChild(node, child => visit(child, active));
}
visit(syntax);
assert.equal(mapping.length, 14);
const result = { sourcePath, sourceSha256: createHash('sha256').update(source).digest('hex'),
  rustPath, rustSha256: createHash('sha256').update(rust).digest('hex'),
  originalAsyncTests: mapping.length, mappedRustFunctions: new Set(mapping.map(row => row.rustTest)).size,
  contract: 'Callback order, exact original full/chunked RESULT, pause kind/position/handler index, true/false bubbling, disposal and reset assertions are executed in Rust. JavaScript promises map to caller-driven Rust futures.', mapping };
await mkdir('results/escape-parser', { recursive: true });
await writeFile('results/escape-parser/async-mapping.json', JSON.stringify(result, null, 2) + '\n');
console.log(JSON.stringify({ originalAsyncTests: result.originalAsyncTests, mappedRustFunctions: result.mappedRustFunctions }));

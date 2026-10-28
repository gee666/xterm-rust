// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
// Preserve each upstream synchronous test's identity even when Rust loops share setup.
import assert from 'node:assert/strict';
import { readFile, writeFile, mkdir } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import ts from '../source_repo/node_modules/typescript/lib/typescript.js';
const sourcePath = 'source_repo/src/common/parser/EscapeSequenceParser.test.ts';
const source = await readFile(sourcePath, 'utf8');
const mapping = [];
let exampleIndex = 0;
const sourceHash = createHash('sha256').update(source).digest('hex');
assert.equal(sourceHash, 'dd1ea180ec271a2807435f73108b1398d5252e85def8f1f0ade57c6ac1a3434a', 'review mapping boundaries after an upstream test change');
const syntax = ts.createSourceFile(sourcePath, source, ts.ScriptTarget.Latest, true);
const discovered = [];
function discover(node) {
  if (ts.isCallExpression(node) && node.expression.getText(syntax) === 'it' && ts.isStringLiteral(node.arguments[0])) {
    const assertions = [];
    const helpers = [];
    function calls(child) {
      if (ts.isCallExpression(child)) {
        const name = child.expression.getText(syntax);
        const line = syntax.getLineAndCharacterOfPosition(child.getStart(syntax)).line + 1;
        if (name.startsWith('assert.')) assertions.push({ line, name });
        if (name === 'testTerminal.compare' || name === 'test') helpers.push({ line, name });
      }
      ts.forEachChild(child, calls);
    }
    ts.forEachChild(node.arguments[1], calls);
    discovered.push({ name: node.arguments[0].text, sourceLine: syntax.getLineAndCharacterOfPosition(node.getStart(syntax)).line + 1, assertionSites: assertions, helperExpectations: helpers });
  }
  ts.forEachChild(node, discover);
}
discover(syntax);
function stateTarget(name) {
  if (/ANYWHERE --> GROUND|ANYWHERE --> ESCAPE/.test(name)) return 'states::original_anywhere_cancel_st_and_escape_clear_rules';
  if (/ANYWHERE\/ESCAPE|^state (DCS_ENTRY|APC_ENTRY)$/.test(name)) return 'states::original_entry_clear_and_sos_pm_transitions';
  if (/OSC_STRING (ignore|put)/.test(name)) return 'strings::original_osc_raw_put_and_ignore_expectations_with_real_dispatch';
  if (/execute|GROUND print/.test(name)) return 'states::original_executable_state_rules_and_ground_printables';
  if (/ecs_dispatch|esc_dispatch/.test(name)) return 'states::original_esc_dispatch_ranges';
  if (/csi_dispatch|CSI_IGNORE --> GROUND/.test(name)) return 'parameters::original_csi_dispatch_and_ignore_exit_rules';
  if (/param\/collect|param action|for ":"/.test(name)) return 'parameters::original_parameter_and_prefix_rules';
  if (/collect/.test(name)) return 'parameters::original_collect_ranges';
  if (name.startsWith('trans ') && name.includes('-->') && /CSI_IGNORE|DCS_IGNORE/.test(name)) return 'parameters::original_ignore_entry_from_parameter_and_intermediate';
  if (/ignore/.test(name)) return 'states::original_ignore_rules';
  if (/DCS_PASSTHROUGH|APC_PASSTHROUGH/.test(name)) return 'strings::original_dcs_apc_start_put_and_end_rules';
  throw new Error(`unmapped state test: ${name}`);
}
for (const { sourceLine, name, assertionSites, helperExpectations } of discovered) {
  if (sourceLine >= 2067) { mapping.push({ sourceLine, name, assertionSites, helperExpectations, ownerTask: 26, status: 'async-owned-separately' }); continue; }
  let rustTest;
  if (sourceLine < 266) rustTest = 'states::construction_initial_and_reset_states';
  else if (sourceLine < 1187) rustTest = stateTarget(name);
  else if (sourceLine < 1351) rustTest = `examples::example_${(exampleIndex++).toString().padStart(2, '0')}`;
  else if (sourceLine < 1394) rustTest = 'strings::original_unicode_coverage_cases';
  else if (sourceLine < 1880) {
    const direct = {
      'print handler': 'basic_handlers::original_print_execute_and_esc_set_clear_handlers',
      'ESC handler': 'basic_handlers::original_print_execute_and_esc_set_clear_handlers',
      'EXECUTE handler': 'basic_handlers::original_print_execute_and_esc_set_clear_handlers',
      'CSI handler': 'registrations::original_csi_osc_set_clear_handlers',
      'OSC handler': 'registrations::original_csi_osc_set_clear_handlers',
      'DCS handler': 'basic_handlers::original_dcs_apc_raw_streaming_set_clear_handlers',
      'APC handler': 'basic_handlers::original_dcs_apc_raw_streaming_set_clear_handlers',
      'ERROR handler': 'basic_handlers::original_error_handler_diagnostic_and_clear',
    };
    rustTest = direct[name] ?? (name.startsWith('Execution order') ? 'registrations::original_exact_latest_to_original_order' : 'registrations::original_custom_handler_bubbling_order_and_disposal_all_five_families');
  } else if (sourceLine < 1926) rustTest = 'identifiers::original_prefix_intermediate_and_final_limits';
  else if (sourceLine < 1934) rustTest = 'identifiers::original_identifier_packing_order';
  else rustTest = 'identifiers::original_identifier_invocations_all_four_families';
  const [module, test] = rustTest.split('::');
  const file = `crates/xterm-core/tests/escape_upstream_support/${module}.rs`;
  assert((await readFile(file, 'utf8')).includes(`fn ${test}(`), `missing ${rustTest}`);
  mapping.push({ sourceLine, name, assertionSites, helperExpectations, rustFile: file, rustTest, ownerTask: 90, status: 'mapped' });
}
const result = { sourcePath, sourceSha256: sourceHash, typescript: ts.version,
  staticAssertionSites: mapping.reduce((sum, row) => sum + row.assertionSites.length, 0),
  staticHelperExpectations: mapping.reduce((sum, row) => sum + row.helperExpectations.length, 0),
  originalDiscovered: mapping.length, synchronousMapped: mapping.filter(row => row.status === 'mapped').length,
  asyncOwnedSeparately: mapping.filter(row => row.status === 'async-owned-separately').length,
  fixtureAdaptations: [
    'Rust typed states cannot contain the private invalid number124. Reset is exercised after parsed nondefault state/parameters/collect instead. Live OSC payload # is reset, old end emits no event, and a fresh OSC completes without stale payload.',
    'OSC raw put/ignore assertions use a real OSC payload initialized with0; rather than replacing the concrete subparser with a mock.',
    'Representation-only constant placeholder fields in raw DCS/APC test callback arrays become typed event variants. Payload/parameters/order/success expectations are preserved.',
    'The original ERROR callback returns a state object with abortfalse. Rust returns false and preserves every diagnostic field.'
  ], mapping };
assert.equal(result.originalDiscovered, 161);
assert.equal(exampleIndex, 21);
await mkdir('results/escape-upstream', { recursive: true });
await writeFile('results/escape-upstream/mapping.json', JSON.stringify(result, null, 2) + '\n');
console.log(JSON.stringify({ originalDiscovered: result.originalDiscovered, synchronousMapped: result.synchronousMapped, asyncOwnedSeparately: result.asyncOwnedSeparately }));

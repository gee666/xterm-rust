// Copyright (c) 2014-2026 The xterm.js authors. All rights reserved. MIT license.
// Source-derived erase case inventory, including explicit partial/unrun cases.
import assert from 'node:assert/strict';
import { readFile,writeFile,mkdir } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import ts from '../source_repo/node_modules/typescript/lib/typescript.js';
const sourceFiles=['source_repo/src/common/InputHandler.test.ts','source_repo/test/playwright/InputHandler.test.ts'];
const sources=[];const rows=[];
for(const path of sourceFiles) {
  const source=await readFile(path,'utf8');sources.push({path,sha256:createHash('sha256').update(source).digest('hex')});
  const syntax=ts.createSourceFile(path,source,ts.ScriptTarget.Latest,true);
  function visit(node,suites=[]) {
    if(ts.isCallExpression(node)) {
      const call=node.expression.getText(syntax),name=node.arguments[0]?.text;
      if(call==='describe'||call==='test.describe')suites=[...suites,name];
      if((call==='it'||call==='test')&&typeof name==='string') {
        const line=syntax.getLineAndCharacterOfPosition(node.getStart(syntax)).line+1;
        let rustTest,status='assertions-mapped',rustFile='crates/xterm-core/tests/csi_erase_cases.rs';
        if(path.includes('/common/')) {
          const base=new Map([[398,'original_erase_in_line_assertions'],[435,'original_erase_in_line_reflow_assertions'],[485,'original_ed2_scroll_on_erase_assertions'],[516,'original_erase_in_display_assertions_and_wrap_cases']]);
          if(base.has(line)){rustTest=base.get(line);rustFile='crates/xterm-core/tests/csi_erase_upstream.rs';}
          else if(line===1456||line===1466){rustTest='original_ech_last_cell_and_large_positive_cursor_restriction';if(line===1456)status='partial-negative-private-coordinate-task152';}
          else if(line===1771)rustTest='original_wide_el_cell_repairs';
          else if(line===1797)rustTest='original_wide_ech_cell_repairs';
          else if(line>=2312&&line<2397)rustTest=name==='cursor never advances beyond cols'?'original_following_sequences_never_advance_past_columns':'original_pending_wrap_ed_el_cursor_positions_and_rows';
          else if(line>=2397&&line<2445)rustTest=name.includes('resized alt')?'original_alt_ed3_preserves_normal_user_viewport_after_resizing':'original_ed3_unlocks_normal_viewport_and_future_scroll_follows_bottom';
          else if(line>=2445&&line<2470)rustTest='original_decsca_default_and_selective_line_display_erasure';
          else if(line>=2470&&line<2487)rustTest='original_decrqss_reports_protection_default_set_and_reset';
        }else {
          rustFile='crates/xterm-core/tests/csi_erase_playwright_assertions.rs';
          if(line===114||line===134)rustTest='original_browser_ed_and_decsed_content_assertions';
          else if(line===154||line===172)rustTest='original_browser_el_and_decsel_content_assertions';
          else if(line===249)rustTest='original_browser_ech_content_assertions';
          else if(line===1534)rustTest='original_browser_decsca_content_assertions';
          if(rustTest)status='native-assertions-mapped-actual-browser-unrun';
        }
        if(rustTest) {
          const assertions=[];
          function calls(child) {
            if(ts.isCallExpression(child)) {
              const call=child.expression.getText(syntax);
              if(call.startsWith('assert.')||call==='pollFor')assertions.push({line:syntax.getLineAndCharacterOfPosition(child.getStart(syntax)).line+1,call,source:child.getText(syntax)});
            }
            ts.forEachChild(child,calls);
          }
          ts.forEachChild(node.arguments[1],calls);
          rows.push({source:path,line,name:[...suites,name].join(' / '),rustFile,rustTest,status,assertions});
        }
      }
    }
    ts.forEachChild(node,child=>visit(child,suites));
  }
  visit(syntax);
}
for(const row of rows)assert((await readFile(row.rustFile,'utf8')).includes(`fn ${row.rustTest}(`),`missing ${row.rustTest}`);
const common=rows.filter(row=>row.source.includes('/common/'));
const browser=rows.filter(row=>row.source.includes('/playwright/'));
assert.equal(common.length,22);assert.equal(browser.length,6);
const rawBaseline=await readFile('results/csi-erase/original-input-handler.json','utf8');
const baselineStart=rawBaseline.indexOf('{');assert(baselineStart>=0,'missing original reporter object');
const baseline=JSON.parse(rawBaseline.slice(baselineStart)).stats;
const report={sources,typescript:ts.version,originalUnitBaseline:baseline,staticCommonAssertionSites:common.reduce((n,row)=>n+row.assertions.length,0),staticBrowserAssertionSites:browser.reduce((n,row)=>n+row.assertions.length,0),assertionCountNote:'Static call sites, not dynamic assertion execution counts; loop expansions remain explicit in source and Rust tests.',commonSourceCases:common.length,commonFullyMapped:common.filter(row=>row.status==='assertions-mapped').length,commonPartial:common.filter(row=>row.status!=='assertions-mapped').length,browserSourceCases:browser.length,browserNativeAssertionMappings:browser.length,browserActualExecution:'UNRUN in this scoped result; source/Rust-backed browser matrix has separate tasks',sourceFixtureInventory:['t0054-ECH','t0055-EL','t0056-ED','t0057-ED3','t0091-alt_screen_ED3'],fixtureNote:'EL is explicitly skipped by original Terminal2.test.ts, which skips all PTY cases on Windows. Native ONLCR replay is a separate check, not native PTY or browser proof.',rows};
await mkdir('results/csi-erase',{recursive:true});await writeFile('results/csi-erase/mapping.json',JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify({common:report.commonSourceCases,fullyMapped:report.commonFullyMapped,partial:report.commonPartial,browserNative:report.browserNativeAssertionMappings}));

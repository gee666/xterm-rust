// Copyright (c) 2014-2026 The xterm.js authors. All rights reserved. MIT license.
import test from 'node:test';
import assert from 'node:assert/strict';
import {caseIds,scenarios} from './acceptance-adapters/terminal/replies-window.inputs.mjs';
import {tracesForCase} from './acceptance-adapters/terminal/replies-window.cases.mjs';
import {protocol} from './full-trace-capture.mjs';

for(const caseId of caseIds){
    test(`${caseId} preserves every generated operation through the shared protocol`,()=>{
        const traces=tracesForCase(caseId),expected=scenarios(caseId);
        assert.equal(traces.length,expected.length);
        assert.equal(protocol(traces),expected.map(item=>item.input).join(''));
        assert.ok(traces.every(trace=>trace.ops.length>0));
    });
}
test('case ownership is exact and rejects unknown IDs',()=>{
    assert.deepEqual(caseIds,['vt-32','vt-33','vt-35','vt-36','vt-37','vt-38','vt-39','vt-42','vt-43']);
    assert.equal(new Set(caseIds).size,9);
    assert.throws(()=>tracesForCase('vt-34'),/unsupported/);
});
test('title traces preserve unpaired UTF-16 units and overflow the ten-entry stacks',()=>{
    for(const trace of tracesForCase('vt-38')){
        const writes=trace.ops.filter(op=>op[0]==='W').map(op=>op[1]);
        assert.ok(writes.some(text=>text.includes('\ud800')));
        assert.equal(writes.filter(text=>/^\x1b\[22;[012]t$/.test(text)).length,13);
        assert.equal(writes.filter(text=>/^\x1b\[23;[012]t$/.test(text)).length,16);
    }
});
test('each window permission has independent disabled and enabled writes',()=>{
    const traces=tracesForCase('vt-39');assert.equal(traces.length,23);
    for(const trace of traces){
        const bags=trace.ops.filter(op=>op[0]==='O'&&op[1]==='windowOptions').map(op=>op[2]);
        assert.equal(Object.values(bags[0])[0],false);
        assert.equal(Object.values(bags[1])[0],true);
        assert.equal(Object.values(bags[2])[0],false);
    }
});

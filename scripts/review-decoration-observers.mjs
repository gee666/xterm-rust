// Independent regressions. Run decoration-oracle.mjs first to rebuild the
// read-only upstream runtime under tmp/decoration-oracle/runtime.cjs.
import { writeFile } from 'node:fs/promises';
import { pathToFileURL } from 'node:url';
import { resolve } from 'node:path';
import assert from 'node:assert/strict';
const { DecorationService, Buffer, OptionsService } = await import(pathToFileURL(resolve('tmp/decoration-oracle/runtime.cjs')));
const options = new OptionsService();
const buffer = new Buffer(true, options, { cols: 80, rows: 24 });
const log = { debug() {}, info() {}, warn() {}, error() {}, trace() {}, logLevel: 0 };
const service = new DecorationService(log, { buffer, buffers: { onBufferActivate: () => ({ dispose() {} }) } });
const iterator = service.getDecorationsAtCell(0, 0);
const decoration = service.registerDecoration({ marker: buffer.addMarker(0) });
const sourceColdIteratorSeesLateRegistration = iterator.next().value === decoration;
assert.equal(sourceColdIteratorSeesLateRegistration, true);
const bucket = service._lineCache.getDecorationsOnLine(0);
decoration.dispose();
const sourceHeldReadonlyBucketLengthAfterDispose = bucket.length;
assert.equal(sourceHeldReadonlyBucketLengthAfterDispose, 0);
assert.equal(iterator.next().done, true);
service.registerDecoration({ marker: buffer.addMarker(0) });
const sourceExhaustedIteratorRemainsDone = iterator.next().done;
assert.equal(sourceExhaustedIteratorRemainsDone, true);
service.dispose();
const result = { sourceColdIteratorSeesLateRegistration, sourceHeldReadonlyBucketLengthAfterDispose, sourceExhaustedIteratorRemainsDone };
await writeFile('results/decoration/review-source-observers.json', JSON.stringify(result, null, 2) + '\n');
console.log(JSON.stringify(result));

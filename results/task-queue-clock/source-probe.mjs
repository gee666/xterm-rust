// Retained counterexample for open task367, not a passing parity test.
import fs from 'node:fs';
import path from 'node:path';
import vm from 'node:vm';
import {createHash} from 'node:crypto';
import {createRequire} from 'node:module';
const require = createRequire(import.meta.url);
const root = process.env.XT_SOURCE_ROOT;
if (!root) throw new Error('Set XT_SOURCE_ROOT to the unchanged source_repo with TypeScript installed');
const ts = require(path.resolve(root, 'node_modules/typescript'));
const source = fs.readFileSync(path.join(root, 'src/common/TaskQueue.ts'), 'utf8');
const js = ts.transpileModule(source, {
  compilerOptions: {module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2022}
}).outputText;
let callback, queue, called = false;
const trace = [], exports = {};
vm.runInNewContext(js, {
  exports,
  requestIdleCallback: cb => { callback = cb; return 1; },
  cancelIdleCallback() {},
  performance: {now() {
    if (!called) {
      called = true;
      queue.clear();
      queue.enqueue(() => { trace.push('new'); return false; });
    }
    return 0;
  }}
});
queue = new exports.IdleTaskQueue({warn() {}});
queue.enqueue(() => { trace.push('old'); return false; });
callback({timeRemaining: () => 100});
console.log(JSON.stringify({
  task: 367, sourceTrace: trace, sourceSha256: createHash('sha256').update(source).digest('hex'),
  node: process.version, typescript: ts.version,
  scope: 'Unchanged source in controlled idle/clock host, not a real browser runtime'
}, null, 2));

// Original xterm.js service modules execute unchanged in a private CommonJS loader.
import fs from 'node:fs';
import path from 'node:path';
import vm from 'node:vm';
import { createHash } from 'node:crypto';
import { createRequire } from 'node:module';
import { spawnSync } from 'node:child_process';
const require = createRequire(import.meta.url);
const ts = require('../source_repo/node_modules/typescript');
const root = path.resolve(import.meta.dirname, '..');
const sourceFiles = ['ServiceRegistry.ts', 'InstantiationService.ts'];
const hashes = {};
function original(name, imports = {}) {
  const file = path.join(root, 'source_repo/src/common/services', name);
  const source = fs.readFileSync(file, 'utf8');
  hashes[name] = createHash('sha256').update(source).digest('hex');
  const js = ts.transpileModule(source, { compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2022 } }).outputText;
  const module = { exports: {} };
  vm.runInThisContext(`(function(require,module,exports){${js}\n})`, { filename: file })(id => {
    if (!(id in imports)) throw new Error(`Unmapped original import ${id}`);
    return imports[id];
  }, module, module.exports);
  return module.exports;
}
const registry = original(sourceFiles[0]);
const selfId = registry.createDecorator('InstantiationService');
const { ServiceCollection, InstantiationService } = original(sourceFiles[1], {
  './Services': { IInstantiationService: selfId }, './ServiceRegistry': registry
});
const service = new InstantiationService();
if (service.getService(selfId) !== service) throw new Error('Original self identity failed');
const collection = new ServiceCollection();
const ids = new Map();
const ctors = new Map();
const trace = [];
const expected = [];
function record(...fields) {
  trace.push(fields.join('\t'));
  const [op, a, b, c, d] = fields;
  let output;
  try {
    switch (op) {
      case 'V': {
        const values = { undefined: undefined, null: null, false: false, true: true, zero: 0, 'negative-zero': -0, nan: NaN, infinity: Infinity, empty: '', text: 'text', object: [] };
        const injection = new InstantiationService();
        const id = registry.createDecorator('value');
        injection.setService(id, values[a]);
        const ctor = { Ctor: function (...args) { this.args = args; } }.Ctor;
        id(ctor, undefined, 0);
        injection.createInstance(ctor);
        output = 'ok';
        break;
      }
      case 'R': {
        const id = registry.createDecorator(a);
        output = String(ids.has(a) && ids.get(a) === id);
        ids.set(a, id);
        break;
      }
      case 'S': output = collection.set(ids.get(a), b); service.setService(ids.get(a), b); break;
      case 'G': output = collection.get(ids.get(a)); break;
      case 'H': output = String(collection.has(ids.get(a))); break;
      case 'E': { const entries = []; collection.forEach((id, value) => entries.push(`${id._id}=${value}`)); output = entries.join(','); break; }
      case 'L': {
        const entries = [];
        collection.forEach((id, value) => {
          entries.push(`${id._id}=${value}`);
          if (id === ids.get('a')) {
            collection.set(ids.get('b'), 'Bnext');
            collection.set(ids.get('d'), 'D');
            let count = 0;
            collection.forEach(() => count++);
            entries.push(`nested=${count}`);
          }
        });
        output = entries.join(','); break;
      }
      case 'N': {
        const ctor = { [a]: function (...args) { this.args = args; } }[a];
        if (b !== '-') Object.setPrototypeOf(ctor, ctors.get(b));
        ctors.set(a, ctor); output = 'ok'; break;
      }
      case 'D': {
        const args = [ctors.get(a), undefined, Number(c)];
        args.length = Number(d);
        ids.get(b)(...args); output = 'ok'; break;
      }
      case 'M': output = registry.getServiceDependencies(ctors.get(a)).map(v => `${v.id._id}@${v.index}`).join(','); break;
      case 'I': output = service.createInstance(ctors.get(a), ...Array.from({ length: Number(b) }, (_, i) => `static${i}`)).args.join(','); break;
      default: throw new Error(`Unknown operation ${op}`);
    }
  } catch (e) { output = e.message; }
  expected.push(String(output));
}
for (const name of ['undefined', 'null', 'false', 'true', 'zero', 'negative-zero', 'nan', 'infinity', 'empty', 'text', 'object']) record('V', name);
for (const id of ['', 'a', 'b', 'c', '🦀']) { record('R', id); record('R', id); record('H', id); record('G', id); }
record('N', 'Base', '-'); record('D', 'Base', 'a', 2, 3); record('D', 'Base', 'b', 0, 3);
record('N', 'Child', 'Base'); record('M', 'Child'); record('I', 'Child', 1); record('M', 'Base');
for (const arity of [0, 1, 2, 4]) record('D', 'Child', 'a', 1, arity);
record('S', 'a', 'A'); record('S', 'b', 'B'); record('I', 'Child', 0); record('M', 'Base');
record('D', 'Child', 'c', 0, 3); record('D', 'Child', 'a', 0, 3); record('D', 'Child', 'b', 9, 3);
record('I', 'Child', 1); record('S', 'c', 'C'); record('I', 'Child', 0); record('I', 'Child', 1); record('M', 'Child');
record('R', 'd'); record('L'); record('E');
record('N', 'Empty', '-'); for (let count = 0; count < 5; count++) record('I', 'Empty', count);
const seed = 0x5e7a1ce;
let state = seed;
const random = n => { state ^= state << 13; state ^= state >>> 17; state ^= state << 5; return (state >>> 0) % n; };
for (let i = 0; i < 2500; i++) {
  const id = ['a', 'b', 'c'][random(3)];
  const op = random(8);
  if (op < 3) record('S', id, random(5) === 0 ? '' : `v${random(100)}`);
  else if (op === 3) record('G', id);
  else if (op === 4) record('H', id);
  else if (op === 5) record('E');
  else if (op === 6) record('I', 'Child', random(3));
  else record('M', 'Child');
}
fs.mkdirSync(path.join(root, 'results/service-injection'), { recursive: true });
const build = spawnSync('cargo', ['build', '-p', 'xterm-core', '--example', 'service_injection_trace'], { cwd: root, encoding: 'utf8', timeout: 120000 });
fs.writeFileSync(path.join(root, 'results/service-injection/build.log'), build.stdout + build.stderr);
if (build.status !== 0) {
  fs.writeFileSync(path.join(root, 'results/service-injection/failed-build.json'), JSON.stringify({ command: 'cargo build -p xterm-core --example service_injection_trace', status: build.status, error: build.stderr || build.error?.message }, null, 2) + '\n');
  throw new Error(build.stderr || build.error?.message || 'Cargo build failed');
}
const executable = path.join(root, 'target/debug/examples/service_injection_trace' + (process.platform === 'win32' ? '.exe' : ''));
const run = spawnSync(executable, [], { cwd: root, input: trace.join('\n') + '\n', encoding: 'utf8', timeout: 30000 });
fs.writeFileSync(path.join(root, 'results/service-injection/actual.txt'), run.stdout);
fs.writeFileSync(path.join(root, 'results/service-injection/expected.txt'), expected.join('\n') + '\n');
if (run.status !== 0) throw new Error(run.stderr || run.error?.message || 'Rust trace failed');
const actual = run.stdout.split(/\r?\n/);
if (actual.at(-1) === '') actual.pop();
const failures = [];
for (let i = 0; i < Math.max(expected.length, actual.length); i++) if (expected[i] !== actual[i]) failures.push({ index: i, operation: trace[i], expected: expected[i], actual: actual[i] });
const rustc = spawnSync('rustc', ['--version'], { encoding: 'utf8', timeout: 10000 });
const cargo = spawnSync('cargo', ['--version'], { encoding: 'utf8', timeout: 10000 });
if (rustc.status !== 0 || cargo.status !== 0) throw new Error('Toolchain version query failed');
const implementationFiles = ['crates/xterm-core/src/service_registry.rs', 'crates/xterm-core/src/instantiation.rs', 'crates/xterm-core/examples/service_injection_trace.rs', 'crates/xterm-core/tests/service_injection.rs', 'scripts/service-injection-oracle.mjs'];
const implementation = implementationFiles.map(file => ({ path: file, sha256: createHash('sha256').update(fs.readFileSync(path.join(root, file))).digest('hex') }));
const results = { schemaVersion: 1, rustc: rustc.stdout.trim(), cargo: cargo.stdout.trim(), implementation, status: failures.length ? 'failed' : 'passed', seed, iterations: 2500, comparisons: expected.length, originalUpstreamTests: 0, sourceFiles: hashes, node: process.version, typescript: ts.version, platform: process.platform, arch: process.arch, command: 'node scripts/service-injection-oracle.mjs', failures, gaps: ['No original test files exist for these two modules.', 'Rc identity/drop, typed mismatch and constructor failure are separate Rust tests, not this line protocol.', 'Terminal direct construction and browser integration are not verified by these utility traces.', 'Native Linux/macOS and WASM not run by this command.'] };
fs.writeFileSync(path.join(root, 'results/service-injection/trace.tsv'), trace.join('\n') + '\n');
fs.writeFileSync(path.join(root, 'results/service-injection/oracle.json'), JSON.stringify(results, null, 2) + '\n');
console.log(JSON.stringify(results, null, 2));
if (failures.length) process.exitCode = 1;

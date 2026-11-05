// Producer snapshot and independent durable/runtime verification are separate.
import {readFile, writeFile, readdir, lstat} from 'node:fs/promises';
import {createHash} from 'node:crypto';
import path from 'node:path';
const root = path.resolve(import.meta.dirname, '..');
const source = path.resolve(process.env.SOURCE_ROOT ?? path.join(root, 'source_repo'));
const output = path.join(root, 'results/cursor-blink/audit.json');
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
const pin = async file => {
  const bytes = await readFile(file);
  // Markdown follows Git's native newline conversion. Code and raw evidence
  // remain byte-exact; only documentation is checked as canonical LF text.
  const normalization = file.endsWith('.md') ? 'LF' : undefined;
  return {path:path.relative(root, file).replaceAll('\\', '/'), normalization,
    sha256:hash(normalization ? bytes.toString().replaceAll('\r\n', '\n') : bytes)};
};
async function files(directory) {
  const result = [];
  for (const entry of await readdir(directory, {withFileTypes:true})) {
    const file = path.join(directory, entry.name);
    if ((await lstat(file)).isSymbolicLink()) throw Error(`Refusing linked evidence: ${file}`);
    if (entry.isDirectory()) result.push(...await files(file));
    else if (entry.isFile()) result.push(file);
    else throw Error(`Nonregular evidence: ${file}`);
  }
  return result;
}
const reference = JSON.parse(await readFile(path.join(root, 'docs/baseline/reference-manifest.json')));
for (const entry of reference) {
  if (hash(await readFile(path.join(source, entry.path))) !== entry.sha256) throw Error(`Reference changed: ${entry.path}`);
}
if (!(await readFile(path.join(root, 'crates/xterm-web/src/webgl/mod.rs'), 'utf8')).includes('pub mod cursor_blink;')) throw Error('Module binding absent');
if (process.argv.includes('--record')) {
  const owned = [
    'crates/xterm-web/src/webgl/cursor_blink.rs', 'crates/xterm-web/src/webgl/cursor_blink_timers.rs',
    'crates/xterm-web/tests/wasm_cursor_blink.rs', 'crates/xterm-web/tests/cursor_blink_cases.js',
    'scripts/cursor-blink-oracle.mjs', 'scripts/cursor-blink-negative.mjs', 'scripts/cursor-blink-audit.mjs',
    'docs/cursor-blink.md', '.pi/swarm/skills/cursor-blink-check/SKILL.md', 'Cargo.lock'
  ];
  const durable = await files(path.join(root, 'results/cursor-blink'));
  const browser = await files(path.join(root, 'results/browser-host/wasm_cursor_blink'));
  const runtime = browser.filter(file => file.includes(`${path.sep}artifacts${path.sep}`));
  for (const file of browser.filter(file => !runtime.includes(file))) durable.push(file);
  durable.push(path.join(root, 'results/browser-host/wasm_cursor_blink-results.json'));
  runtime.push(path.join(root, 'tmp/cursor-blink/prelude.js'));
  for (const file of browser.filter(file => file.endsWith('-results.json'))) {
    const report = JSON.parse(await readFile(file));
    if (report.artifacts) {
      // Cargo reuses its compiler path across mutant/restored builds. Verify
      // each run's retained delivered WASM/bindings, not an overwritten raw
      // compiler artifact. The final raw compiler file is pinned separately.
      for (const key of ['wasm', 'bindings']) {
        const artifact = report.artifacts[key];
        if (hash(await readFile(artifact.path)) !== artifact.sha256) throw Error(`Delivered artifact changed: ${artifact.path}`);
      }
      runtime.push(path.resolve(root, report.artifacts.compiler.path));
    }
  }
  for (const file of await readdir(path.join(root, 'target/debug/deps'))) {
    if (/^webgl-[a-f0-9]+\.exe$/.test(file)) runtime.push(path.join(root, 'target/debug/deps', file));
  }
  const unique = values => [...new Set(values)].sort();
  const receipt = {
    operation:'producer snapshot, not independent verification', referenceFiles:reference.length,
    source:await Promise.all(owned.map(file => pin(path.join(root, file)))),
    durable:await Promise.all(unique(durable.filter(file => file !== output)).map(pin)),
    runtime:await Promise.all(unique(runtime).map(pin))
  };
  await writeFile(output, JSON.stringify(receipt, null, 2) + '\n');
}
const receipt = JSON.parse(await readFile(output));
for (const entry of [...receipt.source, ...receipt.durable, ...(process.argv.includes('--runtime') ? receipt.runtime : [])]) {
  const bytes = await readFile(path.resolve(root, entry.path));
  const actual = hash(entry.normalization === 'LF' ? bytes.toString().replaceAll('\r\n', '\n') : bytes);
  if (actual !== entry.sha256) throw Error(`Pin changed: ${entry.path}`);
}
if (process.argv.includes('--runtime')) {
  for (const entry of receipt.durable.filter(entry => entry.path.endsWith('-results.json'))) {
    const report = JSON.parse(await readFile(path.resolve(root, entry.path)));
    if (!report.artifacts) continue;
    for (const key of ['wasm', 'bindings']) {
      const artifact = report.artifacts[key];
      if (hash(await readFile(artifact.path)) !== artifact.sha256) throw Error(`Delivered artifact changed: ${artifact.path}`);
    }
  }
  const latest = JSON.parse(await readFile(path.join(root, 'results/browser-host/wasm_cursor_blink-results.json')));
  const compiler = latest.artifacts.compiler;
  if (hash(await readFile(path.resolve(root, compiler.path))) !== compiler.sha256) throw Error('Final compiler artifact changed');
}
console.log(`PASS ${reference.length} original hashes; ${receipt.source.length} source; ${receipt.durable.length} durable; ${process.argv.includes('--runtime') ? receipt.runtime.length : 0} runtime pins`);

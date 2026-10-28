// Copyright (c) 2026 The xterm.js authors. All rights reserved. MIT license.
// Execute the original terminal, without changing source handlers or assertions.
import {createRequire} from 'node:module';
import {readFile, mkdir, writeFile} from 'node:fs/promises';
import {createHash} from 'node:crypto';
import path from 'node:path';
const source = path.resolve(process.argv[2] ?? 'source_repo');
const require = createRequire(path.join(source, 'package.json'));
const {build} = require('esbuild');
const bundle = await build({entryPoints:[path.join(source,'src/headless/public/Terminal.ts')],
  bundle:true,write:false,platform:'node',format:'esm',target:'node24',
  tsconfig:path.join(source,'src/headless/tsconfig.json'),metafile:true});
const {Terminal} = await import(`data:text/javascript;base64,${Buffer.from(bundle.outputFiles[0].contents).toString('base64')}`);
const pty = () => ({backend:'conpty', buildNumber:21375});
const rows = [];
function run(name, setup, input = 'abcde\n', initial = false) {
  const terminal = new Terminal({cols:5, rows:3, allowProposedApi:true, logLevel:'off',
    ...(initial ? {windowsPty:pty()} : {})});
  const trace = [];
  const wrapped = () => terminal._core.buffers.active.lines.get(1).isWrapped;
  const observe = label => trace.push(`${label}:${wrapped()}`);
  setup(terminal, observe);
  try { terminal.write(input); terminal._core._writeBuffer.flushSync(); }
  catch(error) { trace.push(`error:${error.message}`); }
  trace.push(`final:${wrapped()}`);
  rows.push([name, trace.join(',')]);
  terminal.dispose();
}
run('lf-before-enable', (t, log) => {
  t.onLineFeed(() => log('early')); t.options.windowsPty = pty();
  t.onLineFeed(() => log('late'));
});
run('lf-constructor-only', (t, log) => t.onLineFeed(() => log('user')), 'abcde\n', true);
run('lf-enabled-idempotent', (t, log) => {
  t.options.windowsPty = pty(); t.onLineFeed(() => log('user'));
  t.options.windowsPty = pty();
});
run('lf-reenabled', (t, log) => {
  t.options.windowsPty = pty(); t.onLineFeed(() => log('early'));
  t.options.windowsPty = {}; t.options.windowsPty = pty();
  t.onLineFeed(() => log('late'));
});
run('lf-disable-during-dispatch', (t, log) => {
  t.onLineFeed(() => { log('early'); t.options.windowsPty = {}; });
  t.options.windowsPty = pty(); t.onLineFeed(() => log('late'));
});
run('lf-enable-during-dispatch', (t, log) => {
  t.onLineFeed(() => { log('early'); t.options.windowsPty = pty(); });
  t.onLineFeed(() => log('late'));
});
run('lf-error-before-hook', (t, log) => {
  t.onLineFeed(() => {log('early'); throw new Error('listener failed');});
  t.options.windowsPty = pty();
});
for (const before of [true, false]) {
  run(`lf-dispose-${before?'before':'after'}-hook`, (t, log) => {
    const register = () => t.onLineFeed(() => {log('dispose'); t.dispose();});
    if (before) register(); t.options.windowsPty = pty(); if (!before) register();
    t.onLineFeed(() => log('late'));
  });
}
for (const before of [true, false]) for (const consume of [true, false]) {
  run(`csi-${before?'before':'after'}-${consume?'consume':'bubble'}`, (t, log) => {
    t.write('abcde\x1b[2;1H'); t._core._writeBuffer.flushSync();
    const register = () => t.parser.registerCsiHandler({final:'H'}, () => {log('user'); return consume;});
    if(before) register(); t.options.windowsPty = pty(); if(!before) register();
  }, '\x1b[3;1H');
}
run('csi-reenabled', (t, log) => {
  t.write('abcde\x1b[2;1H'); t._core._writeBuffer.flushSync();
  t.options.windowsPty = pty();
  t.parser.registerCsiHandler({final:'H'}, () => {log('user'); return true;});
  t.options.windowsPty = {}; t.options.windowsPty = pty();
}, '\x1b[3;1H');
run('reset-rechecks-mutated-object', (t, log) => {
  const value = pty(); t.options.windowsPty = value;
  value.buildNumber = 21376; t.reset(); t.onLineFeed(() => log('user'));
});
const sources = await Promise.all(Object.keys(bundle.metafile.inputs).sort().map(async file => ({
  file:path.relative(source,file).replaceAll(path.sep,'/'),
  sha256:createHash('sha256').update(await readFile(file)).digest('hex')
})));
await mkdir('results/windows-wrapping', {recursive:true});
await writeFile('results/windows-wrapping/source.json', JSON.stringify({
  command:'node scripts/windows-wrapping-oracle.mjs [source-root]', node:process.version,
  platform:process.platform, sources, rows
}, null, 2)+'\n');
await writeFile('results/windows-wrapping/source.tsv', rows.map(row => row.join('\t')).join('\n')+'\n');
console.log(rows.map(row => row.join('\t')).join('\n'));

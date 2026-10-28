// Licensed under MIT. A small source-path build while unrelated shared modules
// are being edited. Does not replace Cargo integration or workspace gates.
import assert from 'node:assert/strict';
import { mkdirSync, mkdtempSync, readFileSync, readdirSync, rmSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { spawnSync } from 'node:child_process';

export function focusedBuild(root, resultDir) {
  const temporaryRoot = path.join(root, 'tmp');
  mkdirSync(temporaryRoot, { recursive: true });
  const dir = mkdtempSync(path.join(temporaryRoot, 'keyboard-dispatch-focused-'));
  const commands = [];
  let compileLog = '';
  let testLog = '';
  function run(command, args, testing = false) {
    const result = spawnSync(command, args, { cwd: root, encoding: 'utf8', timeout: 120000 });
    const log = result.stdout + result.stderr;
    commands.push({ command, args, exit: result.status });
    if (testing) testLog += log; else compileLog += log;
    writeFileSync(path.join(resultDir, 'focused-build.log'), compileLog);
    writeFileSync(path.join(resultDir, 'focused-tests.log'), testLog);
    writeFileSync(path.join(resultDir, 'focused-commands.json'), JSON.stringify(commands, null, 2) + '\n');
    if (result.error) throw result.error;
    assert.equal(result.status, 0, log);
  }
  const rustc = process.env.RUSTC || 'rustc';
  const target = process.env.CARGO_TARGET_DIR || path.join(root, 'target');
  const dependencies = path.join(target, 'debug/deps');
  const quotePath = name => JSON.stringify(path.join(root, name).replaceAll('\\', '/'));
  const module = (name, file) => `#[path=${quotePath(file)}] pub mod ${name};`;
  try {
    run(process.env.CARGO || 'cargo', ['build', '-p', 'ryu-js']);
    const candidates = readdirSync(dependencies).filter(name => /^libryu_js-.*\.rlib$/.test(name));
    assert.ok(process.env.RYU_JS_RLIB || candidates.length === 1, 'set RYU_JS_RLIB if multiple cached ryu-js libraries exist');
    const ryu = process.env.RYU_JS_RLIB || path.join(dependencies, candidates[0]);
    const corePath = path.join(dir, 'core.rs');
    // A #[path] inclusion changes Rust's inferred submodule directory. Adjust
    // only those two paths in a temporary copy, never production logic.
    const kittyPath = path.join(dir, 'kitty.rs');
    let kittySource = readFileSync(path.join(root, 'crates/xterm-core/src/input/kitty_keyboard.rs'), 'utf8');
    for (const name of ['keys', 'inherited']) kittySource = kittySource.replace(`mod ${name};`,
      `#[path=${quotePath(`crates/xterm-core/src/input/kitty_keyboard/${name}.rs`)}] mod ${name};`);
    writeFileSync(kittyPath, kittySource);
    writeFileSync(corePath, 'pub mod input {\n' + [
      module('keyboard', 'crates/xterm-core/src/input/keyboard.rs'),
      `#[path=${JSON.stringify(kittyPath.replaceAll('\\', '/'))}] pub mod kitty_keyboard;`,
      module('win32_input_mode', 'crates/xterm-core/src/input/win32_input_mode.rs')
    ].join('\n') + '\n}\n');
    const core = path.join(dir, 'libxterm_core.rlib');
    const flags = ['--edition=2021', '-Dwarnings', '-Cdebuginfo=0', '-L', `dependency=${dependencies}`, '-L', `dependency=${dir}`];
    run(rustc, [...flags, '--crate-name', 'xterm_core', '--crate-type', 'rlib', corePath, '--extern', `ryu_js=${ryu}`, '-o', core]);
    const webPath = path.join(dir, 'web.rs');
    writeFileSync(webPath, 'pub mod input {\n' + [
      module('keyboard', 'crates/xterm-web/src/input/keyboard.rs'),
      module('keyboard_dispatch_types', 'crates/xterm-web/src/input/keyboard_dispatch_types.rs'),
      module('keyboard_dispatch', 'crates/xterm-web/src/input/keyboard_dispatch.rs')
    ].join('\n') + '\n}\n');
    const web = path.join(dir, 'libxterm_web.rlib');
    run(rustc, [...flags, '--crate-name', 'xterm_web', '--crate-type', 'rlib', webPath, '--extern', `xterm_core=${core}`, '-o', web]);
    const externs = ['--extern', `xterm_core=${core}`, '--extern', `xterm_web=${web}`];
    const clippy = process.env.CLIPPY_DRIVER || 'clippy-driver';
    run(clippy, [...flags, '-Wclippy::all', '--crate-name', 'xterm_web', '--crate-type', 'rlib', webPath,
      '--extern', `xterm_core=${core}`, '--emit=metadata', '-o', path.join(dir, 'web-check.rmeta')]);
    for (const name of ['keyboard_dispatch', 'keyboard_dispatch_callbacks', 'keyboard_dispatch_getters']) {
      const binary = path.join(dir, process.platform === 'win32' ? `${name}.exe` : name);
      run(rustc, [...flags, ...externs, '--test', path.join(root, `crates/xterm-web/tests/${name}.rs`), '-o', binary]);
      run(binary, [], true);
      run(clippy, [...flags, '-Wclippy::all', ...externs, '--test', path.join(root, `crates/xterm-web/tests/${name}.rs`),
        '--emit=metadata', '-o', path.join(dir, `${name}-check.rmeta`)]);
    }
    const binary = path.join(dir, process.platform === 'win32' ? 'oracle.exe' : 'oracle');
    run(rustc, [...flags, ...externs, path.join(root, 'crates/xterm-web/examples/keyboard_dispatch_oracle.rs'), '-o', binary]);
    return { binary, cleanup: () => rmSync(dir, { recursive: true, force: true }),
      scope: 'Focused rustc source-path build; does not compile unrelated core/web modules or establish full Cargo integration.' };
  } catch (error) {
    rmSync(dir, { recursive: true, force: true });
    throw error;
  }
}

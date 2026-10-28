import { resolve, relative, sep, join } from 'node:path';
import { mkdir, lstat } from 'node:fs/promises';
import { DEFAULT_MIN_FREE_MIB } from './quality-resources.mjs';
import { stripVTControlCharacters } from 'node:util';

export function parseOptions(args, platform, architecture) {
  const options = { destination: `results/quality/${platform}-${architecture}-${Date.now()}`, timeoutMs: 300_000, wasm: false, minFreeMiB: DEFAULT_MIN_FREE_MIB };
  for (let i = 0; i < args.length; i++) {
    if (args[i] === '--out' && args[i + 1]) options.destination = args[++i];
    else if (args[i] === '--timeout-ms' && args[i + 1]) options.timeoutMs = Number(args[++i]);
    else if (args[i] === '--wasm-check') options.wasm = true;
    else if (args[i] === '--min-free-mib' && args[i + 1]) options.minFreeMiB = Number(args[++i]);
    else throw new Error(`Unknown or incomplete option: ${args[i]}`);
  }
  if (!Number.isSafeInteger(options.timeoutMs) || options.timeoutMs < 100 || options.timeoutMs > 1_800_000) {
    throw new Error('--timeout-ms must be an integer between 100 and 1800000');
  }
  if (!Number.isSafeInteger(options.minFreeMiB) || options.minFreeMiB < 256 || options.minFreeMiB > 1_048_576) {
    throw new Error('--min-free-mib must be an integer between 256 and 1048576');
  }
  return options;
}
export function outputPath(root, destination) {
  const path = resolve(root, destination);
  const name = relative(root, path);
  if (!name || name.startsWith('..') || name.split(sep)[0] !== 'results' || name.split(sep).length < 2) {
    throw new Error('--out must name a subdirectory of repository-root results/');
  }
  return path;
}
export async function reserveOutput(root, destination) {
  const out = outputPath(root, destination);
  const parts = relative(root, out).split(sep);
  let current = root;
  for (const part of parts.slice(0, -1)) {
    current = join(current, part);
    try { await mkdir(current); }
    catch (error) { if (error.code !== 'EEXIST') throw error; }
    const info = await lstat(current);
    if (!info.isDirectory() || info.isSymbolicLink()) throw new Error('Output parent must be a real directory inside repository results/');
  }
  await mkdir(out);
  return out;
}
export function assertNativeHost(platform, host, declaredTarget) {
  const pattern = { win32: /windows/, linux: /linux/, darwin: /apple-darwin/ }[platform];
  if (!host || !pattern?.test(host)) throw new Error('rustc host does not match the executing OS; native evidence rejected');
  if (declaredTarget && declaredTarget !== host) {
    throw new Error('CARGO_BUILD_TARGET differs from rustc host; unset it for native runtime evidence');
  }
}
export function assertBuildEnvironment(env) {
  const overrides = Object.keys(env).filter(key =>
    key === 'CARGO_TARGET_DIR' || (key === 'CARGO_INCREMENTAL' && env[key] !== '0') || /^CARGO_PROFILE_(DEV|TEST|RELEASE)_/.test(key));
  if (overrides.length) throw new Error(`Build policy overrides must be unset: ${overrides.sort().join(', ')}`);
}
export function assertResolvedTarget(platform, cfg) {
  const targetOs = cfg.match(/^target_os="([^"]+)"$/m)?.[1];
  const expected = { win32: 'windows', linux: 'linux', darwin: 'macos' }[platform];
  if (!expected || targetOs !== expected) throw new Error(`Resolved Cargo target OS ${targetOs ?? 'unknown'} does not match runtime ${platform}`);
  return targetOs;
}
export function assertSharedTarget(root, actual, platform) {
  const canonical = value => platform === 'win32' ? resolve(value).toLowerCase() : resolve(value);
  if (canonical(actual) !== canonical(resolve(root, 'target'))) throw new Error('Cargo metadata target_directory differs from repository shared target/');
}
export function testCounts(log) {
  const counts = [];
  let suite = null;
  for (const line of stripVTControlCharacters(log).split('\n')) {
    const header = line.match(/^\s*(?:Running\s+(.+)|Doc-tests\s+(.+))$/);
    if (header) suite = (header[1] ?? `Doc-tests ${header[2]}`).trim();
    const match = line.match(/test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out/);
    if (match) counts.push({
      ...(suite ? { suite } : {}),
      passed: Number(match[1]), failed: Number(match[2]), ignored: Number(match[3]),
      measured: Number(match[4]), filteredOut: Number(match[5])
    });
  }
  return counts;
}

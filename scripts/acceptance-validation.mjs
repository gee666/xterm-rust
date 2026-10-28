import { strict as assert } from 'node:assert';
import { readFile, lstat, realpath } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import path from 'node:path';
import { isProxy } from 'node:util/types';

function serializationHook(value) {
  for (let current = value; current; current = Object.getPrototypeOf(current)) {
    if (isProxy(current)) throw new Error('proxy in capture prototype chain');
    const descriptor = Object.getOwnPropertyDescriptor(current, 'toJSON');
    if (descriptor) return !Object.hasOwn(descriptor, 'value') || typeof descriptor.value === 'function';
  }
  return false;
}

export function assertJsonSafe(value, location = 'result', seen = new WeakSet()) {
  if (value === null || typeof value === 'string' || typeof value === 'boolean') return;
  if (typeof value === 'number' && Number.isFinite(value) && !Object.is(value, -0)) return;
  if (typeof value === 'object') {
    if (isProxy(value) || serializationHook(value)) throw new Error(`${location} has dynamic serialization behavior`);
    if (seen.has(value)) throw new Error(`${location} reuses an object; encode identities and capture executions separately`);
    seen.add(value);
  }
  if (Array.isArray(value)) {
    const keys = Reflect.ownKeys(value);
    if (keys.length !== value.length + 1 || keys.some(key => key !== 'length' && (typeof key !== 'string' || !/^(0|[1-9]\d*)$/.test(key) || Number(key) >= value.length))) {
      throw new Error(`${location} has sparse or extra array properties`);
    }
    for (let i = 0; i < value.length; i++) {
      const descriptor = Object.getOwnPropertyDescriptor(value, String(i));
      if (!descriptor?.enumerable || !Object.hasOwn(descriptor, 'value')) throw new Error(`${location} has a dynamic array entry`);
      assertJsonSafe(descriptor.value, `${location}[${i}]`, seen);
    }
    return;
  }
  if (typeof value === 'object') {
    const prototype = Object.getPrototypeOf(value);
    const plain = prototype === null || prototype === Object.prototype
      || (Object.getPrototypeOf(prototype) === null && Object.getOwnPropertyDescriptor(prototype, 'constructor')?.value?.name === 'Object');
    if (plain) {
      for (const key of Reflect.ownKeys(value)) {
        const descriptor = Object.getOwnPropertyDescriptor(value, key);
        if (typeof key !== 'string' || !descriptor.enumerable || !Object.hasOwn(descriptor, 'value')) throw new Error(`${location} has hidden, symbolic or accessor properties`);
        assertJsonSafe(descriptor.value, `${location}.${key}`, seen);
      }
      return;
    }
  }
  throw new Error(`${location} is not losslessly JSON-serializable; encode special values explicitly`);
}

export function portablePath(relative) {
  if (typeof relative !== 'string' || !relative || relative.includes('\\') || relative.includes('\0')
    || relative.startsWith('/') || relative.includes(':')
    || relative.split('/').some(part => !part || part === '.' || part === '..')) {
    throw new Error(`unsafe repository path: ${relative}`);
  }
  return relative;
}

export async function regularFile(root, relative) {
  portablePath(relative);
  let current = root;
  const parts = relative.split('/');
  for (const [index, part] of parts.entries()) {
    current = path.join(current, part);
    const info = await lstat(current);
    if (info.isSymbolicLink() || (index < parts.length - 1 ? !info.isDirectory() : !info.isFile())) {
      throw new Error(`nonregular or symbolic reference: ${relative}`);
    }
  }
  return current;
}

export async function referenceFile(root, relative) {
  portablePath(relative);
  if (!relative.startsWith('source_repo/')) throw new Error('oracle source must be under source_repo');
  // The worktree script deliberately shares this one read-only reference root.
  // Resolve that root only; arbitrary and nested links remain rejected.
  const reference = await realpath(path.join(root, 'source_repo'));
  return regularFile(reference, relative.slice('source_repo/'.length));
}

async function verifyHashes(root, entries, label) {
  if (!Array.isArray(entries) || entries.length === 0) throw new Error(`${label} hashes are required`);
  const seen = new Set();
  for (const entry of entries) {
    if (seen.has(entry.path) || !/^[a-f0-9]{64}$/.test(entry.sha256)) throw new Error(`invalid or duplicate ${label} hash`);
    seen.add(entry.path);
    const file = label === 'source' ? await referenceFile(root, entry.path) : await regularFile(root, entry.path);
    const bytes = await readFile(file);
    assert.equal(createHash('sha256').update(bytes).digest('hex'), entry.sha256, `${label} drift: ${entry.path}`);
    if (label === 'source' && !entry.path.startsWith('source_repo/')) throw new Error('oracle source must be under source_repo');
    if (label === 'artifact' && entry.path.startsWith('source_repo/')) throw new Error('original JS is not a Rust artifact');
    if (label === 'artifact' && entry.path.endsWith('.wasm')) {
      assert.deepEqual([...bytes.subarray(0, 4)], [0, 97, 115, 109], 'invalid WASM artifact header');
    }
  }
}

export function validateCaptures(contract, result) {
  assertJsonSafe(result);
  assert.equal(result.caseId, contract.id, 'adapter returned a different case');
  if (!Array.isArray(result.captures) || result.captures.length === 0) throw new Error('zero captures');
  const engines = new Set();
  let assertions = 0;
  let skipped = 0;
  for (const capture of result.captures) {
    if (!contract.engines.includes(capture.engine) || engines.has(capture.engine)) throw new Error('wrong or duplicate engine');
    engines.add(capture.engine);
    if (capture.skipped) {
      if (!contract.id.startsWith('gpu-') || !capture.skipped.reason || !capture.skipped.condition
        || !capture.skipped.source?.startsWith('source_repo/')) throw new Error('invalid upstream skip');
      portablePath(capture.skipped.source);
      skipped++;
      continue;
    }
    const implementation = capture.engine === 'native' ? 'rust-native' : 'rust-wasm';
    assert.equal(capture.implementation, implementation, 'wrong implementation runtime');
    if (typeof capture.runtimeVersion !== 'string' || !capture.runtimeVersion.trim()) throw new Error('runtime version is required');
    if (!Number.isSafeInteger(capture.assertions) || capture.assertions < 1) throw new Error('zero or invalid assertions');
    for (const side of ['reference', 'rust']) {
      if (!capture[side] || Array.isArray(capture[side]) || typeof capture[side] !== 'object') throw new Error(`missing ${side} capture`);
      for (const field of contract.fields) {
        if (!Object.hasOwn(capture[side], field)) throw new Error(`missing ${side}.${field}`);
      }
    }
    assert.deepEqual(capture.rust, capture.reference, `${contract.id} ${capture.engine} state mismatch`);
    assertions += capture.assertions;
    if (!Number.isSafeInteger(assertions)) throw new Error('assertion total overflow');
  }
  for (const engine of contract.engines) if (!engines.has(engine)) throw new Error(`missing engine ${engine}`);
  if (assertions === 0) throw new Error('all runtimes skipped or zero assertions');
  return { assertions, executedRuntimes: engines.size - skipped, skippedRuntimes: skipped };
}

export async function validateResult(root, contract, result) {
  const totals = validateCaptures(contract, result);
  await verifyHashes(root, result.sourceHashes, 'source');
  await verifyHashes(root, result.artifactHashes, 'artifact');
  const cited = contract.source.match(/source_repo\/[A-Za-z0-9_./-]+\.ts/);
  if (cited && !result.sourceHashes.some(entry => entry.path === cited[0])) throw new Error('cited primary oracle source is not hashed');
  if (contract.engines.some(engine => engine !== 'native') && !result.artifactHashes.some(entry => entry.path.endsWith('.wasm'))) {
    throw new Error('browser capture lacks a WASM artifact');
  }
  for (const capture of result.captures.filter(entry => entry.skipped)) {
    const source = await referenceFile(root, capture.skipped.source);
    if (!result.sourceHashes.some(entry => entry.path === capture.skipped.source)) throw new Error('upstream skip source is not hashed');
    if (!(await readFile(source, 'utf8')).includes(capture.skipped.condition)) throw new Error('upstream skip condition is not present in cited source');
  }
  if (!Array.isArray(result.commands) || result.commands.length === 0) throw new Error('command receipts are required');
  for (const command of result.commands) {
    if (!Array.isArray(command.argv) || command.argv.length === 0 || command.argv.some(arg => typeof arg !== 'string')) throw new Error('invalid command argv');
    assert.equal(command.exitCode, 0, 'adapter command failed');
    await regularFile(root, command.log);
  }
  return totals;
}

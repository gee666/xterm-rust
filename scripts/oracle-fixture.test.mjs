import { strict as assert } from 'node:assert';
import { test } from 'node:test';
import { mkdtemp, mkdir, readFile, writeFile, rm } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import { verifyOrGenerate } from './oracle-fixture.mjs';

const tmp = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../tmp');
await mkdir(tmp, { recursive: true });
async function fixture(t) {
  const directory = await mkdtemp(path.join(tmp, 'oracle-fixture-'));
  t.after(() => rm(directory, { recursive: true }));
  return path.join(directory, 'fixture.tsv');
}

test('verification checks contents without replacing CRLF bytes', async t => {
  const file = await fixture(t);
  await writeFile(file, 'a\r\nb\r\n');
  await verifyOrGenerate(file, 'a\nb\n', false);
  assert.equal(await readFile(file, 'utf8'), 'a\r\nb\r\n');
});

test('a changed expected result fails and preserves the original fixture', async t => {
  const file = await fixture(t);
  await writeFile(file, 'old\n');
  await assert.rejects(verifyOrGenerate(file, 'new\n', false), /fixture drift/);
  assert.equal(await readFile(file, 'utf8'), 'old\n');
});

test('missing fixtures fail ordinary verification instead of regenerating', async t => {
  const file = await fixture(t);
  await assert.rejects(verifyOrGenerate(file, 'expected\n', false), { code: 'ENOENT' });
  await assert.rejects(readFile(file), { code: 'ENOENT' });
});

test('explicit generation creates and replaces exact Unicode and NUL bytes', async t => {
  const file = await fixture(t);
  const expected = '🤣\0\n';
  await verifyOrGenerate(file, expected, true);
  assert.deepEqual(await readFile(file), Buffer.from(expected));
  await verifyOrGenerate(file, '', true);
  assert.equal((await readFile(file)).length, 0);
});

test('ordinary verification propagates nonfile I/O failures', async t => {
  const file = await fixture(t);
  await mkdir(file);
  await assert.rejects(verifyOrGenerate(file, '', false));
});

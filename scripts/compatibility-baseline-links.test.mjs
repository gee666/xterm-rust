// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
// Paired positive/negative controls never change source_repo or shared receipts.
import assert from 'node:assert/strict';
import test from 'node:test';
import { randomUUID } from 'node:crypto';
import { mkdir, readFile, writeFile, symlink, rm } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { loadBaseline } from '../tools/compatibility/baseline.mjs';
const root = fileURLToPath(new URL('../', import.meta.url));
const readJson = async relative => JSON.parse(await readFile(path.join(root, relative), 'utf8'));

test('all baseline reads reject junction aliases after regular positive controls', async () => {
  const dir = `tmp/compatibility-baseline-links-${randomUUID()}`;
  const data = `${dir}/data`;
  const linked = `${dir}/linked`;
  await mkdir(path.join(root, data), { recursive: true });
  const latest = await readJson('results/upstream-baseline/latest.json');
  const original = await readJson(`results/upstream-baseline/${latest.summary}`);
  const manifest = await readJson('results/compatibility/manifest.json');
  const writeJson = (relative, value) => writeFile(path.join(root, relative), JSON.stringify(value));
  const regularPointer = `${data}/pointer.json`;
  const rootPointer = `${dir}/root-pointer.json`;
  try {
    await writeJson(`${data}/summary.json`, original);
    await writeJson(regularPointer, { ...latest, summary: 'summary.json' });
    await loadBaseline(root, regularPointer, manifest);
    await symlink(path.join(root, data), path.join(root, linked), process.platform === 'win32' ? 'junction' : 'dir');
    await assert.rejects(loadBaseline(root, `${linked}/pointer.json`, manifest), /Baseline file must not be linked/);
    await writeJson(rootPointer, { ...latest, summary: 'data/summary.json' });
    await loadBaseline(root, rootPointer, manifest);
    await writeJson(rootPointer, { ...latest, summary: 'linked/summary.json' });
    await assert.rejects(loadBaseline(root, rootPointer, manifest), /Baseline file must not be linked/);
    await writeJson(rootPointer, { ...latest, summary: 'data/summary.json' });
    const variants = [
      { source: original.commands[0].log, assign: (s, p) => { s.commands[0].log = p; } },
      { source: original.reference.manifest, assign: (s, p) => { s.reference.manifest = p; } },
      { source: original.rawReport, assign: (s, p) => { s.rawReport = p; } },
      { source: original.registrationReport, assign: (s, p) => { s.registrationReport = p; } }
    ];
    for (let i = 0; i < variants.length; i++) {
      const { source, assign } = variants[i];
      const name = `payload-${i}.json`;
      await writeFile(path.join(root, data, name), await readFile(path.join(root, source)));
      const summary = structuredClone(original);
      assign(summary, `${data}/${name}`);
      await writeJson(`${data}/summary.json`, summary);
      await loadBaseline(root, rootPointer, manifest);
      assign(summary, `${linked}/${name}`);
      await writeJson(`${data}/summary.json`, summary);
      await assert.rejects(loadBaseline(root, rootPointer, manifest), /Baseline file must not be linked/);
    }
  } finally { await rm(path.join(root, dir), { recursive: true }); }
});

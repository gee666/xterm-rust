import { readFile, writeFile } from 'node:fs/promises';
import { strict as assert } from 'node:assert';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

export const FAMILIES = Object.freeze({
  img: { adapter: 'image.mjs', engines: ['chromium', 'firefox', 'webkit'], fields: ['pixels', 'buffer', 'cursor', 'replies', 'events'] },
  gpu: { adapter: 'webgl.mjs', engines: ['chromium', 'firefox', 'webkit'], fields: ['pixels', 'state', 'events'] },
  vt: { adapter: 'terminal.mjs', engines: ['native'], fields: ['cells', 'cursor', 'margins', 'normal', 'alternate', 'events', 'replies'] },
  move: { adapter: 'move-to-cell.mjs', engines: ['native'], fields: ['returns', 'errors', 'postState', 'events'] },
  async: { adapter: 'async-timers.mjs', engines: ['native'], fields: ['returns', 'errors', 'postState', 'events'] },
});

export function familyContract(id, verification = '') {
  const family = FAMILIES[id?.split('-')[0]];
  if (!family) throw new Error(`unknown acceptance family for ${id}`);
  if (verification.includes('[full text:')) throw new Error('unresolved acceptance verification text');
  const explicitBrowserMatrix = ['chromium', 'firefox', 'webkit']
    .every(engine => new RegExp(`\\b${engine}\\b`, 'i').test(verification));
  const browser = ['img', 'gpu'].includes(id.split('-')[0]) || explicitBrowserMatrix;
  return {
    adapter: family.adapter,
    engines: browser ? ['chromium', 'firefox', 'webkit'] : [...family.engines],
    fields: id.startsWith('vt-') && browser ? [...family.fields, 'render', 'api', 'callbacks'] : [...family.fields],
  };
}

export function validateCaseContracts(cases) {
  if (!Array.isArray(cases) || cases.length === 0) throw new Error('no acceptance contracts');
  const seen = new Set();
  for (const entry of cases) {
    const family = FAMILIES[entry.id?.split('-')[0]];
    if (!family || !/^[a-z]+-\d+$/.test(entry.id) || seen.has(entry.id)) throw new Error('invalid or duplicate case');
    seen.add(entry.id);
    const expected = familyContract(entry.id, entry.verification);
    assert.equal(entry.adapter, expected.adapter, 'wrong family adapter');
    assert.deepEqual(entry.engines, expected.engines, 'runtime matrix was weakened');
    assert.deepEqual(entry.fields, expected.fields, 'capture domain was weakened');
    if (!Number.isSafeInteger(entry.criterionId) || entry.criterionId < 1 || !entry.source || !entry.title) throw new Error('case lacks source criterion identity');
  }
  return cases;
}

export function verifyCaseSnapshot(cases, criteria) {
  validateCaseContracts(cases);
  assert.deepEqual(cases, buildCases(criteria), 'case catalog differs from authoritative criterion snapshot');
}

export function buildCases(criteria) {
  if (!Array.isArray(criteria)) throw new Error('criterion list is required');
  const found = new Set();
  const cases = [];
  for (const criterion of criteria) {
    const match = criterion.verification.match(/node scripts\/acceptance-oracle\.mjs --case ([a-z]+-\d+)/);
    if (!match) continue;
    const id = match[1];
    const family = id.split('-')[0];
    if (!FAMILIES[family]) throw new Error(`unknown acceptance family ${family}`);
    if (found.has(id)) throw new Error(`duplicate acceptance case ${id}`);
    if (!Number.isSafeInteger(criterion.id) || criterion.id < 1) throw new Error(`invalid criterion for ${id}`);
    found.add(id);
    cases.push({ id, criterionId: criterion.id, criterionRevision: criterion.revision,
      title: criterion.title, source: criterion.source, body: criterion.body, verification: criterion.verification,
      ...familyContract(id, criterion.verification) });
  }
  if (cases.length === 0) throw new Error('no acceptance cases discovered');
  return validateCaseContracts(cases.sort((a, b) => a.id.localeCompare(b.id, 'en')));
}

async function main() {
  const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
  const snapshot = JSON.parse(await readFile(path.join(root, 'results/acceptance/criteria-index.json'), 'utf8'));
  const cases = buildCases(snapshot.criteria);
  const output = JSON.stringify({ scope: 'Declared User acceptance contracts, not test results', cases }, null, 2) + '\n';
  const destination = path.join(root, 'docs/acceptance/cases.json');
  if (process.argv.includes('--generate')) await writeFile(destination, output);
  else assert.equal((await readFile(destination, 'utf8')).replace(/\r\n/g, '\n'), output, 'acceptance case registry drift');
  console.log(JSON.stringify({ declared: cases.length, generated: process.argv.includes('--generate') }));
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) await main();

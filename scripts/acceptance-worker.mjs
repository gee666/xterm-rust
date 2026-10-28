import { readFile, writeFile } from 'node:fs/promises';
import { pathToFileURL } from 'node:url';
import path from 'node:path';
import { assertJsonSafe, regularFile } from './acceptance-validation.mjs';

const config = JSON.parse(await readFile(process.argv[2], 'utf8'));
const adapter = await regularFile(config.root, `scripts/acceptance-adapters/${config.contract.adapter}`);
const module = await import(pathToFileURL(adapter).href);
if (typeof module.runCase !== 'function') throw new Error('adapter must export runCase');
const controller = new AbortController();
process.once('SIGTERM', () => controller.abort(new Error('case execution terminated')));
const result = await module.runCase({
  caseId: config.contract.id,
  root: config.root,
  outputDirectory: config.outputDirectory,
  signal: controller.signal,
  contract: config.contract,
  executionMode: 'worktree',
});
assertJsonSafe(result);
await writeFile(path.join(config.outputDirectory, 'worker-result.json'), JSON.stringify(result, null, 2) + '\n', { flag: 'wx' });

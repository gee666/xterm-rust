import { readFile, writeFile, mkdir } from 'node:fs/promises';
import { randomUUID, createHash } from 'node:crypto';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import { runCommand } from './quality-process.mjs';
import { diskHeadroom, requireHeadroom, DEFAULT_MIN_FREE_MIB } from './quality-resources.mjs';
import { regularFile, validateResult } from './acceptance-validation.mjs';
import { verifyCaseSnapshot } from './acceptance-case-index.mjs';

export const DEFAULT_TIMEOUT_MS = 120_000;
export const MAX_TIMEOUT_MS = 900_000;
export function parseArguments(args) {
  const options = { timeoutMs: DEFAULT_TIMEOUT_MS, list: false };
  const seen = new Set();
  for (let i = 0; i < args.length; i++) {
    const flag = args[i];
    if (seen.has(flag)) throw new Error(`duplicate argument ${flag}`);
    seen.add(flag);
    if (flag === '--list') options.list = true;
    else if (flag === '--case') options.caseId = args[++i];
    else if (flag === '--timeout-ms') options.timeoutMs = Number(args[++i]);
    else throw new Error(`unknown argument ${flag}`);
  }
  if (!Number.isSafeInteger(options.timeoutMs) || options.timeoutMs < 1 || options.timeoutMs > MAX_TIMEOUT_MS) throw new Error('invalid case deadline');
  if (options.list && (options.caseId || seen.has('--timeout-ms'))) throw new Error('--list cannot execute cases');
  if (!options.list && !/^[a-z]+-\d+$/.test(options.caseId ?? '')) throw new Error('--case <exact-id> is required');
  return options;
}

export async function runAttempt(root, contract, { timeoutMs = DEFAULT_TIMEOUT_MS, signal, execute = runCommand, observeDisk = diskHeadroom } = {}) {
  if (!Number.isSafeInteger(timeoutMs) || timeoutMs < 1 || timeoutMs > MAX_TIMEOUT_MS) throw new Error('invalid case deadline');
  if (!/^[a-z]+-\d+$/.test(contract.id)) throw new Error('invalid case id');
  if (!/^[a-z-]+\.mjs$/.test(contract.adapter)) throw new Error('invalid family adapter');
  const outputDirectory = path.join(root, 'results/acceptance-runs', contract.id, `${Date.now()}-${randomUUID()}`);
  await mkdir(outputDirectory, { recursive: true });
  const receipt = {
    caseId: contract.id, criterionId: contract.criterionId, status: 'failed',
    contractSha256: createHash('sha256').update(JSON.stringify(contract)).digest('hex'),
    started: new Date().toISOString(), environment: { node: process.version, platform: process.platform, architecture: process.arch },
    timeoutMs, outputDirectory: path.relative(root, outputDirectory).replaceAll('\\', '/'), disk: [],
  };
  const checkPolicy = async () => {
    signal?.throwIfAborted();
    const observation = await observeDisk(root, DEFAULT_MIN_FREE_MIB);
    receipt.disk.push(observation);
    requireHeadroom(observation);
  };
  let phase = 'locate-adapter';
  try {
    await regularFile(root, `scripts/acceptance-adapters/${contract.adapter}`);
    phase = 'resource-preflight';
    await checkPolicy();
    const configuration = path.join(outputDirectory, 'configuration.json');
    await writeFile(configuration, JSON.stringify({ root, contract, outputDirectory }) + '\n', { flag: 'wx' });
    phase = 'execute';
    receipt.process = await execute(process.execPath, [path.join(root, 'scripts/acceptance-worker.mjs'), configuration], {
      cwd: root, env: process.env, log: path.join(outputDirectory, 'process.log'), timeoutMs,
      signal, resourceCheck: checkPolicy,
    });
    if (receipt.process.status !== 'passed') throw new Error(`adapter process failed: ${JSON.stringify(receipt.process)}`);
    phase = 'validate-result';
    const result = JSON.parse(await readFile(path.join(outputDirectory, 'worker-result.json'), 'utf8'));
    receipt.totals = await validateResult(root, contract, result);
    await checkPolicy();
    receipt.status = receipt.totals.skippedRuntimes ? 'passed_with_source_skips' : 'passed';
  } catch (error) {
    receipt.status = phase === 'locate-adapter' && error.code === 'ENOENT' ? 'unrun' : 'failed';
    receipt.failurePhase = phase;
    receipt.error = String(error.stack ?? error);
    receipt.cancelled = signal?.aborted ?? false;
  }
  receipt.finished = new Date().toISOString();
  await writeFile(path.join(outputDirectory, 'receipt.json'), JSON.stringify(receipt, null, 2) + '\n', { flag: 'wx' });
  return receipt;
}

async function main() {
  const options = parseArguments(process.argv.slice(2));
  const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
  const registry = JSON.parse(await readFile(path.join(root, 'docs/acceptance/cases.json'), 'utf8'));
  const snapshot = JSON.parse(await readFile(path.join(root, 'results/acceptance/criteria-index.json'), 'utf8'));
  verifyCaseSnapshot(registry.cases, snapshot.criteria);
  if (options.list) {
    console.log(JSON.stringify({ status: 'declared, not executed', cases: registry.cases.map(({ id, criterionId, title }) => ({ id, criterionId, title })) }, null, 2));
    return;
  }
  const matches = registry.cases.filter(entry => entry.id === options.caseId);
  if (matches.length !== 1) throw new Error(`unknown or duplicate case ${options.caseId}`);
  const controller = new AbortController();
  const interrupt = () => controller.abort(new Error('Acceptance command interrupted'));
  process.once('SIGINT', interrupt);
  process.once('SIGTERM', interrupt);
  try {
    const receipt = await runAttempt(root, matches[0], { timeoutMs: options.timeoutMs, signal: controller.signal });
    console.log(JSON.stringify({ caseId: receipt.caseId, status: receipt.status, totals: receipt.totals, outputDirectory: receipt.outputDirectory, error: receipt.error }));
    if (!['passed', 'passed_with_source_skips'].includes(receipt.status)) process.exitCode = 1;
  } finally {
    process.removeListener('SIGINT', interrupt);
    process.removeListener('SIGTERM', interrupt);
  }
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) await main();

// Real three-browser mode integration. Native-only evidence cannot pass these IDs.
import assert from 'node:assert/strict';
import { isDeepStrictEqual } from 'node:util';
import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { createHash } from 'node:crypto';
import { prepareBrowserCapture, withBrowserPair } from '../../browser-capture/host.mjs';
import { caseIds, exerciseModeCase } from './mode-inputs.mjs';
export { caseIds };

export async function runCase(context) {
  assert(caseIds.includes(context.caseId), `Unsupported actual browser mode case ${context.caseId}`);
  const host = await prepareBrowserCapture(context);
  const captures = [];
  const failures = [];
  try {
    for (const engine of ['chromium', 'firefox', 'webkit']) {
      try {
        const pair = await withBrowserPair(host, engine, (page, implementation) =>
          exerciseModeCase(page, host, implementation, engine, context.caseId));
        assert(pair.reference.assertions > 0 && pair.rust.assertions > 0, 'Both implementations must execute named assertions');
        await host.saveArtifact(`${engine}-reference-capture.json`, `${JSON.stringify(pair.reference.capture)}\n`);
        await host.saveArtifact(`${engine}-rust-capture.json`, `${JSON.stringify(pair.rust.capture)}\n`);
        // Save the entire compared domain before checking it. A mismatch remains
        // a counterexample, and cannot prevent the other engines from running.
        assert(isDeepStrictEqual(pair.rust.capture, pair.reference.capture),
          `${context.caseId} ${engine} full state/events/pixels differ; inspect retained captures`);
        captures.push({ engine, runtimeVersion: pair.runtimeVersion, implementation: 'rust-wasm',
          assertions: pair.reference.assertions + pair.rust.assertions + 1,
          reference: pair.reference.capture, rust: pair.rust.capture });
      } catch (error) {
        failures.push({ engine, error: error.stack ?? String(error) });
      }
    }
    await host.saveArtifact('browser-failures.json', `${JSON.stringify(failures, null, 2)}\n`);
    await host.saveArtifact('http-requests.json', `${JSON.stringify(host.server.requests, null, 2)}\n`);
    if (failures.length) throw new AggregateError(failures.map(failure => new Error(failure.error)),
      `${context.caseId}: ${failures.map(failure => failure.engine).join(', ')} failed; inspect retained browser-failures.json`);
    const sourceHashes = [...host.build.sourceHashes];
    for (const path of ['source_repo/css/xterm.css', 'source_repo/src/common/InputHandler.test.ts',
      'source_repo/test/playwright/InputHandler.test.ts']) {
      if (!sourceHashes.some(source => source.path === path)) sourceHashes.push({ path,
        sha256: createHash('sha256').update(await readFile(resolve(context.root, path))).digest('hex') });
    }
    assert(host.server.requests.every(request => request.status === 200), 'Missing/failed assets must fail the case');
    return { caseId: context.caseId, sourceHashes, artifactHashes: host.artifacts,
      commands: host.commands, captures };
  } finally { await host.server.close(); }
}

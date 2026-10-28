// Real three-browser mode integration. Native-only evidence cannot pass these IDs.
import assert from 'node:assert/strict';
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
  try {
    for (const engine of ['chromium', 'firefox', 'webkit']) {
      const pair = await withBrowserPair(host, engine, (page, implementation) =>
        exerciseModeCase(page, host, implementation, engine, context.caseId));
      assert(pair.reference.assertions > 0 && pair.rust.assertions > 0, 'Both implementations must execute named assertions');
      await host.saveArtifact(`${engine}-reference-capture.json`, `${JSON.stringify(pair.reference.capture)}\n`);
      await host.saveArtifact(`${engine}-rust-capture.json`, `${JSON.stringify(pair.rust.capture)}\n`);
      // The entire compared domain is retained before comparison. A mismatch
      // remains a counterexample; no normalization or skipped feature clears it.
      assert.deepEqual(pair.rust.capture, pair.reference.capture, `${context.caseId} ${engine} full state/events/pixels differ`);
      captures.push({ engine, runtimeVersion: pair.runtimeVersion, implementation: 'rust-wasm',
        assertions: pair.reference.assertions + pair.rust.assertions + 1,
        reference: pair.reference.capture, rust: pair.rust.capture });
    }
    const sourceHashes = [...host.build.sourceHashes];
    for (const path of ['source_repo/css/xterm.css', 'source_repo/src/common/InputHandler.test.ts',
      'source_repo/test/playwright/InputHandler.test.ts']) {
      if (!sourceHashes.some(source => source.path === path)) sourceHashes.push({ path,
        sha256: createHash('sha256').update(await readFile(resolve(context.root, path))).digest('hex') });
    }
    await host.saveArtifact('http-requests.json', `${JSON.stringify(host.server.requests, null, 2)}\n`);
    assert(host.server.requests.every(request => request.status === 200), 'Missing/failed assets must fail the case');
    return { caseId: context.caseId, sourceHashes, artifactHashes: host.artifacts,
      commands: host.commands, captures };
  } finally { await host.server.close(); }
}

import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import test from 'node:test';

for (const value of ['0', '-1', '1.5', 'NaN', 'Infinity', '2147483648']) {
  test(`browser runner rejects invalid test timeout ${value} before jobs`, () => {
    const result = spawnSync(process.execPath,
      [fileURLToPath(new URL('./browser-host-check.mjs', import.meta.url))], {
        env: {...process.env, WASM_TEST_TIMEOUT_MS: value},
        encoding: 'utf8', timeout: 10_000,
      });
    assert.ifError(result.error);
    assert.equal(result.status, 1);
    assert.match(result.stderr, /WASM_TEST_TIMEOUT_MS must be an integer from 1 to 2147483647/);
  });
}

# Owned browser-test processes

`startManagedProcess` from `scripts/quality-process.mjs` starts one shell-free, detached process tree and returns `{ pid, ready, finished, stop }`. Only an owned `ChildProcess` handle can be passed to `stopProcessTree`; arbitrary PIDs are rejected.

The log path must be new. Creation uses exclusive access so a retry cannot overwrite a failed server attempt. Startup errors, logging failures, failed readiness callbacks, resource failures, deadlines and unexpected zero-status exits remain failed receipts. An intentional successful cleanup has status `stopped`, not `passed`.

```js
import { startManagedProcess, runCommand } from './quality-process.mjs';
import { diskHeadroom, requireHeadroom } from './quality-resources.mjs';

const checkDisk = async () => requireHeadroom(await diskHeadroom(root, 1024));
await checkDisk();
const server = startManagedProcess(process.execPath, ['demo/start.js'], {
  cwd: referenceDirectory,
  env: process.env,
  log: newServerLog,
  timeoutMs: 300_000,
  startupTimeoutMs: 15_000,
  resourceCheck: checkDisk,
  ready: async ({ signal }) => {
    try {
      return (await fetch('http://127.0.0.1:3000/test', { signal })).ok;
    } catch (error) {
      if (error.cause?.code === 'ECONNREFUSED') return false;
      throw error;
    }
  }
});
try {
  await server.ready;
  // Run the real test command here with runCommand and an independent deadline.
  // Disable the test framework's duplicate server manager.
} finally {
  const receipt = await server.stop();
  // Save receipt and treat any status other than stopped as a server failure.
}
```

Readiness callbacks return false only for expected not-yet-ready states. Exceptions terminate startup rather than becoming silent retries. Probes run at a bounded interval until the explicit startup deadline; every probe receives an AbortSignal. The lifetime deadline remains active after readiness, so forgetting cleanup cannot leave a normal live server running indefinitely.

Alternatively pass `readyPattern: /owned startup message/` for a process whose emitted message guarantees readiness. A generic log line is not proof of HTTP readiness. Verify the expected endpoint and content, and reject a pre-existing unrelated service rather than treating it as the new server.

The server's `finished` promise resolves to a receipt even after unexpected failure. Callers must retain and check it. Normal cleanup is idempotent. Windows cleanup uses bounded `taskkill /T /F`; Unix cleanup targets the private process group. Cleanup errors are recorded, not swallowed. The helper does not claim to manage services that predate its process handle, and does not kill unknown PIDs.

`results/quality/runner-tests-managed-errors.log` contains 27 passing tests across the quality helpers, including 11 managed-process cases. Independent task 114 reviews the helper. This automation is not evidence that any terminal or addon contract passed.

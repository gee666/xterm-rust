// Run the real numeric buffer bindings with the existing browser test runner.
import { readFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
process.env.WASM_PACKAGE = 'xterm-web';
process.env.WASM_TEST = 'public_buffer_numbers_wasm';
process.env.BROWSERS = 'chromium,firefox,webkit';
await import('./browser-host-check.mjs');
const path = fileURLToPath(new URL('../results/browser-host/public_buffer_numbers_wasm-results.json', import.meta.url));
const report = JSON.parse(await readFile(path, 'utf8'));
const passed = report.status === 'passed' && report.browsers.length === 3
  && report.browsers.every(row => row.status === 'passed' && row.counts?.passed === 7
    && row.counts.failed === 0 && row.counts.ignored === 0 && row.counts.filtered === 0);
console.log(`Numeric buffer bindings: ${passed ? 'PASS' : 'FAIL'}; expected 7 tests in each of 3 browsers.`);
if (!passed) process.exitCode = 1;

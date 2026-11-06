const loaded = require('P:/workshop/Windows_Worshop/oira666/xterm-rust/source_repo/test/playwright/playwright.config.ts');
const original = loaded.default ?? loaded;
module.exports = { ...original,
  testDir: 'O:/xterm-rust-swarm-16/tmp/worktrees/Chagatai566/scripts/demo-e2e',
  testMatch: 'demo-06.spec.cjs', timeout: 30000, webServer: undefined,
  use: { ...original.use, baseURL: process.env.DEMO_ORIGIN ?? 'http://127.0.0.1:1',
    viewport: { width: 1280, height: 900 }, deviceScaleFactor: 1,
    screenshot: 'only-on-failure', trace: 'retain-on-failure' }
};

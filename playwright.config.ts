import { defineConfig } from '@playwright/test';
export default defineConfig({
  testDir: './tests', timeout: 60000, workers: 1,
  use: { browserName: 'chromium', trace: 'retain-on-failure' },
  webServer: [
    { command: 'npm --prefix web/react run dev', url: 'http://127.0.0.1:5173', reuseExistingServer: !process.env.CI },
    { command: 'npm --prefix web/solid run dev', url: 'http://127.0.0.1:5174', reuseExistingServer: !process.env.CI },
  ],
});

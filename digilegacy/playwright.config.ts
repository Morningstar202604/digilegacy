import { defineConfig, devices } from '@playwright/test';

// E2E：跑在浏览器预览模式（vite dev + src/api/mock.ts）。
//
// 为什么不跑真实 Tauri 窗口：CI 是 Linux 无窗口环境，且 WebView2 无法在
// GitHub Actions 里启动。预览模式复用了与生产完全相同的 Vue 组件、store
// 与路由；唯一替换的是 IPC 后端（mock 实现了同一套命令契约）。
//
// 覆盖目标：7 个此前单测 0% 的视图的真实交互链路。
export default defineConfig({
  testDir: './e2e',
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 1 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: process.env.CI ? [['list'], ['html', { open: 'never' }]] : 'list',
  timeout: 30_000,
  expect: { timeout: 8_000 },

  use: {
    baseURL: 'http://localhost:1420',
    trace: 'retain-on-failure',
    screenshot: 'only-on-failure',
    viewport: { width: 1200, height: 800 },
  },

  projects: [
    { name: 'chromium', use: { ...devices['Desktop Chrome'] } },
  ],

  webServer: {
    command: 'npm run dev',
    url: 'http://localhost:1420',
    reuseExistingServer: !process.env.CI,
    timeout: 60_000,
  },
});

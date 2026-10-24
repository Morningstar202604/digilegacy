// 展示物料生成器：截图 + 演示视频。
//
// 用法（需要先启动站点）：
//   1) npm run dev                       # 开发预览，http://localhost:1420
//   2) node scripts/demo-media.mjs       # 截图到 docs/screenshots，视频到 docs/media
//
// 可选：DEMO_BASE=http://localhost:4173/digilegacy/demo/ 针对静态演示构建出图。
//
// 为什么用 Playwright 而不是裸 CDP：视频录制需要浏览器上下文级 API，
// 且 Playwright 已是本项目的 E2E 依赖，不再引入新工具。

import { chromium } from '@playwright/test';
import { mkdir, readFile, unlink, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const HERE = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(HERE, '..', '..');
const SHOTS = path.join(ROOT, 'docs', 'screenshots');
const MEDIA = path.join(ROOT, 'docs', 'media');
const BASE = process.env.DEMO_BASE || 'http://localhost:1420';

const PAGES = [
  { file: '01-dashboard', path: '/dashboard', title: '仪表板' },
  { file: '02-discover', path: '/discover', title: '资产发现' },
  { file: '03-assets', path: '/assets', title: '我的数字资产' },
  { file: '04-asset-detail', path: '/assets/1', title: '资产详情' },
  { file: '05-profile', path: '/profile', title: '我的档案' },
  { file: '06-dms', path: '/dms', title: '安心传承' },
  { file: '07-report', path: '/report', title: '继承报告' },
  { file: '08-heir-guide', path: '/heir', title: '继承人向导' },
  { file: '09-legal', path: '/legal', title: '法律模板' },
];

const VIEWPORT = { width: 1200, height: 800 };

async function seed(page) {
  // 必须先让应用建出演示状态，再改写它——首次访问时 localStorage 还是空的，
  // 直接 addInitScript 拿不到 state，截图就会显示「未启用加密」。
  await page.goto(`${BASE}/dashboard`, { waitUntil: 'domcontentloaded' });
  await page.waitForTimeout(900);
  await page.evaluate(() => {
    const KEY = 'digilegacy-web-preview-v1';
    const raw = localStorage.getItem(KEY);
    if (raw) {
      try {
        const state = JSON.parse(raw);
        state.masterPassword = 'demo-capture-pass';
        localStorage.setItem(KEY, JSON.stringify(state));
      } catch {
        /* 损坏的演示状态会在应用侧重建 */
      }
    }
    sessionStorage.setItem('digilegacy-web-preview-unlocked', '1');
    localStorage.setItem('digilegacy_onboarding_done', '1');
  });
  await page.reload({ waitUntil: 'networkidle' });
}

async function captureScreenshots(browser) {
  await mkdir(SHOTS, { recursive: true });
  const context = await browser.newContext({ viewport: VIEWPORT, deviceScaleFactor: 2 });
  const page = await context.newPage();

  const consoleErrors = [];
  page.on('console', (m) => {
    if (m.type() === 'error') consoleErrors.push(m.text());
  });
  page.on('pageerror', (e) => consoleErrors.push('PAGEERROR ' + e.message));

  await seed(page);
  for (const item of PAGES) {
    await page.goto(`${BASE}${item.path}`, { waitUntil: 'networkidle' });
    await page.waitForTimeout(700);
    const heading = await page.locator('.header-title').textContent().catch(() => '');
    if (!heading?.trim()) throw new Error(`${item.file}: 页面标题为空，未渲染完成`);
    await page.screenshot({ path: path.join(SHOTS, `${item.file}.png`) });
    console.log(`captured ${item.file}.png  ← ${item.path} 「${heading.trim()}」`);
  }

  // 深色模式截图（展示品牌在暗色下的表现）
  await page.goto(`${BASE}/dashboard`, { waitUntil: 'networkidle' });
  await page.evaluate(() => {
    localStorage.setItem('digilegacy_theme', 'dark');
    document.documentElement.setAttribute('data-theme', 'dark');
    document.documentElement.style.colorScheme = 'dark';
  });
  await page.waitForTimeout(700);
  await page.screenshot({ path: path.join(SHOTS, '10-dashboard-dark.png') });
  console.log('captured 10-dashboard-dark.png');

  await context.close();
  if (consoleErrors.length > 0) {
    console.warn(`注意：截图过程中出现 ${consoleErrors.length} 条控制台错误：`);
    for (const e of consoleErrors.slice(0, 5)) console.warn('  ' + e);
  }
}

async function recordWalkthrough(browser) {
  await mkdir(MEDIA, { recursive: true });
  const context = await browser.newContext({
    viewport: VIEWPORT,
    recordVideo: { dir: MEDIA, size: VIEWPORT },
  });
  const page = await context.newPage();
  await seed(page);

  const pause = (ms) => page.waitForTimeout(ms);

  await page.goto(`${BASE}/dashboard`, { waitUntil: 'networkidle' });
  await pause(1600);

  // 资产发现：选类别 → 勾平台 → 填详情 → 保存
  await page.goto(`${BASE}/discover`, { waitUntil: 'networkidle' });
  await pause(900);
  await page.getByRole('button', { name: /继续/ }).click();
  await pause(700);
  await page.locator('.category-card').first().click();
  await pause(500);
  await page.getByRole('button', { name: /下一步：勾选平台/ }).click();
  await pause(900);
  await page.locator('.platform-card').first().click();
  await pause(500);
  await page.getByRole('button', { name: /下一步：设定资产详情/ }).click();
  await pause(900);
  const heir = page.locator('input[id^="heir-name-"]').first();
  if (await heir.count()) await heir.fill('家人');
  await pause(600);
  await page.getByRole('button', { name: /保存全部资产/ }).click();
  await pause(1400);

  // 资产列表 → 详情
  await page.goto(`${BASE}/assets`, { waitUntil: 'networkidle' });
  await pause(1000);
  await page.locator('.asset-row').first().click();
  await pause(1400);
  await page.locator('#detail-disposal').selectOption('inherit');
  await pause(500);
  await page.getByRole('button', { name: '保存修改' }).click();
  await pause(1200);

  // 安心传承
  await page.goto(`${BASE}/dms`, { waitUntil: 'networkidle' });
  await pause(1000);
  await page.getByRole('button', { name: /我还安好/ }).click();
  await pause(1200);

  // 继承报告
  await page.goto(`${BASE}/report`, { waitUntil: 'networkidle' });
  await pause(1800);
  await page.mouse.wheel(0, 700);
  await pause(900);
  await page.mouse.wheel(0, 700);
  await pause(1200);

  // 继承人向导 + 法律模板
  await page.goto(`${BASE}/heir`, { waitUntil: 'networkidle' });
  await pause(1200);
  await page.getByRole('tab', { name: /变现/ }).click();
  await pause(1400);
  await page.goto(`${BASE}/legal`, { waitUntil: 'networkidle' });
  await pause(900);
  await page.getByRole('button', { name: '预览' }).first().click();
  await pause(1500);
  await page.keyboard.press('Escape');
  await pause(600);

  // 回到仪表板收尾
  await page.goto(`${BASE}/dashboard`, { waitUntil: 'networkidle' });
  await pause(1600);

  const video = page.video();
  await context.close();
  if (!video) throw new Error('未生成视频');
  const src = await video.path();
  const target = path.join(MEDIA, 'demo-walkthrough.webm');
  const buf = await readFile(src);
  await writeFile(target, buf);
  // Playwright 会额外留下 page@<hash>.webm，删除避免仓库里出现重复文件
  if (path.resolve(src) !== path.resolve(target)) {
    await unlink(src).catch(() => {});
  }
  console.log(`recorded ${path.relative(ROOT, target)}`);
}

const browser = await chromium.launch();
try {
  await captureScreenshots(browser);
  if (process.env.SKIP_VIDEO !== '1') await recordWalkthrough(browser);
} finally {
  await browser.close();
}

// 截图脚本：驱动 headless Chrome 打开 Vite 预览模式，按应用真实 1200x800 视口逐页截取。
// 走 CDP（Node 原生 WebSocket），不依赖 playwright/puppeteer，也不受桌面 DPI 缩放影响。
//
// 用法：先 `npm run dev`，再 `node capture.mjs [--out <dir>] [--port 1420]`

import { spawn } from 'node:child_process';
import { mkdir, writeFile, rm } from 'node:fs/promises';
import { existsSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import os from 'node:os';

const HERE = path.dirname(fileURLToPath(import.meta.url));
const args = process.argv.slice(2);
const argOf = (flag, fallback) => {
  const i = args.indexOf(flag);
  return i >= 0 && args[i + 1] ? args[i + 1] : fallback;
};

const OUT_DIR = path.resolve(argOf('--out', path.join(HERE, '..', 'docs', 'screenshots')));
const PORT = Number(argOf('--port', '1420'));
const BASE = `http://localhost:${PORT}`;

const VIEWPORT = { width: 1200, height: 800, deviceScaleFactor: 2 };

const CHROME_CANDIDATES = [
  'C:/Program Files/Google/Chrome/Application/chrome.exe',
  'C:/Program Files (x86)/Google/Chrome/Application/chrome.exe',
  'C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe',
  'C:/Program Files/Microsoft/Edge/Application/msedge.exe',
];

const PAGES = [
  { file: '01-dashboard', path: '/dashboard', wait: 'main' },
  { file: '02-discover', path: '/discover', wait: 'main' },
  { file: '03-assets', path: '/assets', wait: 'main' },
  { file: '04-profile', path: '/profile', wait: 'main' },
  { file: '05-dms', path: '/dms', wait: 'main' },
  { file: '06-report', path: '/report', wait: 'main' },
  { file: '07-heir-guide', path: '/heir', wait: 'main' },
  { file: '08-legal', path: '/legal', wait: 'main' },
];

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

function findChrome() {
  for (const p of CHROME_CANDIDATES) if (existsSync(p)) return p;
  throw new Error('未找到 Chrome/Edge，可安装其一或改 CHROME_CANDIDATES');
}

async function waitForHttp(url, timeoutMs = 30000) {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    try {
      const res = await fetch(url);
      if (res.ok) return true;
    } catch {
      /* dev server 未就绪，继续等 */
    }
    await sleep(400);
  }
  throw new Error(`dev server 未在 ${timeoutMs}ms 内就绪：${url}`);
}

class CDP {
  #ws;
  #id = 0;
  #pending = new Map();
  #handlers = new Map();

  static async attach(wsUrl) { const c = new CDP(); await c.#open(wsUrl); return c; }

  #open(wsUrl) {
    return new Promise((resolve, reject) => {
      this.#ws = new WebSocket(wsUrl);
      this.#ws.onopen = () => resolve();
      this.#ws.onerror = (e) => reject(new Error(`CDP 连接失败：${e.message ?? 'unknown'}`));
      this.#ws.onclose = () => {
        for (const { reject: rj } of this.#pending.values()) rj(new Error('CDP 连接已关闭'));
        this.#pending.clear();
      };
      this.#ws.onmessage = (ev) => {
        const msg = JSON.parse(ev.data);
        if (msg.id && this.#pending.has(msg.id)) {
          const { resolve: rs, reject: rj } = this.#pending.get(msg.id);
          this.#pending.delete(msg.id);
          msg.error ? rj(new Error(`${msg.error.message} (${msg.error.code})`)) : rs(msg.result);
          return;
        }
        const hs = this.#handlers.get(msg.method);
        if (hs) for (const h of hs) h(msg.params);
      };
    });
  }

  send(method, params = {}) {
    const id = ++this.#id;
    this.#ws.send(JSON.stringify({ id, method, params }));
    return new Promise((resolve, reject) => this.#pending.set(id, { resolve, reject }));
  }

  on(method, handler) {
    if (!this.#handlers.has(method)) this.#handlers.set(method, new Set());
    this.#handlers.get(method).add(handler);
    return () => this.#handlers.get(method).delete(handler);
  }

  close() { try { this.#ws.close(); } catch { /* 已关闭 */ } }
}

async function waitForLoad(cdp, timeoutMs = 20000) {
  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => { off(); reject(new Error('页面加载超时')); }, timeoutMs);
    const off = cdp.on('Page.loadEventFired', () => { clearTimeout(timer); off(); resolve(); });
  });
}

async function evaluate(cdp, expression) {
  const { result, exceptionDetails } = await cdp.send('Runtime.evaluate', {
    expression, awaitPromise: true, returnByValue: true,
  });
  if (exceptionDetails) throw new Error(exceptionDetails.text ?? '页面脚本异常');
  return result.value;
}

async function capture(cdp, filePath) {
  const { data } = await cdp.send('Page.captureScreenshot', {
    format: 'png', captureBeyondViewport: false, fromSurface: true,
  });
  await writeFile(filePath, Buffer.from(data, 'base64'));
}

async function main() {
  await waitForHttp(`${BASE}/dashboard`);
  await mkdir(OUT_DIR, { recursive: true });

  const userDataDir = path.join(os.tmpdir(), `digilegacy-capture-${process.pid}`);
  const chrome = spawn(findChrome(), [
    '--headless=new',
    '--disable-gpu',
    '--hide-scrollbars',
    '--no-first-run',
    '--no-default-browser-check',
    '--force-device-scale-factor=1',
    '--window-size=' + VIEWPORT.width + ',' + VIEWPORT.height,
    '--remote-debugging-port=0',
    `--user-data-dir=${userDataDir}`,
    'about:blank',
  ], { stdio: ['ignore', 'pipe', 'pipe'] });

  const { wsUrl, port } = await new Promise((resolve, reject) => {
    let buf = '';
    const timer = setTimeout(() => reject(new Error('Chrome 启动超时')), 30000);
    const onData = (chunk) => {
      buf += chunk.toString();
      const m = buf.match(/DevTools listening on (ws:\/\/[^\s/]+)(\/devtools\/browser\/[^\s]+)/);
      if (m) {
        clearTimeout(timer);
        resolve({ wsUrl: m[1] + m[2], port: new URL(m[1]).port });
      }
    };
    chrome.stdout.on('data', onData);
    chrome.stderr.on('data', onData);
    chrome.on('exit', (code) => { clearTimeout(timer); reject(new Error(`Chrome 异常退出 (${code})`)); });
  });

  // 取 about:blank 页面的独立 target，直接按页面级 WebSocket 驱动
  let pageTarget;
  for (let i = 0; i < 30 && !pageTarget; i++) {
    const list = await fetch(`http://127.0.0.1:${port}/json/list`).then((r) => r.json()).catch(() => []);
    pageTarget = list.find((t) => t.type === 'page' && t.webSocketDebuggerUrl);
    if (!pageTarget) await sleep(200);
  }
  if (!pageTarget) throw new Error('未找到可调试的页面 target');

  const pageCdp = await CDP.attach(pageTarget.webSocketDebuggerUrl);
  try {
    await pageCdp.send('Page.enable');
    await pageCdp.send('Runtime.enable');
    await pageCdp.send('Emulation.setDeviceMetricsOverride', {
      width: VIEWPORT.width, height: VIEWPORT.height,
      deviceScaleFactor: VIEWPORT.deviceScaleFactor, mobile: false,
    });

    // 首跳建出预览态 localStorage，再重置为「已启用主密码 + 未锁定」的演示态：
    // 侧栏显示加密已启用，头部出现锁定按钮，与产品主张一致；引导弹层同时标记为已完成。
    await pageCdp.send('Page.navigate', { url: `${BASE}/dashboard` });
    await waitForLoad(pageCdp);
    await sleep(1200);
    await evaluate(pageCdp, `(() => {
      const KEY = 'digilegacy-web-preview-v1';
      const raw = localStorage.getItem(KEY);
      const state = raw ? JSON.parse(raw) : null;
      if (state) {
        state.masterPassword = 'demo-capture-pass';
        localStorage.setItem(KEY, JSON.stringify(state));
      }
      localStorage.setItem('digilegacy_onboarding_done', '1');
      sessionStorage.setItem('digilegacy-web-preview-unlocked', '1');
      return true;
    })()`);

    for (const page of PAGES) {
      await pageCdp.send('Page.navigate', { url: `${BASE}${page.path}` });
      await waitForLoad(pageCdp);
      await sleep(1400);
      const url = await evaluate(pageCdp, 'location.pathname');
      if (url !== page.path) throw new Error(`${page.file}: 期望路径 ${page.path}，实际 ${url}`);
      const title = await evaluate(pageCdp, 'document.querySelector(".header-title")?.textContent?.trim() ?? ""');
      if (!title) throw new Error(`${page.file}: 页面标题为空，可能未渲染完成`);
      const out = path.join(OUT_DIR, `${page.file}.png`);
      await capture(pageCdp, out);
      console.log(`captured ${page.file}.png  ← ${page.path}  「${title}」`);
    }
  } finally {
    pageCdp.close();
    chrome.kill();
    await sleep(300);
    await rm(userDataDir, { recursive: true, force: true }).catch(() => {});
  }
}

main().catch((err) => {
  console.error('截图失败：', err.message);
  process.exitCode = 1;
});

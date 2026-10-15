// 归零 · 云端心跳服务器（参考实现）
//
// 设计原则（与客户端 heartbeat.rs 对应）：
//   1. 零内容：只接收令牌，不接收/不存储任何资产、凭据、档案数据；
//   2. 失联提醒：令牌超期未心跳 → 通过预注册的 Webhook（如邮件网关）发提醒；
//   3. 可自托管：单文件、无外部服务依赖，数据仅 内存/一个 JSON 文件。
//
// 运行：HEARTBEAT_PORT=8080 node heartbeat-server.mjs
// 客户端环境变量：DMS_HEARTBEAT_ENABLED=1  DMS_HEARTBEAT_TOKEN=<令牌>
//                DMS_HEARTBEAT_ENDPOINT=http://<host>:8080
//
// 注册令牌：向 registrations.json 添加 {"<token>": {"notify_webhook": "...", "grace_days": 21}}
// Webhook 载荷：POST {"token","last_seen","silent_days"} —— 由网关侧转换为邮件/短信。

import http from 'node:http';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const PORT = Number(process.env.HEARTBEAT_PORT || 8080);
const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REG_FILE = path.join(__dirname, 'registrations.json');
const STATE_FILE = path.join(__dirname, 'state.json');

function loadJson(file, fallback) {
  try {
    return JSON.parse(fs.readFileSync(file, 'utf8'));
  } catch {
    return fallback;
  }
}

const registrations = loadJson(REG_FILE, {});
const state = loadJson(STATE_FILE, {}); // { [token]: last_seen_iso }

function saveState() {
  fs.writeFileSync(STATE_FILE, JSON.stringify(state, null, 2));
}

const server = http.createServer((req, res) => {
  if (req.method === 'POST' && req.url === '/heartbeat') {
    let body = '';
    req.on('data', (c) => { body += c; if (body.length > 4096) req.destroy(); });
    req.on('end', () => {
      try {
        const { token } = JSON.parse(body);
        if (!token || typeof token !== 'string') {
          res.writeHead(400).end('{"error":"token required"}');
          return;
        }
        state[token] = new Date().toISOString();
        saveState();
        res.writeHead(200).end('{"ok":true}');
      } catch {
        res.writeHead(400).end('{"error":"bad json"}');
      }
    });
    return;
  }

  if (req.method === 'GET' && req.url === '/check') {
    const now = Date.now();
    const alerts = [];
    for (const [token, reg] of Object.entries(registrations)) {
      const last = state[token];
      const graceDays = reg.grace_days ?? 21;
      if (!last) continue;
      const silentDays = (now - new Date(last).getTime()) / 86400000;
      if (silentDays >= graceDays && !reg._alerted) {
        alerts.push({ token, last_seen: last, silent_days: Math.floor(silentDays), webhook: reg.notify_webhook });
        reg._alerted = true; // 内存标记，避免重复轰炸；真实部署可用数据库
      }
    }
    res.writeHead(200, { 'Content-Type': 'application/json' });
    res.end(JSON.stringify({ alerts, checked_at: new Date().toISOString() }, null, 2));
    return;
  }

  res.writeHead(404).end('not found');
});

server.listen(PORT, () => {
  console.log(`心跳服务器已启动: http://localhost:${PORT}`);
  console.log(`  POST /heartbeat  {"token":"..."}`);
  console.log(`  GET  /check      检查超时令牌并返回提醒载荷（由定时任务调用后转发 webhook）`);
});

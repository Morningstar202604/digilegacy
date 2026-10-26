#!/usr/bin/env node
// 文档数字一致性检查：测试数/命令数/表数这类「手抄数字」在 6 个文件里各自维护，
// 历史上漂移过多次（265 → 318 → 330 → 332 → 341 → 347）。这里从源码重新推导，
// 再与文档声明比对，不一致就失败并指出该改哪里。
//
// 用法：node scripts/check-doc-facts.mjs [--json]
// 退出码：0 一致 / 1 不一致（打印差异）

import { readFileSync } from 'node:fs';
import path from 'node:path';
import { facts, e2eCount, ROOT } from './doc-facts-lib.mjs';

const base = facts();
const facts_ = { ...base, e2e: e2eCount() };

function read(rel) {
  return readFileSync(path.join(ROOT, rel), 'utf8');
}

const problems = [];

function expectIn(file, pattern, expected, label) {
  const text = read(file);
  const m = text.match(pattern);
  if (!m) {
    problems.push(`${file}: 找不到 ${label} 的声明（期望 ${expected}）`);
    return;
  }
  const actual = Number(m[1]);
  if (actual !== expected) {
    problems.push(`${file}: ${label} 声明为 ${actual}，源码推导为 ${expected}`);
  }
}

// README 中英：测试徽章与验证表
for (const file of ['README.md', 'README.zh-CN.md']) {
  expectIn(file, /tests-(\d+)%20Rust/, facts_.rustTests, '徽章 Rust 测试数');
  expectIn(file, /%20(\d+)%20frontend/, facts_.frontendCases, '徽章前端测试数');
  expectIn(
    file,
    /(?:Frontend unit tests|前端单元测试)\s*\|\s*\*\*(\d+)/,
    facts_.frontendCases,
    '验证表前端测试数',
  );
  expectIn(
    file,
    /(?:Rust tests|Rust 测试)\s*\|\s*\*\*(\d+)/,
    facts_.rustTests,
    '验证表 Rust 测试数',
  );
  if (facts_.e2e > 0) {
    expectIn(file, /tests-\d+%20Rust%20%C2%B7%20\d+%20frontend%20%C2%B7%20(\d+)%20E2E/, facts_.e2e, '徽章 E2E 数');
  }
}

// 官网双语：首屏 pill 与「已验证」两处都要对上
for (const file of ['docs/index.html', 'docs/zh/index.html']) {
  const text = read(file);
  for (const m of text.matchAll(/<li class="pill">(\d+)\s*(?:Rust|项 Rust)\s*·\s*(\d+)\s*(?:frontend|项前端)/g)) {
    if (Number(m[1]) !== facts_.rustTests) {
      problems.push(`${file}: 首屏 Rust 测试数声明为 ${m[1]}，源码推导为 ${facts_.rustTests}`);
    }
    if (Number(m[2]) !== facts_.frontendCases) {
      problems.push(`${file}: 首屏前端测试数声明为 ${m[2]}，源码推导为 ${facts_.frontendCases}`);
    }
  }
  for (const m of text.matchAll(/<strong>已(?:验证|Verified)<\/strong>(\d+)\s*(?:项 Rust|Rust)\s*\+\s*(\d+)\s*(?:项前端|frontend)/g)) {
    if (Number(m[1]) !== facts_.rustTests) {
      problems.push(`${file}: 「已验证」Rust 数声明为 ${m[1]}，源码推导为 ${facts_.rustTests}`);
    }
    if (Number(m[2]) !== facts_.frontendCases) {
      problems.push(`${file}: 「已验证」前端数声明为 ${m[2]}，源码推导为 ${facts_.frontendCases}`);
    }
  }
}

// IPC 命令数与平台数
for (const file of ['README.md', 'README.zh-CN.md']) {
  expectIn(file, /\*\*(\d+)\*\*\s*(?:IPC commands|个 IPC 命令)/, facts_.ipc, 'IPC 命令数');
  expectIn(file, /\*\*(\d+)\*\*\s*(?:platforms|个平台)/, facts_.platforms, '平台数');
}
expectIn('docs/ARCHITECTURE.md', /Tauri IPC \((\d+) commands\)/, facts_.ipc, 'IPC 命令数');
expectIn('docs/ARCHITECTURE.md', /built-in (\d+)-platform policy database/, facts_.platforms, '平台库条数');

if (process.argv.includes('--json')) {
  console.log(JSON.stringify({ facts: facts_, problems }, null, 2));
} else {
  console.log('源码推导事实：');
  console.log(`  Rust 测试        ${facts_.rustTests}`);
  console.log(`  前端测试         ${facts_.frontendCases}（${facts_.frontendFiles} 文件）`);
  console.log(`  IPC 命令         ${facts_.ipc}`);
  console.log(`  SQLite 表        ${facts_.tables}`);
  console.log(`  内置平台         ${facts_.platforms}`);
  console.log(`  E2E 用例         ${facts_.e2e || '未校验（playwright --list 不可用）'}`);
  if (problems.length) {
    console.log('\n文档数字不一致：');
    for (const p of problems) console.log(`  - ${p}`);
  } else {
    console.log('\n文档数字与源码一致。');
  }
}

process.exit(problems.length ? 1 : 0);

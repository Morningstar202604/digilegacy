// E2E 覆盖率收集：Playwright 跑完后从页面里取 window.__coverage__，合并成报告。
//
// 用法：
//   E2E_COVERAGE=1 npx playwright test        # 先带插桩跑
//   node e2e/collect-coverage.mjs             # 再生成报告
//
// 说明：E2E 覆盖率与单元覆盖率是两套口径，不合并进 vitest 的阈值门禁。
// 它的价值是回答「用户真的点到的路径，代码执行了多少」，单测无法替代。

import { readFile, writeFile, mkdir } from 'node:fs/promises';
import path from 'node:path';
import { existsSync } from 'node:fs';

const HERE = path.dirname(new URL(import.meta.url).pathname.replace(/^\/([A-Za-z]:)/, '$1'));
const COV_DIR = path.join(HERE, '.coverage');
const OUT = path.join(HERE, '..', 'coverage-e2e');

async function main() {
  if (!existsSync(COV_DIR)) {
    console.error('未找到 E2E 覆盖率数据。请先运行：E2E_COVERAGE=1 npx playwright test');
    process.exitCode = 1;
    return;
  }

  const files = (await import('node:fs')).readdirSync(COV_DIR).filter(f => f.endsWith('.json'));
  if (files.length === 0) {
    console.error('覆盖率目录为空。');
    process.exitCode = 1;
    return;
  }

  // 合并所有分片的 window.__coverage__（按文件累加命中次数）
  const merged = {};
  for (const f of files) {
    const raw = JSON.parse(await readFile(path.join(COV_DIR, f), 'utf-8'));
    for (const [file, data] of Object.entries(raw)) {
      if (!merged[file]) {
        merged[file] = data;
        continue;
      }
      for (const [id, counts] of Object.entries(data.s)) {
        if (!merged[file].s[id]) merged[file].s[id] = 0;
        merged[file].s[id] += counts;
      }
      for (const [id, counts] of Object.entries(data.f || {})) {
        if (!merged[file].f[id]) merged[file].f[id] = 0;
        merged[file].f[id] += counts;
      }
    }
  }

  await mkdir(OUT, { recursive: true });
  await writeFile(path.join(OUT, 'coverage-final.json'), JSON.stringify(merged, null, 2));

  // 用 istanbul-lib-coverage 汇总（vite-plugin-istanbul 的依赖里已有）。
  // 它是 CJS 包，用 createRequire 加载，避免 ESM 互操作差异。
  let summary;
  try {
    const { createRequire } = await import('node:module');
    const require = createRequire(import.meta.url);
    const libCoverage = require('istanbul-lib-coverage');
    const map = libCoverage.createCoverageMap(merged);
    const sum = libCoverage.createCoverageSummary();
    map.files().forEach(f => sum.merge(map.fileCoverageFor(f).toSummary()));
    summary = {
      statements: sum.statements.pct,
      branches: sum.branches.pct,
      functions: sum.functions.pct,
      lines: sum.lines.pct,
      files: map.files().length,
    };
  } catch {
    summary = { note: 'istanbul-lib-coverage 不可用，仅输出原始 coverage-final.json' };
  }

  await writeFile(path.join(OUT, 'summary.json'), JSON.stringify(summary, null, 2));
  console.log('E2E 覆盖率已生成：', path.join(OUT, 'summary.json'));
  console.log(summary);
}

main();

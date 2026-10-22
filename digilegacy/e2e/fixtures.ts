// E2E 覆盖率 fixture：每个用例结束后把 window.__coverage__ 落到 .coverage/。
//
// 用法：在 spec 里 import { test, expect } from './fixtures';
// 或在全部 spec 中统一替换 '@playwright/test' 的导入。
//
// 只有 E2E_COVERAGE=1 时才采集，否则零开销。

import { test as base, expect } from '@playwright/test';
import { mkdir, writeFile } from 'node:fs/promises';
import path from 'node:path';

const HERE = path.dirname(new URL(import.meta.url).pathname.replace(/^\/([A-Za-z]:)/, '$1'));
const OUT = path.join(HERE, '.coverage');
const enabled = process.env.E2E_COVERAGE === '1';

export const test = base.extend({
  page: async ({ page }, use, testInfo) => {
    await use(page);

    if (!enabled) return;
    try {
      const coverage = await page.evaluate(() => (window as unknown as { __coverage__?: unknown }).__coverage__);
      if (!coverage) return;
      await mkdir(OUT, { recursive: true });
      const name = `${testInfo.testId}-${testInfo.title.replace(/[^\w\u4e00-\u9fa5]+/g, '_').slice(0, 40)}.json`;
      await writeFile(path.join(OUT, name), JSON.stringify(coverage));
    } catch {
      // 页面已关闭或未插桩：静默跳过，不影响测试结果
    }
  },
});

export { expect };

import { beforeEach, describe, expect, it, vi } from 'vitest';
import { flushPromises, mount } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import type { Mock } from 'vitest';

// ReportView 的两个导出按钮此前失败时完全静默：
// 点击后界面毫无反应，用户会以为文件已生成却找不到。
vi.mock('../api/invoke', () => ({
  invoke: vi.fn(),
  isTauriEnv: () => false,
}));

import ReportView from './ReportView.vue';
import { invoke } from '../api/invoke';
import { silenceExpectedErrors } from '../test-utils/silence';

silenceExpectedErrors();

const mockedInvoke = invoke as unknown as Mock;

/** ZIP 路径包含动态 import + JSZip 压缩 + btoa，jsdom 下需要真实事件循环。 */
async function settle(w: { vm: { $nextTick: () => Promise<unknown> } }) {
  for (let i = 0; i < 4; i++) { await flushPromises(); await w.vm.$nextTick(); }
  await new Promise(r => setTimeout(r, 400));
  for (let i = 0; i < 4; i++) { await flushPromises(); await w.vm.$nextTick(); }
}

function reportData() {
  return {
    assets: [],
    property_assets_count: 0,
    personality_assets_count: 0,
    mixed_assets_count: 0,
    total_estimated_value: 0,
    generated_at: '2026-01-01 00:00:00',
    user: {
      full_name: '测试用户', phone: '13800000000', email: 'a@b.com',
      heir_primary_name: '继承人', heir_primary_relation: '子女',
      emergency_contact_name: '联系人', emergency_contact_relation: '配偶',
    },
  };
}

async function mountReport(over: Record<string, unknown> = {}) {
  mockedInvoke.mockImplementation(async (cmd: string) => {
    if (cmd in over) return over[cmd as keyof typeof over];
    if (cmd === 'get_inheritance_report') return { success: true, message: '', data: reportData() };
    if (cmd === 'get_all_platforms') return { success: true, message: '', data: [] };
    return { success: true, message: '', data: null };
  });
  const pinia = createPinia();
  setActivePinia(pinia);
  const w = mount(ReportView, { global: { plugins: [pinia] } });
  await flushPromises();
  await w.vm.$nextTick();
  return w;
}

beforeEach(() => {
  setActivePinia(createPinia());
  mockedInvoke.mockReset();
});

describe('ReportView 导出反馈', () => {
  it('导出成功时显示落地路径', async () => {
    const w = await mountReport({
      export_report_to_file: { success: true, message: 'ok', data: 'C:/app/报告.html' },
    });

    const btn = w.findAll('button').find(b => b.text().includes('导出报告'))!;
    await btn.trigger('click');
    await flushPromises();

    expect(w.text()).toContain('C:/app/报告.html');
  });

  it('导出返回 success:false 时必须显示错误（不得静默）', async () => {
    const w = await mountReport({
      export_report_to_file: { success: false, message: '磁盘已满', data: null },
    });

    const btn = w.findAll('button').find(b => b.text().includes('导出报告'))!;
    await btn.trigger('click');
    await flushPromises();

    expect(w.text(), '导出失败必须给用户可见提示').toContain('磁盘已满');
  });

  it('导出抛异常时必须显示错误（不得静默）', async () => {
    const w = await mountReport();
    mockedInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === 'export_report_to_file') throw new Error('IPC 断开');
      if (cmd === 'get_inheritance_report') return { success: true, message: '', data: reportData() };
      return { success: true, message: '', data: [] };
    });

    const btn = w.findAll('button').find(b => b.text().includes('导出报告'))!;
    await btn.trigger('click');
    await flushPromises();

    expect(w.text(), '异常必须转成用户可见提示').toContain('导出失败');
  });
});

describe('导出继承工具包（ZIP）', () => {
  it('生成 ZIP 并调用 export_zip_to_file，内容为 base64', async () => {
    const w = await mountReport({
      export_zip_to_file: { success: true, message: 'ok', data: 'C:/app/工具包.zip' },
    });

    const btn = w.findAll('button').find(b => b.text().includes('导出继承工具包'))!;
    expect(btn, 'ZIP 导出按钮必须存在').toBeTruthy();
    await btn.trigger('click');
    // JSZip 生成是异步的，需要多轮
    for (let i = 0; i < 5; i++) { await flushPromises(); await w.vm.$nextTick(); }
    await new Promise(r => setTimeout(r, 500));
    for (let i = 0; i < 5; i++) { await flushPromises(); await w.vm.$nextTick(); }

    console.log('DIAG calls=', mockedInvoke.mock.calls.map(c=>c[0]).join(','));
    const zipCalls = mockedInvoke.mock.calls.filter(c => c[0] === 'export_zip_to_file');
    expect(zipCalls.length, '必须调用 export_zip_to_file').toBe(1);
    const payload = zipCalls[0][1] as { content: string };
    expect(typeof payload.content).toBe('string');
    expect(payload.content.length).toBeGreaterThan(100);
    expect(w.text()).toContain('工具包.zip');
  });

  it('ZIP 导出失败必须给出可见提示', async () => {
    const w = await mountReport({
      export_zip_to_file: { success: false, message: '磁盘空间不足', data: null },
    });

    const btn = w.findAll('button').find(b => b.text().includes('导出继承工具包'))!;
    await btn.trigger('click');
    await settle(w);

    expect(w.text()).toContain('磁盘空间不足');
  });

  it('ZIP 导出抛异常时提示失败', async () => {
    const w = await mountReport();
    mockedInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === 'get_inheritance_report') return { success: true, message: '', data: reportData() };
      if (cmd === 'get_all_platforms') return { success: true, message: '', data: [] };
      if (cmd === 'export_zip_to_file') throw new Error('IPC 断开');
      return { success: true, message: '', data: null };
    });

    const btn = w.findAll('button').find(b => b.text().includes('导出继承工具包'))!;
    await btn.trigger('click');
    await settle(w);

    expect(w.text()).toContain('导出工具包失败');
  });
});

describe('打印', () => {
  it('点打印调用 window.print', async () => {
    const printSpy = vi.fn();
    vi.stubGlobal('print', printSpy);
    const w = await mountReport();

    const btn = w.findAll('button').find(b => b.text().includes('打印'))!;
    expect(btn, '打印按钮必须存在').toBeTruthy();
    await btn.trigger('click');

    expect(printSpy).toHaveBeenCalled();
  });
});

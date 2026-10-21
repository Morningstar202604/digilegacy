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

const mockedInvoke = invoke as unknown as Mock;

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

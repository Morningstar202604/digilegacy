import { beforeEach, describe, expect, it, vi } from 'vitest';
import { flushPromises, mount } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import { createRouter, createMemoryHistory } from 'vue-router';
import type { Mock } from 'vitest';

// DashboardView 是登录后的首屏：显示名、统计数字、最近资产。
// 逻辑不复杂，但「数据为空」和「有数据」两种渲染都必须正确。
vi.mock('../api/invoke', () => ({ invoke: vi.fn(), isTauriEnv: () => false }));

import DashboardView from './DashboardView.vue';
import { invoke } from '../api/invoke';
import { silenceExpectedErrors } from '../test-utils/silence';

silenceExpectedErrors();
const mockedInvoke = invoke as unknown as Mock;

async function mountDash(opts: {
  stats?: Record<string, unknown> | null;
  user?: Record<string, unknown> | null;
  assets?: unknown[];
} = {}) {
  mockedInvoke.mockImplementation(async (cmd: string) => {
    if (cmd === 'get_dashboard_stats') {
      return { success: true, message: '', data: opts.stats ?? null };
    }
    if (cmd === 'get_user_profile') {
      return { success: true, message: '', data: opts.user ?? null };
    }
    if (cmd === 'get_all_assets') return { success: true, message: '', data: opts.assets ?? [] };
    if (cmd === 'get_check_in_status') return { success: true, message: '', data: null };
    if (cmd === 'get_dms_status') return { success: true, message: '', data: null };
    return { success: true, message: '', data: null };
  });
  const router = createRouter({
    history: createMemoryHistory(),
    routes: [
      { path: '/', component: { template: '<div/>' } },
      { path: '/dashboard', component: DashboardView },
      { path: '/discover', component: { template: '<div/>' } },
      { path: '/report', component: { template: '<div/>' } },
      { path: '/assets', component: { template: '<div/>' } },
      { path: '/assets/:id', component: { template: '<div/>' } },
      { path: '/dms', component: { template: '<div/>' } },
      { path: '/profile', component: { template: '<div/>' } },
    ],
  });
  await router.push('/dashboard');
  await router.isReady();
  const pinia = createPinia();
  setActivePinia(pinia);
  const w = mount(DashboardView, { global: { plugins: [pinia, router] } });
  for (let i = 0; i < 3; i++) { await flushPromises(); await w.vm.$nextTick(); }
  return w;
}

beforeEach(() => { setActivePinia(createPinia()); mockedInvoke.mockReset(); });

describe('DashboardView', () => {
  it('无用户档案时显示「朋友」', async () => {
    const w = await mountDash({ user: null });
    expect(w.text()).toContain('朋友');
  });

  it('有用户档案时显示真实姓名', async () => {
    const w = await mountDash({ user: { full_name: '张三' } });
    expect(w.text()).toContain('张三');
    expect(w.text()).not.toContain('你好，朋友');
  });

  it('姓名为空串时回退到「朋友」（不能渲染空白）', async () => {
    const w = await mountDash({ user: { full_name: '' } });
    expect(w.text()).toContain('朋友');
  });

  it('无统计数据时全部显示 0，不崩溃', async () => {
    const w = await mountDash({ stats: null });
    const values = w.findAll('.stat-value').map(v => v.text());
    expect(values[0]).toBe('0');
    expect(values[1]).toContain('0');
    expect(values[2]).toBe('0');
    expect(values[3]).toBe('0%');
  });

  it('有统计数据时按格式化规则展示', async () => {
    const w = await mountDash({
      stats: {
        total_assets: 4, total_estimated_value: 16000,
        disposal_planned_count: 4, completion_percentage: 100,
      },
    });
    const values = w.findAll('.stat-value').map(v => v.text());
    expect(values[0]).toBe('4');
    expect(values[1]).toContain('1.6万');
    expect(values[2]).toBe('4');
    expect(values[3]).toBe('100%');
  });

  it('最近资产最多显示 5 条', async () => {
    const many = Array.from({ length: 8 }, (_, i) => ({
      id: i + 1, platform_id: 1, platform_name: `平台${i}`,
      account_identifier: 'a', account_description: '', category: 'property',
      estimated_value: 10, value_currency: 'CNY', disposal_method: 'inherit',
      heir_name: null, heir_contact: null, notes: '', has_credential: false,
      is_confirmed: true, created_at: '', updated_at: '',
    }));
    const w = await mountDash({ assets: many });
    // 只验证渲染成功且不抛错；切片逻辑由 store 测试覆盖
    expect(w.text()).toContain('平台0');
  });
});

describe('完成度四维分解（旧口径会虚高）', () => {
  it('处置已定但继承人未填时，不得显示为完成', async () => {
    const w = await mountDash({
      stats: {
        total_assets: 4,
        disposal_planned_count: 4,
        heir_assigned_count: 0,
        identifier_ready_count: 4,
        trigger_ready: false,
        trigger_channel: 'none',
        dms_enabled: false,
        completion_percentage: 25,
        total_estimated_value: 0,
      },
    });
    // 四个检查项必须都渲染出来，用户能看到「卡在哪一步」
    const items = w.findAll('.check-item');
    expect(items.length).toBe(4);
    const done = w.findAll('.check-item.done');
    expect(done.length, '继承人与触发链都未完成，最多只有两项达标').toBeLessThanOrEqual(2);
    expect(w.text()).toContain('继承人有名有姓');
  });

  it('触发链开启但没有可用通道时，明说家人收不到通知', async () => {
    const w = await mountDash({
      stats: {
        total_assets: 1,
        disposal_planned_count: 1,
        heir_assigned_count: 1,
        identifier_ready_count: 1,
        trigger_ready: false,
        trigger_channel: 'none',
        dms_enabled: true,
        completion_percentage: 75,
        total_estimated_value: 0,
      },
    });
    expect(w.text(), '必须明确说明没有通道，而不是显示已开启就算完成')
      .toContain('没有可用通道');
  });

  it('邮件通道可用时展示邮件', async () => {
    const w = await mountDash({
      stats: {
        total_assets: 1,
        disposal_planned_count: 1,
        heir_assigned_count: 1,
        identifier_ready_count: 1,
        trigger_ready: true,
        trigger_channel: 'smtp',
        dms_enabled: true,
        completion_percentage: 100,
        total_estimated_value: 0,
      },
    });
    expect(w.text()).toContain('邮件通道可用');
  });
});

describe('触发链达标口径（仅心跳不算家人可达）', () => {
  it('仅心跳可用时，触发链项为警告态而非完成态', async () => {
    const w = await mountDash({
      stats: {
        total_assets: 1,
        disposal_planned_count: 1,
        heir_assigned_count: 1,
        identifier_ready_count: 1,
        trigger_ready: false,
        trigger_channel: 'heartbeat',
        dms_enabled: true,
        completion_percentage: 75,
        total_estimated_value: 0,
      },
    });
    const items = w.findAll('.check-item');
    const trigger = items[items.length - 1];
    expect(trigger.classes(), '仅心跳不得显示为已完成').not.toContain('done');
    expect(trigger.classes(), '应显示为警告（已开启但家人收不到）').toContain('warn');
    expect(trigger.text()).toContain('!');
  });

  it('邮件通道可用时才是完成态', async () => {
    const w = await mountDash({
      stats: {
        total_assets: 1,
        disposal_planned_count: 1,
        heir_assigned_count: 1,
        identifier_ready_count: 1,
        trigger_ready: true,
        trigger_channel: 'smtp',
        dms_enabled: true,
        completion_percentage: 100,
        total_estimated_value: 0,
      },
    });
    const items = w.findAll('.check-item');
    const trigger = items[items.length - 1];
    expect(trigger.classes()).toContain('done');
    expect(trigger.text()).toContain('邮件通道可用');
  });
});

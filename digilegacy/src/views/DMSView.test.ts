import { beforeEach, describe, expect, it, vi } from 'vitest';
import { flushPromises, mount } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import type { Mock } from 'vitest';

// DMSView 用 days_until_due 推导四种状态（ok/warning/overdue/disabled），
// 每种状态的图标、标题、副标题、颜色都不同；边界（<=0 / <=3）错一位，
// 用户就会在错误的时机收到「已逾期」或漏掉提醒。
vi.mock('../api/invoke', () => ({
  invoke: vi.fn(),
  isTauriEnv: () => false,
}));
vi.mock('../api/reminders', () => ({
  sendTestNotification: vi.fn(async () => true),
  startLocalReminderLoop: () => () => {},
  checkAndNotifyOnce: async () => {},
}));

import DMSView from './DMSView.vue';
import { useFeedback } from '../composables/feedback';
import { invoke } from '../api/invoke';
import { sendTestNotification } from '../api/reminders';
import { useDashboardStore } from '../stores/dashboard';
import { silenceExpectedErrors } from '../test-utils/silence';

silenceExpectedErrors();

const mockedInvoke = invoke as unknown as Mock;
const mockedTestNotify = sendTestNotification as unknown as Mock;

function checkInStatus(over: Record<string, unknown> = {}) {
  return {
    last_check_in: '2026-10-01 00:00:00',
    interval_days: 30,
    dms_enabled: true,
    next_check_in_due: '2026-10-31',
    days_until_due: 10,
    ...over,
  };
}

async function mountDms(status: Record<string, unknown> | null, opts: {
  configureFails?: boolean;
  history?: Array<Record<string, unknown>>;
} = {}) {
  mockedInvoke.mockImplementation(async (cmd: string) => {
    if (cmd === 'get_check_in_status') {
      return { success: true, message: '', data: status };
    }
    if (cmd === 'get_dms_status') {
      return { success: true, message: '', data: { enabled: true, overdue: false, current_stage: 0, days_overdue: 0 } };
    }
    if (cmd === 'get_dms_history') {
      return { success: true, message: '', data: opts.history ?? [] };
    }
    if (cmd === 'set_dead_mans_switch') {
      if (opts.configureFails) return { success: false, message: '锁定状态', data: null };
      return { success: true, message: 'ok', data: null };
    }
    return { success: true, message: '', data: null };
  });

  const pinia = createPinia();
  setActivePinia(pinia);
  const w = mount(DMSView, { global: { plugins: [pinia] } });
  for (let i = 0; i < 3; i++) { await flushPromises(); await w.vm.$nextTick(); }
  return w;
}

beforeEach(() => {
  setActivePinia(createPinia());
  mockedInvoke.mockReset();
  mockedTestNotify.mockReset();
  mockedTestNotify.mockResolvedValue(true);
  vi.stubGlobal('alert', vi.fn());
});

describe('DMS 状态推导（四态 + 边界）', () => {
  it('disabled：未启用时显示未启用状态', async () => {
    const w = await mountDms(checkInStatus({ dms_enabled: false }));
    expect(w.text()).toContain('Dead Man');
    expect(w.text()).toContain('未启用');
  });

  it('overdue：days_until_due <= 0 时判定为逾期', async () => {
    const w = await mountDms(checkInStatus({ days_until_due: 0 }));
    expect(w.text()).toContain('已超过检查时间');
    expect(w.text()).toContain('请立即确认');
  });

  it('overdue：负数天数同样判定为逾期', async () => {
    const w = await mountDms(checkInStatus({ days_until_due: -5 }));
    expect(w.text()).toContain('已超过检查时间');
  });

  it('warning：days_until_due 为 1..3 时判定为即将到期', async () => {
    for (const d of [1, 2, 3]) {
      const w = await mountDms(checkInStatus({ days_until_due: d }));
      expect(w.text(), `days=${d} 应判定为 warning`).toContain('即将需要确认');
      w.unmount();
    }
  });

  it('warning 边界：days=4 必须回到 ok（不能误报）', async () => {
    const w = await mountDms(checkInStatus({ days_until_due: 4 }));
    expect(w.text()).toContain('一切安好');
    expect(w.text()).not.toContain('即将需要确认');
  });

  it('ok：days_until_due > 3 显示剩余天数', async () => {
    const w = await mountDms(checkInStatus({ days_until_due: 20 }));
    expect(w.text()).toContain('一切安好');
    expect(w.text()).toContain('距离下次检查还有 20 天');
  });

  it('ok 但无天数：显示等待首次确认文案', async () => {
    const w = await mountDms(checkInStatus({ days_until_due: null }));
    expect(w.text()).toContain('等待首次确认后开始计算周期');
  });

  it('warning 但无天数：退化文案不得崩溃', async () => {
    const w = await mountDms(checkInStatus({ days_until_due: undefined }));
    // undefined 会走 (?? 999) 分支 → ok 状态；这里只验证不崩溃且有内容
    expect(w.text().length).toBeGreaterThan(50);
  });

  it('status 为 null 时不崩溃', async () => {
    const w = await mountDms(null);
    expect(w.text()).toContain('Dead Man');
  });
});

describe('通知记录', () => {
  it('历史投递持久可见（不是只看本轮评估结果）', async () => {
    const w = await mountDms(checkInStatus(), {
      history: [
        {
          stage: 3,
          recipient: 'heir@example.com',
          subject: '继承指引',
          body: '',
          sent_at: '2026-01-02 00:00:00',
          channel: 'local_outbox',
        },
      ],
    });
    expect(w.text()).toContain('继承指引');
    expect(w.text()).toContain('heir@example.com');
  });

  it('没有历史时不渲染空的「通知记录」卡片', async () => {
    const w = await mountDms(checkInStatus());
    expect(w.text()).not.toContain('通知记录');
  });

  it('打开页面只读状态，绝不触发投递命令', async () => {
    await mountDms(checkInStatus());
    const cmds = mockedInvoke.mock.calls.map(c => c[0]);
    expect(cmds).toContain('get_dms_status');
    expect(cmds).not.toContain('evaluate_dms');
  });
});

describe('签到', () => {
  it('点击「我还安好」调用 checkIn 并复位按钮', async () => {
    const w = await mountDms(checkInStatus());
    const btn = w.findAll('button').find(b => b.text().includes('我还安好'));
    expect(btn, '签到按钮必须存在').toBeTruthy();

    await btn!.trigger('click');
    await flushPromises();

    expect(mockedInvoke.mock.calls.some(c => c[0] === 'update_check_in')).toBe(true);
    expect(btn!.attributes('disabled')).toBeUndefined();
  });
});

describe('设置保存', () => {
  it('onMounted 回填现有配置', async () => {
    const w = await mountDms(checkInStatus({ interval_days: 14, dms_enabled: false }));
    const select = w.find('select');
    expect(select.exists()).toBe(true);
    expect((select.element as HTMLSelectElement).value).toBe('14');
  });

  it('保存成功给出提示', async () => {
    const w = await mountDms(checkInStatus({ interval_days: 30 }));
    const saveBtn = w.findAll('button').find(b => b.text() === '保存设置');
    expect(saveBtn, '保存按钮必须存在').toBeTruthy();

    await saveBtn!.trigger('click');
    await flushPromises();

    expect(mockedInvoke).toHaveBeenCalledWith('set_dead_mans_switch', expect.objectContaining({ intervalDays: 30 }));
    const { toasts } = useFeedback();
    expect(toasts.some(t => t.message === '设置已保存'), '保存成功必须有 toast 提示').toBe(true);
  });

  it('保存失败时必须提示错误（不得静默）', async () => {
    const w = await mountDms(checkInStatus(), { configureFails: true });
    const saveBtn = w.findAll('button').find(b => b.text() === '保存设置');
    expect(saveBtn).toBeTruthy();

    await saveBtn!.trigger('click');
    await flushPromises();

    const { toasts } = useFeedback();
    expect(
      toasts.some(t => t.message.includes('设置保存失败')),
      `应出现失败 toast，实际: ${JSON.stringify(toasts.map(t => t.message))}`
    ).toBe(true);
  });
});

describe('测试提醒', () => {
  it('发送成功后显示 ok 状态', async () => {
    mockedTestNotify.mockResolvedValue(true);
    const w = await mountDms(checkInStatus());

    const btn = w.findAll('button').find(b => b.text().includes('发送测试提醒'));
    expect(btn, '测试提醒按钮必须存在').toBeTruthy();

    await btn!.trigger('click');
    await flushPromises();

    expect(mockedTestNotify).toHaveBeenCalled();
    expect(btn!.attributes('disabled')).toBeUndefined();
  });

  it('发送失败后按钮仍恢复可用', async () => {
    mockedTestNotify.mockResolvedValue(false);
    const w = await mountDms(checkInStatus());

    const btn = w.findAll('button').find(b => b.text().includes('发送测试提醒'))!;
    await btn.trigger('click');
    await flushPromises();

    expect(btn.attributes('disabled')).toBeUndefined();
  });
});

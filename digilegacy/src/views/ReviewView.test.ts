import { beforeEach, describe, expect, it, vi } from 'vitest';
import { flushPromises, mount } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import type { Mock } from 'vitest';

// ReviewView 的关键契约：AI 只能把条目推进到「待人工」，
// 界面上的「通过」按钮背后必须是人工动作，且状态如实展示。
vi.mock('../api/invoke', () => ({ invoke: vi.fn(), isTauriEnv: () => false }));

import ReviewView from './ReviewView.vue';
import { invoke } from '../api/invoke';
import { silenceExpectedErrors } from '../test-utils/silence';

silenceExpectedErrors();
const mockedInvoke = invoke as unknown as Mock;

const PROFILE = {
  id: 'platform-policy',
  title_key: 'review.profile.policy',
  criteria: '检查来源与材料一致性',
  use_ai: true,
};

function state(over: Partial<Record<string, unknown>> = {}) {
  return {
    key: '1',
    title: '支付宝',
    status: 'pending',
    ai_verdict: null,
    ai_note: '',
    human_note: '',
    reviewed_at: null,
    stale: false,
    ...over,
  };
}

async function mountReview(over: Record<string, unknown> = {}) {
  mockedInvoke.mockImplementation(async (cmd: string) => {
    if (cmd in over) return over[cmd as keyof typeof over];
    if (cmd === 'list_review_profiles') return { success: true, message: '', data: [PROFILE] };
    if (cmd === 'review_payload_fields') return { success: true, message: '', data: ['平台名称', '官方链接'] };
    if (cmd === 'review_progress') return { success: true, message: '', data: { total: 1, approved: 0, awaiting_human: 0, pending: 1 } };
    if (cmd === 'list_review_items') return { success: true, message: '', data: [state()] };
    return { success: true, message: '', data: null };
  });
  const pinia = createPinia();
  setActivePinia(pinia);
  const w = mount(ReviewView, { global: { plugins: [pinia] } });
  for (let i = 0; i < 3; i++) {
    await flushPromises();
    await w.vm.$nextTick();
  }
  return w;
}

beforeEach(() => {
  setActivePinia(createPinia());
  mockedInvoke.mockReset();
});

describe('审核场景与送审字段', () => {
  it('按后端注册表渲染场景，不写死业务名', async () => {
    const w = await mountReview();
    const tabs = w.findAll('[role="tab"]');
    expect(tabs.length).toBe(1);
    expect(tabs[0].text()).toContain('平台政策');
  });

  it('展示审核进度，避免用户一条条数', async () => {
    const w = await mountReview();
    expect(w.text()).toContain('共 1 条');
    expect(w.text()).toContain('待审 1');
  });

  it('明确展示 AI 能看到哪些字段', async () => {
    const w = await mountReview();
    expect(w.text()).toContain('AI 能看到的内容');
    expect(w.text()).toContain('平台名称');
    expect(w.text()).toContain('官方链接');
  });

  it('纯人工场景要说明 AI 不参与', async () => {
    const w = await mountReview({
      list_review_profiles: {
        success: true, message: '',
        data: [{ ...PROFILE, id: 'legal', title_key: 'review.profile.policy', use_ai: false }],
      },
    });
    expect(w.text()).toContain('纯人工审核');
  });
});

describe('AI 初审与人工终审的分工', () => {
  it('AI 初审后条目显示「未发现问题」但仍为「待人工确认」', async () => {
    const w = await mountReview({
      run_ai_review: {
        success: true, message: 'AI 初审完成，1 条待人工确认',
        data: [state({ status: 'awaiting_human', ai_verdict: 'pass', ai_note: '未发现矛盾' })],
      },
    });
    await w.findAll('button').find(b => b.text().includes('运行 AI 初审'))!.trigger('click');
    await flushPromises();

    expect(w.text()).toContain('未发现矛盾');
    // 状态标签必须仍是「待人工确认」：AI 不能把条目置为已通过。
    // （不能用整页文本断言「不含已通过」——筛选按钮本身就有这个词。）
    const statusTag = w.findAll('.review-head .tag').map(t => t.text());
    expect(statusTag).toContain('待人工确认');
    expect(statusTag, 'AI 初审后不得出现已通过状态').not.toContain('已通过');
  });

  it('未配置 AI 时如实降级为纯人工，不伪造结论', async () => {
    const w = await mountReview({
      run_ai_review: {
        success: true, message: '未配置 AI 端点，已转为纯人工审核',
        data: [state({ ai_verdict: 'skipped', ai_note: '未配置 AI 端点，已转为纯人工审核' })],
      },
    });
    await w.findAll('button').find(b => b.text().includes('运行 AI 初审'))!.trigger('click');
    await flushPromises();
    expect(w.text()).toContain('未配置 AI 端点');
    expect(w.text()).toContain('未运行');
  });

  it('人工点「通过」才提交人工结论，并带备注', async () => {
    const w = await mountReview({
      submit_human_review: { success: true, message: '审核结论已记录', data: null },
    });
    await w.find('input.form-input').setValue('已核对官方公告');
    await w.findAll('button').find(b => b.text() === '通过')!.trigger('click');
    await flushPromises();

    const call = mockedInvoke.mock.calls.find(c => c[0] === 'submit_human_review');
    expect(call, '必须调用人工终审').toBeTruthy();
    const args = call![1] as Record<string, unknown>;
    expect(args.verdict).toBe('approved');
    expect(args.note).toBe('已核对官方公告');
  });

  it('拒绝与退回修改分别提交对应结论', async () => {
    const w = await mountReview({
      submit_human_review: { success: true, message: 'ok', data: null },
    });
    await w.findAll('button').find(b => b.text() === '拒绝')!.trigger('click');
    await flushPromises();
    await w.findAll('button').find(b => b.text() === '退回修改')!.trigger('click');
    await flushPromises();

    const verdicts = mockedInvoke.mock.calls
      .filter(c => c[0] === 'submit_human_review')
      .map(c => (c[1] as Record<string, unknown>).verdict);
    expect(verdicts).toEqual(['rejected', 'needs_edit']);
  });
});

describe('留痕与失效', () => {
  it('内容变化时显示「需重审」', async () => {
    const w = await mountReview({
      list_review_items: {
        success: true, message: '',
        data: [state({ status: 'pending', stale: true, ai_verdict: 'pass' })],
      },
    });
    expect(w.text()).toContain('内容已变');
  });

  it('审核历史展示操作方与结论', async () => {
    const w = await mountReview({
      review_history: {
        success: true, message: '',
        data: [{
          profile_id: 'platform-policy', item_key: '1', fingerprint: 'abc',
          ai_verdict: 'flag', ai_note: '缺来源', human_verdict: 'approved',
          human_note: '已补链接', decided_by: 'human', created_at: '2026-10-26 10:00:00',
        }],
      },
    });
    await w.findAll('button').find(b => b.text().includes('审核历史'))!.trigger('click');
    await flushPromises();
    expect(w.text()).toContain('2026-10-26 10:00:00');
    expect(w.text()).toContain('已补链接');
    expect(w.text()).toContain('human');
  });

  it('后端报错时可见，不静默', async () => {
    const w = await mountReview({
      run_ai_review: { success: false, message: '未知的审核场景', data: null },
    });
    await w.findAll('button').find(b => b.text().includes('运行 AI 初审'))!.trigger('click');
    await flushPromises();
    expect(w.text()).toContain('未知的审核场景');
  });
});

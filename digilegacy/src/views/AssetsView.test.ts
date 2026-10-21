import { beforeEach, describe, expect, it, vi } from 'vitest';
import { mount } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import { createRouter, createMemoryHistory } from 'vue-router';

// AssetsView 是用户查看全部资产的唯一入口：筛选按钮的计数、空态与行渲染
// 直接决定用户以为「自己有没有资产」。这里用真实渲染 + 真实 store 验证。
vi.mock('../api/invoke', () => ({
  invoke: vi.fn(),
  isTauriEnv: () => false,
}));

import AssetsView from './AssetsView.vue';
import { useAssetsStore } from '../stores/assets';
import { invoke } from '../api/invoke';
import type { Mock } from 'vitest';

const mockedInvoke = invoke as unknown as Mock;

function makeAsset(over: Record<string, unknown> = {}) {
  return {
    id: 1,
    platform_id: 1,
    platform_name: '支付宝',
    account_identifier: '138****0000',
    account_description: '',
    category: 'property',
    estimated_value: 12000,
    value_currency: 'CNY',
    disposal_method: 'inherit',
    heir_name: null,
    heir_contact: null,
    notes: '',
    has_credential: false,
    is_confirmed: true,
    created_at: '',
    updated_at: '',
    ...over,
  };
}

async function mountView() {
  const router = createRouter({
    history: createMemoryHistory(),
    routes: [
      { path: '/', component: { template: '<div/>' } },
      { path: '/assets', component: AssetsView },
      { path: '/assets/:id', component: { template: '<div/>' } },
      { path: '/discover', component: { template: '<div/>' } },
    ],
  });
  await router.push('/assets');
  await router.isReady();
  const pinia = createPinia();
  setActivePinia(pinia);
  const w = mount(AssetsView, { global: { plugins: [pinia, router] } });
  return w;
}

beforeEach(() => {
  setActivePinia(createPinia());
  // onMounted 会拉取列表；默认给一个成功的空响应，避免组件走异常分支污染断言
  mockedInvoke.mockReset();
  mockedInvoke.mockResolvedValue({ success: true, message: '', data: [] });
});

describe('AssetsView 渲染', () => {
  it('空态：显示引导卡片而不是空白页', async () => {
    const w = await mountView();
    expect(w.text()).toContain('还没有登记任何数字资产');
    expect(w.text()).toContain('共 0 项');
  });

  it('有数据时渲染资产行、平台名与格式化金额', async () => {
    const w = await mountView();
    const store = useAssetsStore();
    store.assets = [makeAsset()] as never;
    await w.vm.$nextTick();

    expect(w.text()).toContain('支付宝');
    expect(w.text()).toContain('¥1.2万');
    expect(w.text()).toContain('共 1 项');
    expect(w.findAll('.asset-row')).toHaveLength(1);
  });

  it('类别筛选按钮显示各自计数', async () => {
    const w = await mountView();
    const store = useAssetsStore();
    store.assets = [
      makeAsset({ id: 1, category: 'property' }),
      makeAsset({ id: 2, category: 'personality' }),
      makeAsset({ id: 3, category: 'personality' }),
    ] as never;
    await w.vm.$nextTick();

    const text = w.text();
    expect(text).toContain('💰 财产型 (1)');
    expect(text).toContain('📷 人格型 (2)');
    expect(text).toContain('🔗 混合型 (0)');
  });

  it('点击筛选后只渲染对应类别', async () => {
    const w = await mountView();
    const store = useAssetsStore();
    store.assets = [
      makeAsset({ id: 1, category: 'property', platform_name: '支付宝' }),
      makeAsset({ id: 2, category: 'personality', platform_name: '微信' }),
    ] as never;
    await w.vm.$nextTick();

    const buttons = w.findAll('button');
    await buttons[2].trigger('click');

    expect(w.findAll('.asset-row')).toHaveLength(1);
    expect(w.text()).toContain('微信');
    expect(w.text()).not.toContain('支付宝');
  });

  it('未设定处置的资产标「待设定」而非其它文案（文案漂移回归）', async () => {
    const w = await mountView();
    const store = useAssetsStore();
    store.assets = [makeAsset({ disposal_method: 'no_action' })] as never;
    await w.vm.$nextTick();

    expect(w.text()).toContain('待设定');
  });

  it('已存凭据的资产显示加密标记', async () => {
    const w = await mountView();
    const store = useAssetsStore();
    store.assets = [makeAsset({ has_credential: true })] as never;
    await w.vm.$nextTick();

    expect(w.text()).toContain('已存凭据');
  });

  it('无估值显示占位符而不是 ¥0', async () => {
    const w = await mountView();
    const store = useAssetsStore();
    store.assets = [makeAsset({ estimated_value: null })] as never;
    await w.vm.$nextTick();

    expect(w.text()).toContain('-');
  });
});

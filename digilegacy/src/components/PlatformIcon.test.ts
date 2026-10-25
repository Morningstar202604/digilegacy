import { describe, expect, it } from 'vitest';
import { mount } from '@vue/test-utils';
import PlatformIcon from './PlatformIcon.vue';

describe('PlatformIcon', () => {
  it('已知平台渲染品牌图标（支付宝）', () => {
    const w = mount(PlatformIcon, { props: { name: '支付宝' } });
    expect(w.find('svg').exists()).toBe(true);
    expect(w.attributes('role')).toBe('img');
    expect(w.attributes('aria-label')).toBe('支付宝');
  });

  it('已知分类渲染语义图标（游戏）', () => {
    const w = mount(PlatformIcon, { props: { name: '游戏' } });
    expect(w.find('svg').exists()).toBe(true);
  });

  it('未知名称回退到通用盾形图标而不是空白', () => {
    const w = mount(PlatformIcon, { props: { name: '神秘平台XYZ' } });
    expect(w.find('svg').exists()).toBe(true);
  });

  it('支持自定义尺寸', () => {
    const w = mount(PlatformIcon, { props: { name: '微信', size: 32 } });
    expect(w.attributes('style')).toContain('32px');
  });
});

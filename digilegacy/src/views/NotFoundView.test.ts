import { describe, expect, it } from 'vitest';
import { mount } from '@vue/test-utils';
import { createRouter, createMemoryHistory } from 'vue-router';

import NotFoundView from './NotFoundView.vue';

// 兜底路由必须真的可达：深层链接/旧书签落到空白页会让用户以为应用坏了。
describe('NotFoundView', () => {
  it('渲染 404 文案与返回入口', async () => {
    const router = createRouter({
      history: createMemoryHistory(),
      routes: [
        { path: '/dashboard', component: { template: '<div>dash</div>' } },
        { path: '/:pathMatch(.*)*', component: NotFoundView },
      ],
    });
    await router.push('/no-such-page');
    await router.isReady();

    const w = mount(NotFoundView, { global: { plugins: [router] } });
    expect(w.text()).toContain('这个页面不存在');
    expect(w.text()).toContain('回到仪表板');
    expect(w.find('a[href="/dashboard"]').exists()).toBe(true);
  });
});

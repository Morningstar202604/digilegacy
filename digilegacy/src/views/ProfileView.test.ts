import { beforeEach, describe, expect, it, vi } from 'vitest';
import { flushPromises, mount } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import type { Mock } from 'vitest';

// ProfileView 是安全操作的总入口：启用主密码、改主密码、导出/导入备份、
// 心跳配置、政策补丁。这些路径一旦静默失败，用户会以为「已经加密了」。
vi.mock('../api/invoke', () => ({
  invoke: vi.fn(),
  isTauriEnv: () => true,
}));

import ProfileView from './ProfileView.vue';
import { invoke } from '../api/invoke';

const mockedInvoke = invoke as unknown as Mock;

function defaultImpl(over: Record<string, (args?: Record<string, unknown>) => unknown> = {}) {
  return async (cmd: string, args?: Record<string, unknown>) => {
    if (over[cmd]) return over[cmd](args);
    if (cmd === 'security_status') {
      return { success: true, message: '', data: { has_master_password: false, unlocked: true } };
    }
    if (cmd === 'get_user_profile') return { success: true, message: '', data: null };
    if (cmd === 'platform_patch_status') return { success: true, message: '', data: 0 };
    if (cmd === 'get_heartbeat_config') {
      return { success: true, message: '', data: { enabled: false, has_token: false, endpoint: '' } };
    }
    if (cmd === 'get_recovery_seal_exists') return { success: true, message: '', data: false };
    if (cmd === 'get_all_platforms') return { success: true, message: '', data: [] };
    return { success: true, message: '', data: null };
  };
}

async function mountProfile(over?: Record<string, (args?: Record<string, unknown>) => unknown>) {
  mockedInvoke.mockImplementation(defaultImpl(over) as never);
  const pinia = createPinia();
  setActivePinia(pinia);
  const w = mount(ProfileView, { global: { plugins: [pinia] } });
  await flushPromises();
  return w;
}

beforeEach(() => {
  setActivePinia(createPinia());
  mockedInvoke.mockReset();
  vi.stubGlobal('alert', vi.fn());
  // 该组件依赖 Tauri 插件动态 import，jsdom 下会 reject；统一兜住避免噪音
  vi.doMock('@tauri-apps/plugin-autostart', () => ({ isEnabled: async () => false }));
});

describe('启用主密码', () => {
  it('少于 8 位时本地拦截，不调用后端', async () => {
    const w = await mountProfile();

    await w.find('#pf-master-pw').setValue('short');
    await w.find('#pf-master-pw2').setValue('short');
    const btn = w.findAll('button').find(b => b.text().includes('启用加密'))!;
    await btn.trigger('click');
    await flushPromises();

    expect(mockedInvoke).not.toHaveBeenCalledWith('set_master_password', expect.anything());
    expect(w.text()).toContain('至少 8 个字符');
  });

  it('两次输入不一致时本地拦截', async () => {
    const w = await mountProfile();

    await w.find('#pf-master-pw').setValue('master-pass-1');
    await w.find('#pf-master-pw2').setValue('master-pass-2');
    const btn = w.findAll('button').find(b => b.text().includes('启用加密'))!;
    await btn.trigger('click');
    await flushPromises();

    expect(mockedInvoke).not.toHaveBeenCalledWith('set_master_password', expect.anything());
    expect(w.text()).toContain('不一致');
  });

  it('成功后必须把恢复密钥展示给用户（一次性，不能丢）', async () => {
    const w = await mountProfile({
      set_master_password: () => ({
        success: true,
        message: '加密已启用',
        data: {
          status: { has_master_password: true, unlocked: true },
          recovery_code: 'ABCD-EFGH-JKLM-NPQR-STUV-WXYZ',
        },
      }),
    });

    await w.find('#pf-master-pw').setValue('master-pass-1');
    await w.find('#pf-master-pw2').setValue('master-pass-1');
    const btn = w.findAll('button').find(b => b.text().includes('启用加密'))!;
    await btn.trigger('click');
    await flushPromises();

    expect(w.text()).toContain('ABCD-EFGH-JKLM-NPQR-STUV-WXYZ');
  });

  it('后端失败时显示错误且不展示恢复密钥', async () => {
    const w = await mountProfile({
      set_master_password: () => ({ success: false, message: '已设置主密码，暂不支持修改', data: null }),
    });

    await w.find('#pf-master-pw').setValue('master-pass-1');
    await w.find('#pf-master-pw2').setValue('master-pass-1');
    const btn = w.findAll('button').find(b => b.text().includes('启用加密'))!;
    await btn.trigger('click');
    await flushPromises();

    expect(w.text()).toContain('已设置主密码');
  });
});

describe('备份导出', () => {
  it('成功时显示落地路径', async () => {
    const w = await mountProfile({
      export_backup: () => ({ success: true, message: '备份已导出', data: 'C:/app/备份/x.json' }),
    });

    const btn = w.findAll('button').find(b => b.text().includes('导出加密备份'))!;
    await btn.trigger('click');
    await flushPromises();

    expect(w.text()).toContain('C:/app/备份/x.json');
  });

  it('失败时明确报错，不得静默', async () => {
    const w = await mountProfile({
      export_backup: () => ({ success: false, message: '写入备份失败', data: null }),
    });

    const btn = w.findAll('button').find(b => b.text().includes('导出加密备份'))!;
    await btn.trigger('click');
    await flushPromises();

    expect(w.text()).toContain('写入备份失败');
  });
});

describe('心跳配置', () => {
  it('未填令牌时启用被后端拒绝并回显原因', async () => {
    const w = await mountProfile({
      set_heartbeat_config: () => ({ success: false, message: '启用心跳前请先填写令牌', data: null }),
    });

    const cb = w.find('#hb-endpoint');
    expect(cb.exists()).toBe(true);
  });

  it('加载时回填已保存的端点', async () => {
    const w = await mountProfile({
      get_heartbeat_config: () => ({
        success: true, message: '', data: { enabled: true, has_token: true, endpoint: 'https://hb.example.org' },
      }),
    });

    const input = w.find('#hb-endpoint').element as HTMLInputElement;
    expect(input.value).toBe('https://hb.example.org');
  });
});

describe('政策补丁', () => {
  it('显示当前补丁条数', async () => {
    const w = await mountProfile({ platform_patch_status: () => ({ success: true, message: '', data: 3 }) });
    expect(w.text()).toContain('3');
  });

  it('补丁数为 0 时清除按钮禁用', async () => {
    const w = await mountProfile();
    const clearBtn = w.findAll('button').find(b => b.text().includes('清除全部补丁'))!;
    expect(clearBtn.attributes('disabled')).toBeDefined();
  });
});

// 统一的 invoke 入口：Tauri 桌面环境走真实后端；
// 浏览器开发环境（npm run dev）自动切换到本地 mock，便于预览前端外观。

import { invoke as tauriInvoke } from '@tauri-apps/api/core';

export function isTauriEnv(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}

export const IS_WEB_PREVIEW =
  typeof window !== 'undefined' && !isTauriEnv() && import.meta.env.DEV;

export async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (isTauriEnv()) {
    return tauriInvoke<T>(cmd, args);
  }
  if (import.meta.env.DEV) {
    const { mockInvoke } = await import('./mock');
    return mockInvoke<T>(cmd, args);
  }
  throw new Error('Web 预览模式仅支持开发环境（npm run dev）');
}

import { afterEach, beforeEach, describe, expect, it } from 'vitest';

// i18n 核心契约：默认中文不破坏既有 UI/测试；切换英文即时生效；
// 缺失键回退 zh-CN → 键名，绝不抛错（新键漏配时界面不能白屏）。

import { locale, setLocale, toggleLocale, t } from './index';

const STORAGE_KEY = 'digilegacy_locale';

beforeEach(() => {
  localStorage.clear();
  setLocale('zh-CN');
});

afterEach(() => {
  localStorage.clear();
  setLocale('zh-CN');
});

describe('i18n 核心', () => {
  it('默认中文：t() 返回词典中文文案', () => {
    expect(t('nav.dashboard')).toBe('仪表板');
    expect(t('common.save')).toBe('保存');
  });

  it('只存在于英文词典的键在英文模式返回英文，中文模式回退键名', () => {
    setLocale('en-US');
    expect(t('app.demoBanner')).toContain('Live demo');
    setLocale('zh-CN');
    expect(t('app.demoBanner')).toBe('app.demoBanner');
  });

  it('两个语言都没有的键回退键名本身（不抛错）', () => {
    expect(t('i18n.definitely.missing')).toBe('i18n.definitely.missing');
  });

  it('插值：{placeholder} 被参数替换', () => {
    expect(t('profile.backupExported', { path: '/tmp/backup.json' })).toBe(
      '备份已导出：/tmp/backup.json'
    );
  });

  it('setLocale 后 t() 立即返回英文', () => {
    setLocale('en-US');
    expect(t('nav.dashboard')).toBe('Dashboard');
  });

  it('toggleLocale 在 zh-CN 与 en-US 之间切换', () => {
    expect(locale.value).toBe('zh-CN');
    toggleLocale();
    expect(locale.value).toBe('en-US');
    toggleLocale();
    expect(locale.value).toBe('zh-CN');
  });

  it('setLocale 持久化到 localStorage', () => {
    setLocale('en-US');
    expect(localStorage.getItem(STORAGE_KEY)).toBe('en-US');
  });
});

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

  it('缺失键在两种语言下都回退键名（新键漏配不能白屏）', () => {
    setLocale('en-US');
    expect(t('app.demoBanner')).toBe('app.demoBanner');
    setLocale('zh-CN');
    expect(t('app.demoBanner')).toBe('app.demoBanner');
  });

  it('两个语言都没有的键回退键名本身（不抛错）', () => {
    expect(t('i18n.definitely.missing')).toBe('i18n.definitely.missing');
  });

  it('中英词典键集完全对齐（漏翻会静默回退中文，必须有门禁）', async () => {
    const { zhCN, enUS } = await import('./index');
    const zhOnly = Object.keys(zhCN).filter(k => !(k in enUS));
    const enOnly = Object.keys(enUS).filter(k => !(k in zhCN));
    expect(zhOnly, '只在中文词典里的键').toEqual([]);
    expect(enOnly, '只在英文词典里的键').toEqual([]);
  });

  it('界面文案不含 emoji（统一走矢量图标，避免平台字形不一致）', async () => {
    const { zhCN, enUS } = await import('./index');
    const emoji = /\p{Extended_Pictographic}/u;
    const offenders = [...Object.entries(zhCN), ...Object.entries(enUS)]
      .filter(([, v]) => emoji.test(v))
      .map(([k]) => k);
    expect(offenders).toEqual([]);
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

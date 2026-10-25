// 跨视图共享的格式化与文案映射（单一事实来源）
//
// 提取自 DashboardView / AssetsView / ReportView / AssetDetailView 的重复实现。
// 文案与规则只在此处维护，避免出现「待设定 / 待定」之类的不一致漂移
// （曾因三视图各持一份副本导致同一枚举值显示两种文案的实际缺陷）。

import { t } from '../i18n';

/** 金额格式化：≥1万显示「x.x万 / x.xk」，否则千分位；空值/0 显示「0」 */
export function formatMoney(n: number): string {
  if (!n) return '0';
  if (n >= 10000) {
    return t('common.tenThousand', { value: (n / 10000).toFixed(1) });
  }
  return n.toLocaleString('zh-CN');
}

/** 资产类别文案 */
export function categoryLabel(cat: string): string {
  const map: Record<string, string> = {
    property: 'cat.property',
    personality: 'cat.personality',
    mixed: 'cat.mixed',
  };
  return t(map[cat] || cat);
}

/** 处置意愿文案（no_action 统一为「待设定」，与资产详情下拉选项「暂不设定」语义对应） */
export function disposalLabel(method: string): string {
  const map: Record<string, string> = {
    inherit: 'disposalLabel.inherit',
    liquidate: 'disposalLabel.liquidate',
    memorialize: 'disposalLabel.memorialize',
    authorize_view: 'disposalLabel.authorize_view',
    delete: 'disposalLabel.delete',
    donate: 'disposalLabel.donate',
    no_action: 'disposalLabel.no_action',
  };
  return t(map[method] || method);
}

// 平台名 → 图标：资产列表与资产详情页共用，新增平台在此补一行
const PLATFORM_ICONS: Record<string, string> = {
  支付宝: '💳',
  微信: '💬',
  微博: '📰',
  淘宝: '🛒',
  王者荣耀: '🎮',
  原神: '⚔️',
  同花顺: '📈',
};

/** 平台图标（按平台名；未知平台回退通用图标） */
export function platformIcon(name: string): string {
  return PLATFORM_ICONS[name] || '📱';
}

// 数据分类 → 图标（与 platform_data.rs 的 16 个数据分类一致，DiscoverView 步骤3使用）
const CATEGORY_ICONS: Record<string, string> = {
  支付金融: '💳',
  社交通讯: '💬',
  游戏: '🎮',
  电商: '🛒',
  金融证券: '📈',
  知识付费: '📚',
  工具订阅: '🔧',
  出行住宿: '✈️',
  生活工具: '🏠',
  内容创作: '✍️',
  存储云服务: '☁️',
  政务服务: '🏛️',
  音视频: '🎬',
  学习与考试: '🎓',
  汽车服务: '🚗',
  生活服务: '🍜',
};

/** 数据分类图标（按数据分类名；未知分类回退通用图标） */
export function categoryIcon(cat: string): string {
  return CATEGORY_ICONS[cat] || '📱';
}

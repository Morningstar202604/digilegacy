/** DMS 检查间隔的单一事实来源：两个页面都从这里取，避免选项集合漂移。 */
export interface IntervalOption {
  value: number;
  label: string;
  short: string;
}

export const INTERVAL_OPTIONS: IntervalOption[] = [
  { value: 7, label: '每周确认一次', short: '每周' },
  { value: 14, label: '每两周确认一次', short: '每两周' },
  { value: 30, label: '每月确认一次', short: '每月' },
  { value: 60, label: '每两月确认一次', short: '每两月' },
  { value: 90, label: '每季度确认一次', short: '每季度' },
];

export function intervalLabel(days: number, short = false): string {
  const hit = INTERVAL_OPTIONS.find(o => o.value === days);
  if (hit) return short ? hit.short : hit.label;
  return `每 ${days} 天`;
}

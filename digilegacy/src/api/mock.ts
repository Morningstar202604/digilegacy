// Web 开发预览专用 mock：仅在 npm run dev 的浏览器环境生效
// 数据存 localStorage（明文），只用于查看前端界面外观，不得用于真实数据

import type {
  ApiResponse, CheckInStatus, DashboardStats, DigitalAsset, Platform, PlatformExtended, SecurityStatus, UserProfile,
} from '../types';
import platformsSeed from './mock-platforms.json';

const LS_KEY = 'digilegacy-web-preview-v1';
const SS_UNLOCK = 'digilegacy-web-preview-unlocked';

interface MockDms {
  enabled: boolean;
  interval: number;
  lastCheckIn: string | null;
}

interface MockState {
  assets: DigitalAsset[];
  profile: UserProfile | null;
  checkIns: string[];
  dms: MockDms;
  masterPassword: string | null;
  credentials: Record<string, string>;
  nextAssetId: number;
}

const seedPlatforms = platformsSeed as unknown as Array<Record<string, unknown>>;

function platformById(id: number): Record<string, unknown> | undefined {
  return seedPlatforms.find(p => p.id === id);
}

function platformIdByName(keyword: string): number {
  const hit = seedPlatforms.find(p => String(p.name).includes(keyword));
  return hit ? Number(hit.id) : Number(seedPlatforms[0].id);
}

function now(): string {
  const d = new Date();
  const pad = (n: number) => String(n).padStart(2, '0');
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`;
}

function defaultState(): MockState {
  const demo = (
    pid: number, ident: string, cat: string, value: number, method: string, heir?: string, notes = ''
  ): DigitalAsset => ({
    id: undefined as unknown as number,
    platform_id: pid,
    platform_name: String(platformById(pid)?.name || ''),
    account_identifier: ident,
    account_description: '',
    category: cat as DigitalAsset['category'],
    estimated_value: value,
    value_currency: 'CNY',
    disposal_method: method,
    heir_name: heir || null,
    heir_contact: null,
    notes,
    has_credential: false,
    is_confirmed: true,
    created_at: now(),
    updated_at: now(),
  });
  const assets = [
    demo(platformIdByName('支付宝'), '138****0000（演示）', 'property', 12000, 'inherit', '演示继承人'),
    demo(platformIdByName('B站') , 'demo@example.com', 'mixed', 3000, 'memorialize'),
    demo(platformIdByName('微信'), '138****0000', 'personality', 500, 'delete', undefined, '聊天记录选择彻底删除'),
    demo(platformIdByName('Steam'), 'demo_user', 'property', 800, 'liquidate'),
  ];
  let id = 1;
  for (const a of assets) {
    a.id = id++;
  }
  return {
    assets,
    profile: {
      id: 1,
      full_name: '演示用户',
      phone: '138****0000',
      email: 'demo@example.com',
      id_last4: '',
      emergency_contact_name: '演示联系人',
      emergency_contact_phone: '139****0000',
      emergency_contact_relation: '子女',
      emergency_contact_email: 'contact@example.com',
      heir_primary_name: '演示继承人',
      heir_primary_phone: '137****0000',
      heir_primary_relation: '子女',
      heir_primary_email: 'heir@example.com',
      check_in_interval_days: 30,
      last_check_in: null,
      dms_enabled: true,
      created_at: now(),
      updated_at: now(),
    },
    checkIns: [],
    dms: { enabled: true, interval: 30, lastCheckIn: null },
    masterPassword: null,
    credentials: {},
    nextAssetId: id,
  };
}

function loadState(): MockState {
  try {
    const raw = localStorage.getItem(LS_KEY);
    if (raw) return JSON.parse(raw) as MockState;
  } catch {
    // 忽略损坏的预览数据
  }
  const fresh = defaultState();
  saveState(fresh);
  return fresh;
}

function saveState(state: MockState) {
  localStorage.setItem(LS_KEY, JSON.stringify(state));
}

function ok<T>(data: T, message = '操作成功'): ApiResponse<T> {
  return { success: true, message, data };
}

function fail<T>(message: string): ApiResponse<T> {
  return { success: false, message, data: null };
}

function isUnlocked(): boolean {
  return sessionStorage.getItem(SS_UNLOCK) === '1';
}

function toPlatform(entry: Record<string, unknown>): Platform {
  return {
    id: Number(entry.id),
    name: String(entry.name),
    category: String(entry.category),
    subtype: String(entry.subtype ?? ''),
    has_property: Boolean(entry.has_property),
    has_personality: Boolean(entry.has_personality),
    inheritable: Boolean(entry.inheritable),
    required_documents: String(entry.required_documents ?? ''),
    process_description: String(entry.process_description ?? ''),
    estimated_difficulty: String(entry.estimated_difficulty ?? ''),
    official_policy_url: String(entry.official_policy_url ?? ''),
    estimated_time_days: String(entry.estimated_time_days ?? ''),
    notes: String(entry.notes ?? ''),
  };
}

function withPlatformName(a: DigitalAsset): DigitalAsset {
  return { ...a, platform_name: String(platformById(a.platform_id)?.name || a.platform_name || '') };
}

function download(filename: string, blob: Blob) {
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = filename;
  a.click();
  URL.revokeObjectURL(url);
}

export async function mockInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  const state = loadState();
  const a = (args || {}) as Record<string, never>;
  const req = (args?.req || {}) as Record<string, unknown>;

  switch (cmd) {
    case 'get_all_platforms':
      return ok(seedPlatforms.map(toPlatform)) as T;

    case 'get_all_platforms_extended':
      return ok(seedPlatforms.map(p => ({ ...toPlatform(p), name_en: String(p.name_en ?? ''), data_source: String(p.data_source ?? ''), inheritance_form_url: String(p.inheritance_form_url ?? ''), monetization_path: String(p.monetization_path ?? ''), legal_template_ids: String(p.legal_template_ids ?? ''), risk_warnings: String(p.risk_warnings ?? ''), tags: String(p.tags ?? '') })) as unknown as PlatformExtended[]) as T;

    case 'get_all_assets':
      return ok(state.assets.map(withPlatformName)) as T;

    case 'get_asset_by_id': {
      const hit = state.assets.find(x => x.id === Number(a['id']));
      return ok(hit ? withPlatformName(hit) : null) as T;
    }

    case 'get_assets_by_category':
      return ok(state.assets.filter(x => x.category === a['category']).map(withPlatformName)) as T;

    case 'add_asset': {
      const asset: DigitalAsset = {
        id: state.nextAssetId++,
        platform_id: Number(req.platform_id),
        platform_name: String(platformById(Number(req.platform_id))?.name || ''),
        account_identifier: String(req.account_identifier ?? ''),
        account_description: String(req.account_description ?? ''),
        category: String(req.category) as DigitalAsset['category'],
        estimated_value: (req.estimated_value as number | null) ?? null,
        value_currency: 'CNY',
        disposal_method: String(req.disposal_method ?? 'no_action'),
        heir_name: (req.heir_name as string) || null,
        heir_contact: (req.heir_contact as string) || null,
        notes: String(req.notes ?? ''),
        has_credential: false,
        is_confirmed: true,
        created_at: now(),
        updated_at: now(),
      };
      state.assets.unshift(asset);
      saveState(state);
      return ok(asset.id, '数字资产添加成功') as T;
    }

    case 'update_asset': {
      const target = state.assets.find(x => x.id === Number(a['id']));
      if (!target) return fail<null>('资产不存在或已被删除') as T;
      Object.assign(target, {
        platform_id: Number(req.platform_id),
        account_identifier: String(req.account_identifier ?? ''),
        account_description: String(req.account_description ?? ''),
        category: String(req.category),
        estimated_value: (req.estimated_value as number | null) ?? null,
        disposal_method: String(req.disposal_method ?? 'no_action'),
        heir_name: (req.heir_name as string) || null,
        heir_contact: (req.heir_contact as string) || null,
        notes: String(req.notes ?? ''),
        updated_at: now(),
      });
      saveState(state);
      return ok(null, '资产更新成功') as T;
    }

    case 'delete_asset': {
      state.assets = state.assets.filter(x => x.id !== Number(a['id']));
      delete state.credentials[String(a['id'])];
      saveState(state);
      return ok(null, '资产已删除') as T;
    }

    case 'set_disposal_wish': {
      const target = state.assets.find(x => x.id === Number(a['asset_id']));
      if (!target) return fail<null>('资产不存在') as T;
      target.disposal_method = String(a['disposal_method']);
      target.heir_name = (a['heir_name'] as string) || null;
      target.heir_contact = (a['heir_contact'] as string) || null;
      target.notes = String(a['notes'] ?? '');
      saveState(state);
      return ok(null, '处置意愿已设定') as T;
    }

    case 'get_user_profile':
      return ok(state.profile) as T;

    case 'set_user_profile': {
      const profile = args as unknown as UserProfile;
      state.profile = { ...profile, updated_at: now() };
      saveState(state);
      return ok(null, '用户档案已保存') as T;
    }

    case 'get_inheritance_report': {
      const assets = state.assets.map(withPlatformName);
      const total = assets.reduce((sum, x) => sum + (x.estimated_value || 0), 0);
      return ok({
        user: state.profile || { full_name: '未设置' },
        assets,
        total_estimated_value: total,
        property_assets_count: assets.filter(x => x.category === 'property').length,
        personality_assets_count: assets.filter(x => x.category === 'personality').length,
        mixed_assets_count: assets.filter(x => x.category === 'mixed').length,
        generated_at: now(),
      }) as T;
    }

    case 'get_dashboard_stats': {
      const assets = state.assets;
      const planned = assets.filter(x => x.disposal_method !== 'no_action').length;
      const stats: DashboardStats = {
        total_assets: assets.length,
        property_count: assets.filter(x => x.category === 'property').length,
        personality_count: assets.filter(x => x.category === 'personality').length,
        mixed_count: assets.filter(x => x.category === 'mixed').length,
        confirmed_count: assets.filter(x => x.is_confirmed).length,
        unconfirmed_count: assets.filter(x => !x.is_confirmed).length,
        total_estimated_value: assets.reduce((sum, x) => sum + (x.estimated_value || 0), 0),
        disposal_planned_count: planned,
        last_check_in: state.dms.lastCheckIn,
        dms_enabled: state.dms.enabled,
        completion_percentage: assets.length > 0 ? Math.round((planned * 100) / assets.length) : 0,
      };
      return ok(stats) as T;
    }

    case 'update_check_in': {
      const ts = now();
      state.checkIns.push(ts);
      state.dms.lastCheckIn = ts;
      if (state.profile) state.profile.last_check_in = ts;
      saveState(state);
      return ok(ts, '检查完成，状态良好') as T;
    }

    case 'get_check_in_status': {
      const interval = state.dms.interval;
      let nextDue: string | null = null;
      let daysUntil: number | null = null;
      if (state.dms.lastCheckIn) {
        const last = new Date(state.dms.lastCheckIn.replace(' ', 'T'));
        const due = new Date(last.getTime() + interval * 86400000);
        nextDue = due.toISOString().slice(0, 10);
        daysUntil = Math.ceil((due.getTime() - Date.now()) / 86400000);
      }
      const status: CheckInStatus = {
        last_check_in: state.dms.lastCheckIn,
        interval_days: interval,
        dms_enabled: state.dms.enabled,
        next_check_in_due: nextDue,
        days_until_due: daysUntil,
      };
      return ok(status) as T;
    }

    case 'set_dead_mans_switch':
      state.dms.enabled = Boolean(a['enabled']);
      state.dms.interval = Number(a['intervalDays'] ?? a['interval_days'] ?? 30);
      saveState(state);
      return ok(null, 'Dead Man\'s Switch 设置已保存') as T;

    case 'evaluate_dms':
      return ok({ enabled: state.dms.enabled, overdue: false, current_stage: 0, days_overdue: 0, notifications: [] }) as T;

    case 'export_report_to_file': {
      download('继承报告.html', new Blob([String(a['content'] ?? '')], { type: 'text/html;charset=utf-8' }));
      return ok('（浏览器预览）已下载 继承报告.html', '报告已导出') as T;
    }

    case 'export_zip_to_file': {
      const b64 = String(a['content'] ?? '');
      const bin = atob(b64);
      const bytes = new Uint8Array(bin.length);
      for (let i = 0; i < bin.length; i++) bytes[i] = bin.charCodeAt(i);
      download('继承工具包.zip', new Blob([bytes], { type: 'application/zip' }));
      return ok('（浏览器预览）已下载 继承工具包.zip', '继承工具包已导出') as T;
    }

    case 'security_status': {
      const status: SecurityStatus = { has_master_password: state.masterPassword !== null, unlocked: state.masterPassword === null ? true : isUnlocked() };
      return ok(status) as T;
    }

    case 'set_master_password': {
      const password = String(a['password'] ?? '');
      if (password.length < 8) return fail<null>('主密码至少需要 8 个字符') as T;
      if (state.masterPassword !== null) return fail<null>('已设置主密码，暂不支持修改') as T;
      state.masterPassword = password;
      saveState(state);
      sessionStorage.setItem(SS_UNLOCK, '1');
      // 与桌面端一致：恢复密钥放在 data 字段返回（预览模式为演示数据）
      return ok({ status: { has_master_password: true, unlocked: true }, recovery_code: 'ABCD-EFGH-JKLM-NPQR-STUV-WXYZ' }, '加密已启用（预览模式为演示实现）') as T;
    }

    case 'unlock': {
      if (state.masterPassword === null) return fail<null>('尚未设置主密码') as T;
      if (String(a['password'] ?? '') !== state.masterPassword) return fail<null>('主密码错误') as T;
      sessionStorage.setItem(SS_UNLOCK, '1');
      return ok({ has_master_password: true, unlocked: true }) as T;
    }

    case 'lock':
      sessionStorage.removeItem(SS_UNLOCK);
      return ok({ has_master_password: state.masterPassword !== null, unlocked: false }, '已锁定') as T;

    case 'change_master_password': {
      const oldPw = String(a['oldPassword'] ?? a['old_password'] ?? '');
      const newPw = String(a['newPassword'] ?? a['new_password'] ?? '');
      const confirmPw = String(a['confirmPassword'] ?? a['confirm_password'] ?? '');
      if (newPw.length < 8) return fail<null>('新主密码至少需要 8 个字符') as T;
      if (newPw !== confirmPw) return fail<null>('两次输入的新主密码不一致') as T;
      if (state.masterPassword === null) return fail<null>('尚未设置主密码') as T;
      if (oldPw !== state.masterPassword) return fail<null>('当前主密码错误') as T;
      state.masterPassword = newPw;
      saveState(state);
      return ok({ has_master_password: true, unlocked: true }, '主密码已更新（预览模式）') as T;
    }

    case 'set_asset_credential': {
      const id = String(a['id']);
      const credential = String(a['credential'] ?? '');
      if (credential && state.masterPassword !== null && !isUnlocked()) return fail<null>('请先解锁后再保存凭据') as T;
      if (credential) {
        state.credentials[id] = credential;
      } else {
        delete state.credentials[id];
      }
      const target = state.assets.find(x => x.id === Number(a['id']));
      if (target) target.has_credential = Boolean(credential);
      saveState(state);
      return ok(null, credential ? '凭据已加密保存（预览模式）' : '凭据已清除') as T;
    }

    case 'get_asset_credential': {
      if (state.masterPassword !== null && !isUnlocked()) return fail<null>('数据已加密，请先解锁') as T;
      return ok(state.credentials[String(a['id'])] ?? '') as T;
    }

    case 'export_backup': {
      const backup = {
        app: 'digilegacy',
        version: 1,
        created_at: now(),
        tables: {
          digital_assets: state.assets,
          user_profile: state.profile ? [state.profile] : [],
          check_in_log: state.checkIns.map((t, i) => ({ id: i + 1, checked_at: t, status: 'ok', note: '' })),
          dms_config: state.dms,
          security_config: state.masterPassword ? { kdf_salt: 'preview', verifier: 'preview' } : null,
        },
      };
      download(`归零加密备份_预览_${Date.now()}.json`, new Blob([JSON.stringify(backup, null, 2)], { type: 'application/json' }));
      return ok('（浏览器预览）已下载备份 JSON', '备份已导出') as T;
    }

    case 'import_backup': {
      try {
        const parsed = JSON.parse(String(a['content'] ?? ''));
        if (parsed.app !== 'digilegacy' || parsed.version !== 1) return fail<null>('不是有效的归零备份文件') as T;
        const tables = parsed.tables || {};
        state.assets = (tables.digital_assets || []).map((x: DigitalAsset) => withPlatformName(x));
        state.profile = (tables.user_profile || [])[0] || null;
        state.checkIns = (tables.check_in_log || []).map((x: { checked_at: string }) => x.checked_at);
        state.dms = { enabled: Boolean(tables.dms_config?.enabled), interval: Number(tables.dms_config?.check_in_interval_days ?? 30), lastCheckIn: tables.dms_config?.last_check_in ?? null };
        state.nextAssetId = state.assets.reduce((m, x) => Math.max(m, x.id || 0), 0) + 1;
        saveState(state);
        sessionStorage.removeItem(SS_UNLOCK);
        if (tables.security_config) state.masterPassword = state.masterPassword ?? 'preview';
        return ok(`已恢复 ${state.assets.length} 条资产（预览模式）`, '备份导入完成') as T;
      } catch (e) {
        return fail<null>('备份文件解析失败：' + String(e)) as T;
      }
    }

    case 'open_external': {
      const url = String(a['url'] ?? '');
      if (!/^https?:\/\//i.test(url)) return fail<null>('仅允许打开 http/https 链接') as T;
      window.open(url, '_blank', 'noopener');
      return ok(null, '已在浏览器打开') as T;
    }

    case 'get_recovery_seal_exists':
      return ok(state.masterPassword !== null) as T;

    case 'start_recovery': {
      const code = String(a['recoveryCode'] ?? '').replace(/[\s-]/g, '').toUpperCase();
      if (state.masterPassword === null) return fail<null>('该安装未启用恢复密钥') as T;
      sessionStorage.setItem('digilegacy-web-preview-recovery', code);
      return ok({ has_master_password: true, unlocked: true }) as T;
    }

    case 'complete_recovery': {
      const newPw = String(a['newPassword'] ?? '');
      const confirmPw = String(a['confirmPassword'] ?? '');
      if (newPw.length < 8) return fail<null>('新主密码至少需要 8 个字符') as T;
      if (newPw !== confirmPw) return fail<null>('两次输入的新主密码不一致') as T;
      const expected = sessionStorage.getItem('digilegacy-web-preview-recovery');
      if (!expected) return fail<null>('请先通过恢复密钥解锁') as T;
      state.masterPassword = newPw;
      saveState(state);
      sessionStorage.removeItem('digilegacy-web-preview-recovery');
      return ok('XXXX-XXXX-XXXX-XXXX-XXXX-DEMO', '主密码已重置，请立即保存新的恢复密钥') as T;
    }

    case 'get_heartbeat_status':
      return ok({ enabled: false, endpoint: '（预览模式未配置）', last_attempt: null, last_result: 'never' }) as T;

    case 'send_heartbeat_now':
      return ok({ enabled: false, endpoint: '（预览模式未配置）', last_attempt: new Date().toLocaleString(), last_result: 'never' }) as T;

    case 'set_autostart':
      return ok(null, a['enabled'] ? '已设置开机自启' : '已关闭开机自启') as T;

    case 'hide_main_window':
      return ok(null, '已隐藏到托盘') as T;

    default:
      throw new Error(`Web 预览模式未实现命令: ${cmd}`);
  }
}

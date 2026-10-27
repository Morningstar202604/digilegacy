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
  custody?: { backup_location: string; recovery_code_location: string };
  heirs?: Array<{
    id?: number; name: string; relation: string; phone: string; email: string;
    share_note: string; is_executor: boolean; sort_order: number;
  }>;
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
    custody: { backup_location: '', recovery_code_location: '' },
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
      // 与 Rust 命令层同一边界：非法类别/处置与非有限或负金额必须拒绝，
      // 否则演示环境的行为会与桌面版不一致（用户会以为桌面版也接受）。
      const VALID_CATEGORIES = ['property', 'personality', 'mixed'];
      const VALID_DISPOSALS = ['inherit', 'liquidate', 'memorialize', 'authorize_view', 'delete', 'donate', 'no_action'];
      const inputCategory = String(req.category ?? '');
      const inputDisposal = String(req.disposal_method ?? 'no_action');
      if (!VALID_CATEGORIES.includes(inputCategory)) return fail<null>('无效的资产类别') as T;
      if (!VALID_DISPOSALS.includes(inputDisposal)) return fail<null>('无效的处置方式') as T;
      const inputValue = req.estimated_value as number | null | undefined;
      if (inputValue != null && (!Number.isFinite(inputValue) || inputValue < 0 || inputValue > 1e12)) {
        return fail<null>('预估价值必须是 0 到 1 万亿之间的有限数值') as T;
      }
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
      // 与桌面版同契约：政策随报告一起下发，预览与导出读同一份。
      const policyIds = Array.from(new Set(assets.map(x => x.platform_id)));
      const policies = policyIds.map(pid => {
        const p = platformById(pid);
        return {
          platform_id: pid,
          name: String(p?.name ?? ''),
          required_documents: String(p?.required_documents ?? ''),
          process_description: String(p?.process_description ?? ''),
          estimated_difficulty: String(p?.estimated_difficulty ?? ''),
          estimated_time_days: String(p?.estimated_time_days ?? ''),
          official_policy_url: String(p?.official_policy_url ?? ''),
          monetization_path: String(p?.monetization_path ?? ''),
          risk_warnings: String(p?.risk_warnings ?? ''),
          tags: String(p?.tags ?? ''),
          has_source: false,
        };
      });
      return ok({
        user: state.profile || { full_name: '未设置' },
        assets,
        policies,
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
      // 与桌面版同一口径：继承必须有继承人，占位符不算已填。
      const heirAssigned = assets.filter(
        x => x.disposal_method !== 'inherit' || Boolean(x.heir_name && x.heir_name.trim()),
      ).length;
      const identifierReady = assets.filter(
        x => x.account_identifier && x.account_identifier.trim() && !x.account_identifier.includes('需解锁'),
      ).length;
      const triggerReady = state.dms.enabled;
      const ratio = (n: number) => (assets.length > 0 ? Math.round((n * 100) / assets.length) : 0);
      const completion = assets.length > 0
        ? Math.round((ratio(planned) + ratio(heirAssigned) + ratio(identifierReady) + (triggerReady ? 100 : 0)) / 4)
        : 0;
      const stats: DashboardStats = {
        total_assets: assets.length,
        property_count: assets.filter(x => x.category === 'property').length,
        personality_count: assets.filter(x => x.category === 'personality').length,
        mixed_count: assets.filter(x => x.category === 'mixed').length,
        disposal_planned_count: planned,
        heir_assigned_count: heirAssigned,
        identifier_ready_count: identifierReady,
        trigger_ready: triggerReady,
        trigger_channel: triggerReady ? 'heartbeat' : 'none',
        total_estimated_value: assets.reduce((sum, x) => sum + (x.estimated_value || 0), 0),
        last_check_in: state.dms.lastCheckIn,
        dms_enabled: state.dms.enabled,
        completion_percentage: completion,
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
        recent_check_ins: [...state.checkIns].reverse().slice(0, 5),
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

    case 'get_dms_status':
      return ok({ enabled: state.dms.enabled, overdue: false, current_stage: 0, days_overdue: 0 }) as T;

    case 'get_dms_history':
      return ok([]) as T;

    case 'render_report': {
      // 演示环境用同一份结构的最小 HTML，保持与桌面版同一命令契约
      const r = (await mockInvoke('get_inheritance_report')) as { data?: { assets?: unknown[] } };
      const count = r.data?.assets?.length ?? 0;
      return ok(
        `<!DOCTYPE html><html lang="zh-CN"><head><meta charset="UTF-8"><title>数字遗产继承报告</title>` +
        `<style>@media print{body{padding:0}}</style></head><body>` +
        `<div class="doc-title">数字遗产继承报告</div><p>演示报告：共 ${count} 条资产。</p></body></html>`,
        '报告已生成',
      ) as T;
    }

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
        version: 3,
        created_at: now(),
        tables: {
          digital_assets: state.assets,
          user_profile: state.profile ? [state.profile] : [],
          check_in_log: state.checkIns.map((t, i) => ({ id: i + 1, checked_at: t, status: 'ok', note: '' })),
          dms_config: state.dms,
          security_config: state.masterPassword ? { kdf_salt: 'preview', verifier: 'preview' } : null,
          heartbeat_config: null,
          platform_overrides: [],
        },
      };
      download(`归零加密备份_预览_${Date.now()}.json`, new Blob([JSON.stringify(backup, null, 2)], { type: 'application/json' }));
      return ok('（浏览器预览）已下载备份 JSON', '备份已导出') as T;
    }

    case 'import_backup': {
      try {
        const parsed = JSON.parse(String(a['content'] ?? ''));
        if (parsed.app !== 'digilegacy' || ![1, 2, 3].includes(parsed.version)) return fail<null>('不是有效的归零备份文件') as T;
        const tables = parsed.tables || {};
        const incoming = (tables.digital_assets || []).map((x: DigitalAsset) => withPlatformName(x));
        // 与桌面版同一语义：merge 追加、replace 清空。演示环境也必须一致，
        // 否则用户会在演示里以为「导入=覆盖」而不敢用，或反之误覆盖真实数据。
        const mode = String(a['mode'] ?? 'merge');
        if (mode === 'replace') {
          state.assets = incoming;
          state.profile = (tables.user_profile || [])[0] || null;
          state.checkIns = (tables.check_in_log || []).map((x: { checked_at: string }) => x.checked_at);
        } else {
          let nextId = state.assets.reduce((m, x) => Math.max(m, x.id || 0), 0) + 1;
          for (const item of incoming) {
            state.assets.push({ ...item, id: nextId++ });
          }
        }
        state.dms = { enabled: Boolean(tables.dms_config?.enabled), interval: Number(tables.dms_config?.check_in_interval_days ?? 30), lastCheckIn: tables.dms_config?.last_check_in ?? null };
        state.nextAssetId = state.assets.reduce((m, x) => Math.max(m, x.id || 0), 0) + 1;
        saveState(state);
        sessionStorage.removeItem(SS_UNLOCK);
        if (tables.security_config) state.masterPassword = state.masterPassword ?? 'preview';
        return ok(`已导入 ${incoming.length} 条资产（预览模式，${String(a['mode'] ?? 'merge') === 'replace' ? '覆盖' : '合并'}）`, '备份导入完成') as T;
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

    case 'get_heartbeat_config':
      // 演示环境的心跳状态：始终未配置，界面据此显示「家人收不到邮件」。
      // 名称与返回结构与桌面版命令一致（enabled / has_token / endpoint）。
      return ok({ enabled: false, has_token: false, endpoint: '' }) as T;

    case 'set_heartbeat_config':
      return ok(null, '演示模式不会真的发送心跳') as T;

    // 演示环境没有真实 SMTP：返回「未配置」让界面展示真实降级态，
    // 并让测试发送明确失败，而不是假装成功。
    case 'get_smtp_config':
      return ok({ host: '', port: 587, from_addr: '', username: '', has_password: false, allow_insecure: false }) as T;

    case 'set_smtp_config':
      return ok(null, '演示模式不会真的保存 SMTP 配置') as T;

    case 'get_heirs': {
      if (!state.heirs) {
        // 从旧档案字段派一名执行人，保持演示界面的报告结构完整
        state.heirs = state.profile?.heir_primary_name
          ? [{
              id: 1, name: state.profile.heir_primary_name, relation: state.profile.heir_primary_relation,
              phone: state.profile.heir_primary_phone, email: state.profile.heir_primary_email,
              share_note: '', is_executor: true, sort_order: 0,
            }]
          : [];
      }
      return ok(state.heirs) as T;
    }

    case 'set_heirs': {
      const list = (a['heirs'] as Array<Record<string, unknown>>) || [];
      // 与后端同一校验：恰好一位执行人
      if (list.length > 0 && list.filter(h => Boolean(h.is_executor)).length !== 1) {
        return fail<null>('必须恰好指定一位遗产执行人') as T;
      }
      state.heirs = list.map((h, i) => ({
        id: i + 1,
        name: String(h.name ?? ''),
        relation: String(h.relation ?? ''),
        phone: String(h.phone ?? ''),
        email: String(h.email ?? ''),
        share_note: String(h.share_note ?? ''),
        is_executor: Boolean(h.is_executor),
        sort_order: i,
      }));
      saveState(state);
      return ok(null, '继承人名单已保存（预览模式）') as T;
    }

    // 演示环境没有 AI 服务：配置恒为「关闭且无端点」，提问明确失败。
    case 'get_assistant_config':
      return ok({ enabled: false, endpoint: '', model: '', has_key: false }) as T;

    case 'set_assistant_config':
      return ok(null, '演示模式不会真的保存助手配置') as T;

    case 'preview_assistant_payload':
      return fail<null>('演示模式没有 AI 端点，无法预览出境内容') as T;

    case 'ask_assistant':
      return fail<null>('演示模式没有 AI 端点。本应用不内置任何 AI 服务') as T;

    case 'get_custody_hints':
      return ok({ backup_location: '', recovery_code_location: '' }) as T;

    case 'set_custody_hints': {
      const hints = (a['hints'] || {}) as { backup_location?: string; recovery_code_location?: string };
      state.custody = {
        backup_location: String(hints.backup_location ?? ''),
        recovery_code_location: String(hints.recovery_code_location ?? ''),
      };
      saveState(state);
      return ok(null, '保管位置已保存（预览模式）') as T;
    }

    case 'render_action_guide': {
      const r = (await mockInvoke('get_inheritance_report')) as { data?: { assets?: unknown[] } };
      const count = r.data?.assets?.length ?? 0;
      const loc = String(a['locale'] ?? 'zh-CN');
      const title = loc.startsWith('en') ? 'Digital Legacy Action Guide' : '数字遗产执行清单';
      return ok(
        `<!DOCTYPE html><html lang="${loc.startsWith('en') ? 'en' : 'zh-CN'}"><head><meta charset="UTF-8"><title>${title}</title>` +
        `<style>@media print{body{padding:0}}</style></head><body><div class="doc-title">${title}</div>` +
        `<p>演示清单：共 ${count} 条资产。</p></body></html>`,
        '执行清单已生成',
      ) as T;
    }

    case 'run_family_drill': {
      const assets = state.assets;
      const noHeir = assets.filter(x => x.disposal_method === 'inherit' && !(x.heir_name || '').trim()).length;
      const checks = [
        { key: 'heir', ok: noHeir === 0, count: noHeir },
        { key: 'policy', ok: true, count: 0 },
        { key: 'verified', ok: false, count: assets.length },
        { key: 'trigger', ok: state.dms.enabled, count: state.dms.enabled ? 0 : 1 },
      ];
      return ok({
        platforms: new Set(assets.map(x => x.platform_id)).size,
        assets: assets.length,
        assets_without_heir: noHeir,
        assets_without_policy: 0,
        unverified_platforms: assets.length,
        trigger_ready: state.dms.enabled,
        trigger_channel: state.dms.enabled ? 'heartbeat' : 'none',
        checks,
      }) as T;
    }

    case 'send_test_email':
      return fail<null>('演示模式没有 SMTP 服务器，无法发送测试邮件') as T;

    case 'reveal_exported_file':
      return ok(null, '演示模式不支持打开文件夹') as T;

    case 'get_backup_status':
      return ok({ count: 0, latest_name: null, latest_size: null }) as T;

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

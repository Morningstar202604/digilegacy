<template>
  <div class="profile fade-in">
    <h2 style="font-size: 22px; font-weight: 700; margin-bottom: 24px;">👤 我的档案</h2>
    <p class="text-muted mb-32" style="font-size: 14px;">完善你的个人信息，帮助继承人顺利找到并继承你的数字遗产。</p>

    <div class="grid grid-2">
      <div class="card">
        <div class="card-title mb-16">📝 基本信息</div>
        <div class="form-group">
          <label class="form-label" for="pf-full-name">姓名</label>
          <input class="form-input" id="pf-full-name" v-model="profile.full_name" placeholder="你的姓名" />
        </div>
        <div class="form-group">
          <label class="form-label" for="pf-phone">手机号</label>
          <input class="form-input" id="pf-phone" v-model="profile.phone" placeholder="手机号码" />
        </div>
        <div class="form-group">
          <label class="form-label" for="pf-email">邮箱</label>
          <input class="form-input" id="pf-email" v-model="profile.email" placeholder="电子邮箱" />
        </div>
      </div>

      <div class="card">
        <div class="card-title mb-16">🚨 紧急联系人</div>
        <div class="form-group">
          <label class="form-label" for="pf-ec-name">姓名</label>
          <input class="form-input" id="pf-ec-name" v-model="profile.emergency_contact_name" placeholder="紧急联系人姓名" />
        </div>
        <div class="form-group">
          <label class="form-label" for="pf-ec-phone">电话</label>
          <input class="form-input" id="pf-ec-phone" v-model="profile.emergency_contact_phone" placeholder="联系电话" />
        </div>
        <div class="form-group">
          <label class="form-label" for="pf-ec-email">邮箱</label>
          <input class="form-input" id="pf-ec-email" v-model="profile.emergency_contact_email" placeholder="电子邮箱（安心传承二阶段通知将优先发到该邮箱）" />
        </div>
        <div class="form-group">
          <label class="form-label" for="pf-ec-relation">关系</label>
          <input class="form-input" id="pf-ec-relation" v-model="profile.emergency_contact_relation" placeholder="如：子女、配偶" />
        </div>
      </div>

      <div class="card">
        <div class="card-title mb-16">👑 主要继承人</div>
        <div class="form-group">
          <label class="form-label" for="pf-heir-name">姓名</label>
          <input class="form-input" id="pf-heir-name" v-model="profile.heir_primary_name" placeholder="继承人姓名" />
        </div>
        <div class="form-group">
          <label class="form-label" for="pf-heir-phone">电话</label>
          <input class="form-input" id="pf-heir-phone" v-model="profile.heir_primary_phone" placeholder="联系电话" />
        </div>
        <div class="form-group">
          <label class="form-label" for="pf-heir-email">邮箱</label>
          <input class="form-input" id="pf-heir-email" v-model="profile.heir_primary_email" placeholder="电子邮箱（安心传承三阶段通知将优先发到该邮箱）" />
        </div>
        <div class="form-group">
          <label class="form-label" for="pf-heir-relation">关系</label>
          <input class="form-input" id="pf-heir-relation" v-model="profile.heir_primary_relation" placeholder="如：子女、父母" />
        </div>
      </div>

      <div class="card">
        <div class="card-title mb-16">⏰ Dead Man's Switch</div>
        <div class="form-group">
          <label class="form-label" for="pf-dms-interval">检查间隔</label>
          <select class="form-select" id="pf-dms-interval" v-model.number="profile.check_in_interval_days">
            <option :value="7">每周</option>
            <option :value="14">每两周</option>
            <option :value="30">每月</option>
            <option :value="60">每两月</option>
            <option :value="90">每季度</option>
          </select>
        </div>
        <div class="form-group">
          <label class="flex items-center gap-8" style="cursor: pointer;">
            <input type="checkbox" v-model="profile.dms_enabled" style="width: 18px; height: 18px;" />
            <span style="font-weight: 500;">启用 Dead Man's Switch</span>
          </label>
          <p class="form-hint">开启后将在每次到达检查周期时提醒你确认安全</p>
        </div>
      </div>
    </div>

    <div class="card mt-24">
      <div class="card-title mb-16">🔐 安全与加密</div>
      <template v-if="security.hasMasterPassword">
        <p style="font-size: 13px; color: var(--gray-600);">已启用主密码。资产账号、凭据、档案等敏感字段均以 AES-256-GCM 加密存储于本机，开发者也无法读取。</p>
        <button class="btn btn-secondary btn-sm" style="margin-top: 12px;" @click="lockNow">🔒 立即锁定</button>

        <div style="margin-top: 20px; padding-top: 16px; border-top: 1px solid var(--gray-200, #e8e4de);">
          <div style="font-weight: 600; font-size: 14px; margin-bottom: 8px;">🔑 恢复密钥</div>
          <p class="form-hint" style="margin-top: 0;">忘记主密码时，凭恢复密钥可重设密码并找回全部数据。设置主密码时已生成一次，请核对手抄记录是否可用；如已遗失，"修改主密码"会自动生成新的恢复密钥。</p>
          <div style="font-size: 13px; color: var(--gray-600);">状态：{{ recoveryExists === null ? '检查中…' : recoveryExists ? '已生成（见设置主密码时的提示，或随改密轮换）' : '未生成（建议修改一次主密码以生成）' }}</div>
        </div>

        <div style="margin-top: 20px; padding-top: 16px; border-top: 1px solid var(--gray-200, #e8e4de);">
          <div style="font-weight: 600; font-size: 14px; margin-bottom: 12px;">修改主密码</div>
          <div class="grid grid-2" style="gap: 16px;">
            <div class="form-group">
              <label class="form-label" for="pf-old-pw">当前主密码</label>
              <input class="form-input" id="pf-old-pw" type="password" v-model="oldPassword" placeholder="当前主密码" />
            </div>
            <div class="form-group">
              <label class="form-label" for="pf-new-pw">新主密码（至少 8 位）</label>
              <input class="form-input" id="pf-new-pw" type="password" v-model="newPassword" placeholder="新主密码" />
            </div>
            <div class="form-group">
              <label class="form-label" for="pf-new-pw2">确认新主密码</label>
              <input class="form-input" id="pf-new-pw2" type="password" v-model="newPassword2" placeholder="再次输入新主密码" />
            </div>
          </div>
          <button class="btn btn-secondary btn-sm" :disabled="security.busy" @click="changeMasterPassword">更新主密码（全库重新加密）</button>
          <p v-if="changeMessage" role="alert" style="font-size: 13px; margin-top: 8px;" :style="{ color: changeOk ? 'var(--success)' : 'var(--danger)' }">{{ changeMessage }}</p>
        </div>
      </template>
      <template v-else>
        <p style="font-size: 13px; color: var(--gray-600);">尚未启用加密。设置主密码后，账号、凭据、档案等敏感字段将在写入数据库前自动加密。</p>
        <div class="grid grid-2" style="margin-top: 12px; gap: 16px;">
          <div class="form-group">
            <label class="form-label" for="pf-master-pw">主密码（至少 8 位）</label>
            <input class="form-input" id="pf-master-pw" type="password" v-model="masterPassword" placeholder="设置主密码" />
          </div>
          <div class="form-group">
            <label class="form-label" for="pf-master-pw2">确认主密码</label>
            <input class="form-input" id="pf-master-pw2" type="password" v-model="masterPassword2" placeholder="再次输入" />
          </div>
        </div>
        <p class="form-hint">⚠️ 主密码只用于本机解密，遗失后无法找回已加密数据，请妥善保管（建议手写记录并放在安全位置）。</p>
        <button class="btn btn-primary btn-sm" style="margin-top: 12px;" :disabled="security.busy" @click="enableEncryption">启用加密保护</button>
        <p v-if="securityMessage" role="alert" style="font-size: 13px; margin-top: 8px;" :style="{ color: securityOk ? 'var(--success)' : 'var(--danger)' }">{{ securityMessage }}</p>
      </template>
    </div>

    <div class="card mt-24" v-if="recoveryCode" style="border-color: var(--warning); border-width: 2px;">
      <div class="card-title mb-16">🔑 恢复密钥 · 仅显示这一次，请立即抄写</div>
      <p style="font-size: 13px; color: var(--gray-600);">
        忘记主密码时，凭这串恢复密钥可以重设密码并找回全部数据。<strong>关闭本卡片后将无法再次查看</strong>（改密时会自动轮换生成新的）。
      </p>
      <div class="recovery-code-box">{{ recoveryCode }}</div>
      <div style="display: flex; gap: 12px; margin-top: 12px; align-items: center; flex-wrap: wrap;">
        <button class="btn btn-secondary btn-sm" @click="copyRecoveryCode">复制</button>
        <label style="display: flex; align-items: center; gap: 8px; cursor: pointer; font-size: 13px;">
          <input type="checkbox" v-model="recoveryAck" style="width: 18px; height: 18px;" />
          我已将恢复密钥抄写在安全位置（如纸质笔记本）
        </label>
        <button class="btn btn-primary btn-sm" :disabled="!recoveryAck" @click="recoveryCode = ''">我已保存，关闭</button>
      </div>
      <p class="form-hint">恢复密钥已用你的恢复码加密封存于本机数据库；但它无法从数据库中读出明文，丢失后无法找回。</p>
    </div>

    <div class="card mt-24">
      <div class="card-title mb-16">🗄 加密备份</div>
      <p style="font-size: 13px; color: var(--gray-600);">导出备份用于换电脑迁移或留档：备份内容为密文与安全配置，导入后使用同一主密码解锁。</p>
      <div style="display: flex; gap: 12px; margin-top: 12px; flex-wrap: wrap;">
        <button class="btn btn-secondary btn-sm" @click="exportBackup">导出加密备份</button>
        <button class="btn btn-secondary btn-sm" @click="importBackup">导入备份文件</button>
      </div>
      <p v-if="backupMessage" role="alert" style="font-size: 13px; margin-top: 8px;" :style="{ color: backupOk ? 'var(--success)' : 'var(--danger)' }">{{ backupMessage }}</p>
      <p v-if="!security.hasMasterPassword" class="form-hint">提示：尚未启用主密码时，备份中的敏感字段为明文，建议先启用加密后再导出。</p>
    </div>

    <div class="card mt-24">
      <div class="card-title mb-16">🛰 云端心跳（可选 · 零内容）</div>
      <p style="font-size: 13px; color: var(--gray-600);">开启后，应用每 30 分钟向心跳服务器发送一次"我还安好"令牌。<strong>只发送令牌，不发送任何资产、凭据或档案数据。</strong>服务器只在长时间收不到心跳时，按预注册的联系方式发提醒。失联时本地功能不受影响。</p>
      <template v-if="!isTauri">
        <p class="form-hint">浏览器预览模式不支持心跳，请在桌面版中使用。</p>
      </template>
      <template v-else>
        <p style="font-size: 12px; color: var(--gray-600); margin-top: 8px;">
          通过环境变量配置（DMS_HEARTBEAT_ENABLED=1、DMS_HEARTBEAT_TOKEN=你的令牌、可选 DMS_HEARTBEAT_ENDPOINT）。
        </p>
        <div style="display: flex; align-items: center; gap: 12px; margin-top: 12px; flex-wrap: wrap;">
          <button class="btn btn-secondary btn-sm" @click="sendHeartbeat">发送一次测试心跳</button>
          <span v-if="heartbeatMsg" style="font-size: 12px; color: var(--gray-600);" aria-live="polite">{{ heartbeatMsg }}</span>
        </div>
        <div style="margin-top: 16px; padding-top: 12px; border-top: 1px solid var(--gray-200, #e8e4de);">
          <label class="flex items-center gap-8" style="cursor: pointer;">
            <input type="checkbox" v-model="autostartEnabled" style="width: 18px; height: 18px;" @change="toggleAutostart" />
            <span style="font-weight: 500; font-size: 14px;">开机自动启动（驻留托盘，保证提醒与心跳持续运行）</span>
          </label>
          <p class="form-hint">关闭窗口时应用会驻留系统托盘继续工作；从托盘菜单可"显示/我还安好/锁定/退出"。</p>
        </div>
      </template>
    </div>

    <div class="card mt-24" style="text-align: center;">
      <button class="btn btn-primary btn-lg" @click="saveProfile">💾 保存档案</button>
      <p class="text-muted mt-16" style="font-size: 12px;">
        {{ security.hasMasterPassword ? '敏感数据已 AES-256-GCM 加密保存在本地，不上传任何服务器' : '数据保存在本机；启用主密码后敏感字段将加密存储' }}
      </p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';

import { useDashboardStore } from '../stores/dashboard';
import { useSecurityStore } from '../stores/security';
import { invoke, isTauriEnv } from '../api/invoke';
import type { ApiResponse, UserProfile } from '../types';

const dashboard = useDashboardStore();
const security = useSecurityStore();

const masterPassword = ref('');
const masterPassword2 = ref('');
const securityMessage = ref('');
const securityOk = ref(false);

const oldPassword = ref('');
const newPassword = ref('');
const newPassword2 = ref('');
const changeMessage = ref('');
const changeOk = ref(false);

const backupMessage = ref('');
const backupOk = ref(false);

const isTauri = isTauriEnv();
const recoveryExists = ref<boolean | null>(null);
const heartbeatMsg = ref('');
const autostartEnabled = ref(false);

const recoveryCode = ref('');
const recoveryAck = ref(false);

function copyRecoveryCode() {
  navigator.clipboard.writeText(recoveryCode.value).then(
    () => { alert('已复制到剪贴板，请粘贴到安全位置保存'); },
    () => { alert('复制失败，请手动抄写'); },
  );
}

async function sendHeartbeat() {
  heartbeatMsg.value = '发送中…';
  try {
    const res = await invoke<ApiResponse<{ last_result: string }>>('send_heartbeat_now');
    if (res.success && res.data) {
      heartbeatMsg.value = res.data.last_result === 'ok'
        ? '心跳发送成功（服务器已记录）'
        : '发送失败：服务器未连通（本地功能不受影响）。请确认已配置 DMS_HEARTBEAT_* 环境变量且服务器可达';
    } else {
      heartbeatMsg.value = res.message;
    }
  } catch (e) {
    heartbeatMsg.value = String(e);
  }
}

async function toggleAutostart() {
  try {
    const res = await invoke<ApiResponse<null>>('set_autostart', { enabled: autostartEnabled.value });
    if (!res.success) {
      autostartEnabled.value = !autostartEnabled.value;
      alert(res.message);
    }
  } catch (e) {
    autostartEnabled.value = !autostartEnabled.value;
    alert(String(e));
  }
}

async function changeMasterPassword() {
  changeMessage.value = '';
  if (!oldPassword.value || newPassword.value.length < 8) {
    changeMessage.value = '请填写当前主密码；新主密码至少 8 位';
    changeOk.value = false;
    return;
  }
  if (newPassword.value !== newPassword2.value) {
    changeMessage.value = '两次输入的新主密码不一致';
    changeOk.value = false;
    return;
  }
  try {
    const res = await invoke<ApiResponse<{ status: { has_master_password: boolean; unlocked: boolean }; recovery_code: string }>>('change_master_password', {
      oldPassword: oldPassword.value,
      newPassword: newPassword.value,
      confirmPassword: newPassword2.value,
    });
    changeOk.value = res.success;
    changeMessage.value = res.message;
    if (res.success) {
      oldPassword.value = '';
      newPassword.value = '';
      newPassword2.value = '';
      await security.refresh();
      // 改密即轮换恢复密钥：复用"仅显示这一次"卡片强制用户抄写新码
      if (res.data?.recovery_code) {
        recoveryCode.value = res.data.recovery_code;
        recoveryAck.value = false;
      }
    }
  } catch (e) {
    changeOk.value = false;
    changeMessage.value = String(e);
  }
}

async function exportBackup() {
  backupMessage.value = '';
  try {
    const res = await invoke<ApiResponse<string>>('export_backup');
    backupOk.value = res.success;
    backupMessage.value = res.success ? `备份已导出：${res.data}` : res.message;
  } catch (e) {
    backupOk.value = false;
    backupMessage.value = String(e);
  }
}

async function importBackup() {
  backupMessage.value = '';
  if (!isTauri) {
    backupOk.value = false;
    backupMessage.value = 'Web 预览模式不支持导入备份，请在桌面版中使用';
    return;
  }
  if (!confirm('导入备份将覆盖当前全部资产、档案与安全配置，且不可撤销。确定继续吗？')) {
    return;
  }
  try {
    const { open } = await import('@tauri-apps/plugin-dialog');
    const { readTextFile } = await import('@tauri-apps/plugin-fs');
    const selected = await open({ multiple: false, filters: [{ name: '归零备份', extensions: ['json'] }] });
    if (!selected || typeof selected !== 'string') return;
    const content = await readTextFile(selected);
    const res = await invoke<ApiResponse<string>>('import_backup', { content });
    backupOk.value = res.success;
    backupMessage.value = res.success ? `${res.message}：${res.data}` : res.message;
    if (res.success) {
      await security.refresh();
    }
  } catch (e) {
    backupOk.value = false;
    backupMessage.value = '导入失败：' + String(e);
  }
}

async function enableEncryption() {
  securityMessage.value = '';
  if (masterPassword.value.length < 8) {
    securityMessage.value = '主密码至少 8 个字符';
    securityOk.value = false;
    return;
  }
  if (masterPassword.value !== masterPassword2.value) {
    securityMessage.value = '两次输入的主密码不一致';
    securityOk.value = false;
    return;
  }
  const result = await security.setMasterPassword(masterPassword.value);
  if (!result.ok) {
    securityMessage.value = result.message;
    securityOk.value = false;
    return;
  }
  securityMessage.value = '加密已启用，存量数据已加密';
  securityOk.value = true;
  masterPassword.value = '';
  masterPassword2.value = '';
  // 恢复密钥强制展示，直到用户确认抄写完成
  recoveryCode.value = result.recoveryCode;
  recoveryAck.value = false;
}

async function lockNow() {
  await security.lockNow();
}

const profile = ref<UserProfile>({
  full_name: '', phone: '', email: '', id_last4: '',
  emergency_contact_name: '', emergency_contact_phone: '', emergency_contact_relation: '', emergency_contact_email: '',
  heir_primary_name: '', heir_primary_phone: '', heir_primary_relation: '', heir_primary_email: '',
  check_in_interval_days: 30, last_check_in: null, dms_enabled: false,
  created_at: '', updated_at: '',
});

onMounted(async () => {
  await dashboard.loadUser();
  if (dashboard.user) {
    profile.value = { ...profile.value, ...dashboard.user };
  }
  if (isTauri) {
    try {
      const res = await invoke<ApiResponse<boolean>>('get_recovery_seal_exists');
      recoveryExists.value = res.success ? Boolean(res.data) : null;
    } catch {
      recoveryExists.value = null;
    }
    try {
      const { isEnabled } = await import('@tauri-apps/plugin-autostart');
      autostartEnabled.value = await isEnabled();
    } catch {
      autostartEnabled.value = false;
    }
  }
});

async function saveProfile() {
  try {
    await dashboard.saveUser(profile.value);
    await dashboard.configureDms(profile.value.dms_enabled, profile.value.check_in_interval_days);
    alert('档案保存成功！');
  } catch (e) {
    // 保存失败必须给用户明确反馈，不得静默
    alert('档案保存失败：' + String(e));
  }
}
</script>

<style scoped>
.recovery-code-box {
  margin-top: 12px;
  padding: 12px 14px;
  border: 2px dashed var(--warning, #b7791f);
  border-radius: 8px;
  background: rgba(183, 121, 31, 0.08);
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  font-size: 15px;
  letter-spacing: 1px;
  word-break: break-all;
  text-align: center;
}
</style>

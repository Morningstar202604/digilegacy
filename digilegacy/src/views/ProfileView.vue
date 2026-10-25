<template>
  <div class="profile fade-in">
    <h2 style="font-size: 22px; font-weight: 700; margin-bottom: 24px;">{{ t('nav.profile') }}</h2>
    <p class="text-muted mb-32" style="font-size: 14px;">{{ t('profile.subtitle') }}</p>

    <div class="grid grid-2">
      <div class="card">
        <div class="card-title mb-16">{{ t('profile.basicTitle') }}</div>
        <div class="form-group">
          <label class="form-label" for="pf-full-name">{{ t('profile.fullName') }}</label>
          <input class="form-input" id="pf-full-name" v-model="profile.full_name" :placeholder="t('profile.fullNamePlaceholder')" />
        </div>
        <div class="form-group">
          <label class="form-label" for="pf-phone">{{ t('profile.phone') }}</label>
          <input class="form-input" id="pf-phone" v-model="profile.phone" :placeholder="t('profile.phonePlaceholder')" />
        </div>
        <div class="form-group">
          <label class="form-label" for="pf-email">{{ t('profile.email') }}</label>
          <input class="form-input" id="pf-email" v-model="profile.email" :placeholder="t('profile.emailPlaceholder')" />
        </div>
      </div>

      <div class="card">
        <div class="card-title mb-16">{{ t('profile.emergencyTitle') }}</div>
        <div class="form-group">
          <label class="form-label" for="pf-ec-name">{{ t('profile.fullName') }}</label>
          <input class="form-input" id="pf-ec-name" v-model="profile.emergency_contact_name" :placeholder="t('profile.emergencyNamePlaceholder')" />
        </div>
        <div class="form-group">
          <label class="form-label" for="pf-ec-phone">{{ t('profile.phone') }}</label>
          <input class="form-input" id="pf-ec-phone" v-model="profile.emergency_contact_phone" :placeholder="t('profile.contactPhonePlaceholder')" />
        </div>
        <div class="form-group">
          <label class="form-label" for="pf-ec-email">{{ t('profile.email') }}</label>
          <input class="form-input" id="pf-ec-email" v-model="profile.emergency_contact_email" :placeholder="t('profile.emergencyEmailPlaceholder')" />
        </div>
        <div class="form-group">
          <label class="form-label" for="pf-ec-relation">{{ t('profile.relation') }}</label>
          <input class="form-input" id="pf-ec-relation" v-model="profile.emergency_contact_relation" :placeholder="t('profile.relationPlaceholder')" />
        </div>
      </div>

      <div class="card">
        <div class="card-title mb-16">{{ t('profile.heirTitle') }}</div>
        <div class="form-group">
          <label class="form-label" for="pf-heir-name">{{ t('profile.fullName') }}</label>
          <input class="form-input" id="pf-heir-name" v-model="profile.heir_primary_name" :placeholder="t('profile.heirNamePlaceholder')" />
        </div>
        <div class="form-group">
          <label class="form-label" for="pf-heir-phone">{{ t('profile.phone') }}</label>
          <input class="form-input" id="pf-heir-phone" v-model="profile.heir_primary_phone" :placeholder="t('profile.contactPhonePlaceholder')" />
        </div>
        <div class="form-group">
          <label class="form-label" for="pf-heir-email">{{ t('profile.email') }}</label>
          <input class="form-input" id="pf-heir-email" v-model="profile.heir_primary_email" :placeholder="t('profile.heirEmailPlaceholder')" />
        </div>
        <div class="form-group">
          <label class="form-label" for="pf-heir-relation">{{ t('profile.relation') }}</label>
          <input class="form-input" id="pf-heir-relation" v-model="profile.heir_primary_relation" :placeholder="t('profile.relationPlaceholder2')" />
        </div>
      </div>

      <div class="card">
        <div class="card-title mb-16">⏰ Dead Man's Switch</div>
        <p style="font-size: 13px; color: var(--gray-600);">
          {{ t('profile.dmsNote') }}
        </p>
        <p class="form-hint" style="margin-top: 8px;">
          当前：{{ dmsSummary }}
        </p>
        <router-link to="/dms" class="btn btn-secondary btn-sm mt-16">{{ t('profile.dmsGo') }} →</router-link>
      </div>
    </div>

    <div class="card mt-24">
      <div class="card-title mb-16">{{ t('profile.securityTitle') }}</div>
      <template v-if="security.hasMasterPassword">
        <p style="font-size: 13px; color: var(--gray-600);">{{ t('profile.securityEnabledDesc') }}</p>
        <button class="btn btn-secondary btn-sm" style="margin-top: 12px;" @click="lockNow"><LockClosedOutline /> {{ t('profile.lockNow') }}</button>

        <div style="margin-top: 20px; padding-top: 16px; border-top: 1px solid var(--gray-200, #e8e4de);">
          <div style="font-weight: 600; font-size: 14px; margin-bottom: 8px;">{{ t('profile.recoveryKey') }}</div>
          <p class="form-hint" style="margin-top: 0;">{{ t('profile.recoveryKeyHint') }}</p>
          <div style="font-size: 13px; color: var(--gray-600);">{{ t('profile.recoveryStatus') }}：{{ recoveryExists === null ? t('profile.recoveryChecking') : recoveryExists ? t('profile.recoveryGenerated') : t('profile.recoveryMissing') }}</div>
        </div>

        <div style="margin-top: 20px; padding-top: 16px; border-top: 1px solid var(--gray-200, #e8e4de);">
          <div style="font-weight: 600; font-size: 14px; margin-bottom: 12px;">{{ t('profile.changePassword') }}</div>
          <div class="grid grid-2" style="gap: 16px;">
            <div class="form-group">
              <label class="form-label" for="pf-old-pw">{{ t('profile.oldPassword') }}</label>
              <input class="form-input" id="pf-old-pw" type="password" v-model="oldPassword" :placeholder="t('profile.oldPassword')" />
            </div>
            <div class="form-group">
              <label class="form-label" for="pf-new-pw">{{ t('recovery.newPasswordLabel') }}</label>
              <input class="form-input" id="pf-new-pw" type="password" v-model="newPassword" :placeholder="t('profile.newPasswordPlaceholder')" />
            </div>
            <div class="form-group">
              <label class="form-label" for="pf-new-pw2">{{ t('recovery.confirmPasswordLabel') }}</label>
              <input class="form-input" id="pf-new-pw2" type="password" v-model="newPassword2" :placeholder="t('profile.confirmPasswordPlaceholder')" />
            </div>
          </div>
          <button class="btn btn-secondary btn-sm" :disabled="security.busy" @click="changeMasterPassword">{{ t('profile.changePasswordBtn') }}</button>
          <p v-if="changeMessage" role="alert" style="font-size: 13px; margin-top: 8px;" :style="{ color: changeOk ? 'var(--success)' : 'var(--danger)' }">{{ changeMessage }}</p>
        </div>
      </template>
      <template v-else>
        <p style="font-size: 13px; color: var(--gray-600);">{{ t('profile.securityDisabledDesc') }}</p>
        <div class="grid grid-2" style="margin-top: 12px; gap: 16px;">
          <div class="form-group">
            <label class="form-label" for="pf-master-pw">{{ t('profile.setPasswordLabel') }}</label>
            <input class="form-input" id="pf-master-pw" type="password" v-model="masterPassword" :placeholder="t('profile.setPasswordPlaceholder')" />
          </div>
          <div class="form-group">
            <label class="form-label" for="pf-master-pw2">{{ t('recovery.confirmPasswordLabel') }}</label>
            <input class="form-input" id="pf-master-pw2" type="password" v-model="masterPassword2" :placeholder="t('profile.confirmPlaceholder')" />
          </div>
        </div>
        <p class="form-hint">{{ t('profile.passwordWarning') }}</p>
        <button class="btn btn-primary btn-sm" style="margin-top: 12px;" :disabled="security.busy" @click="enableEncryption">{{ t('profile.enableEncryption') }}</button>
        <p v-if="securityMessage" role="alert" style="font-size: 13px; margin-top: 8px;" :style="{ color: securityOk ? 'var(--success)' : 'var(--danger)' }">{{ securityMessage }}</p>
      </template>
    </div>

    <div class="card mt-24" v-if="recoveryCode" style="border-color: var(--warning); border-width: 2px;">
      <div class="card-title mb-16">{{ t('profile.recoveryOnce') }}</div>
      <p style="font-size: 13px; color: var(--gray-600);">
        {{ t('profile.recoveryOnceHint1') }}<strong>{{ t('profile.recoveryOnceHint2') }}</strong>{{ t('profile.recoveryOnceHint3') }}
      </p>
      <div class="recovery-code-box">{{ recoveryCode }}</div>
      <div style="display: flex; gap: 12px; margin-top: 12px; align-items: center; flex-wrap: wrap;">
        <button class="btn btn-secondary btn-sm" @click="copyRecoveryCode">{{ t('common.copy') }}</button>
        <label style="display: flex; align-items: center; gap: 8px; cursor: pointer; font-size: 13px;">
          <input type="checkbox" v-model="recoveryAck" style="width: 18px; height: 18px;" />
          {{ t('profile.recoveryWritten') }}
        </label>
        <button class="btn btn-primary btn-sm" :disabled="!recoveryAck" @click="recoveryCode = ''">{{ t('recovery.savedClose') }}</button>
      </div>
      <p class="form-hint">{{ t('profile.recoverySealedHint') }}</p>
    </div>

    <div class="card mt-24">
      <div class="card-title mb-16">{{ t('profile.backupTitle') }}</div>
      <p style="font-size: 13px; color: var(--gray-600);">{{ t('profile.backupDesc') }}</p>
      <div style="display: flex; gap: 12px; margin-top: 12px; flex-wrap: wrap;">
        <button class="btn btn-secondary btn-sm" @click="exportBackup">{{ t('profile.exportBackup') }}</button>
        <button class="btn btn-secondary btn-sm" @click="importBackup">{{ t('profile.importBackup') }}</button>
      </div>
      <p v-if="backupMessage" role="alert" style="font-size: 13px; margin-top: 8px;" :style="{ color: backupOk ? 'var(--success)' : 'var(--danger)' }">{{ backupMessage }}</p>
      <p v-if="!security.hasMasterPassword" class="form-hint">{{ t('profile.backupPlainHint') }}</p>
    </div>

    <div class="card mt-24">
      <div class="card-title mb-16">{{ t('profile.heartbeatTitle') }}</div>
      <p style="font-size: 13px; color: var(--gray-600);">{{ t('profile.heartbeatDesc1') }}<strong>{{ t('profile.heartbeatDesc2') }}</strong>{{ t('profile.heartbeatDesc3') }}</p>
      <template v-if="!isTauri">
        <p class="form-hint">{{ t('profile.browserUnsupported') }}</p>
      </template>
      <template v-else>
        <div style="margin-top: 12px; display: grid; gap: 10px; max-width: 520px;">
          <label class="flex items-center gap-8" style="cursor: pointer;">
            <input type="checkbox" v-model="heartbeatEnabled" style="width: 18px; height: 18px;" />
            <span style="font-weight: 500; font-size: 14px;">{{ t('profile.heartbeatEnable') }}</span>
          </label>
          <div class="form-group" style="margin: 0;">
            <label class="form-label" for="hb-endpoint">{{ t('profile.heartbeatEndpoint') }}</label>
            <input class="form-input" id="hb-endpoint" v-model="heartbeatEndpoint" :placeholder="t('profile.heartbeatEndpointPlaceholder')" />
            <p class="form-hint">{{ t('profile.heartbeatSelfHost') }}</p>
          </div>
          <div class="form-group" style="margin: 0;">
            <label class="form-label" for="hb-token">{{ t('profile.heartbeatToken') }}</label>
            <input class="form-input" id="hb-token" type="password" v-model="heartbeatToken" :placeholder="heartbeatHasToken ? t('profile.tokenSaved') : t('profile.tokenPlaceholder')" />
          </div>
          <div>
            <button class="btn btn-primary btn-sm" :disabled="heartbeatBusy" @click="saveHeartbeatConfig">{{ heartbeatBusy ? t('common.saving') : t('profile.saveHeartbeat') }}</button>
          </div>
        </div>
        <p v-if="heartbeatMsg" style="font-size: 12px; color: var(--gray-600); margin-top: 8px;" aria-live="polite">{{ heartbeatMsg }}</p>
        <div style="display: flex; align-items: center; gap: 12px; margin-top: 12px; flex-wrap: wrap;">
          <button class="btn btn-secondary btn-sm" @click="sendHeartbeat">{{ t('profile.testHeartbeat') }}</button>
          <span v-if="heartbeatTestMsg" style="font-size: 12px; color: var(--gray-600);" aria-live="polite">{{ heartbeatTestMsg }}</span>
        </div>
        <div style="margin-top: 16px; padding-top: 12px; border-top: 1px solid var(--gray-200, #e8e4de);">
          <label class="flex items-center gap-8" style="cursor: pointer;">
            <input type="checkbox" v-model="autostartEnabled" style="width: 18px; height: 18px;" @change="toggleAutostart" />
            <span style="font-weight: 500; font-size: 14px;">{{ t('profile.autostart') }}</span>
          </label>
          <p class="form-hint">{{ t('profile.autostartHint') }}</p>
        </div>
      </template>
    </div>

    <div class="card mt-24">
      <div class="card-title mb-16">{{ t('profile.patchTitle') }}</div>
      <p style="font-size: 13px; color: var(--gray-600);">{{ t('profile.patchDesc') }}</p>
      <template v-if="!isTauri">
        <p class="form-hint">{{ t('profile.browserUnsupported') }}</p>
      </template>
      <template v-else>
        <p class="form-hint" style="margin-top: 8px;">{{ t('profile.patchCount') }} <strong>{{ patchCount }}</strong> {{ t('profile.patchCountSuffix') }}。</p>
        <div style="display: flex; gap: 10px; flex-wrap: wrap; margin-top: 12px;">
          <button class="btn btn-secondary btn-sm" @click="importPatch">{{ t('profile.importPatch') }}</button>
          <button class="btn btn-secondary btn-sm" @click="exportPatch">{{ patchCount > 0 ? t('profile.exportPatch') : t('profile.exportPatchTemplate') }}</button>
          <button class="btn btn-secondary btn-sm" :disabled="patchCount === 0" @click="clearPatch">{{ t('profile.clearPatch') }}</button>
        </div>
        <p v-if="patchMsg" style="font-size: 12px; color: var(--gray-600); margin-top: 8px;" aria-live="polite">{{ patchMsg }}</p>
      </template>
    </div>

    <div class="card mt-24" style="text-align: center;">
      <button class="btn btn-primary btn-lg" @click="saveProfile">{{ t('profile.saveProfile') }}</button>
      <p class="text-muted mt-16" style="font-size: 12px;">
        {{ security.hasMasterPassword ? t('profile.saveHintEncrypted') : t('profile.saveHintPlain') }}
      </p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';

import { useI18n } from '../i18n';
import { useDashboardStore } from '../stores/dashboard';
import LockClosedOutline from '@vicons/ionicons5/es/LockClosedOutline';
import { useSecurityStore } from '../stores/security';
import { usePlatformsStore } from '../stores/platforms';
import { useAssetsStore } from '../stores/assets';
import { toast, confirmDialog } from '../composables/feedback';
import { invoke, isTauriEnv } from '../api/invoke';
import type { ApiResponse, UserProfile } from '../types';

const { t } = useI18n();
const dashboard = useDashboardStore();
const security = useSecurityStore();
const platforms = usePlatformsStore();
const assets = useAssetsStore();

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
const heartbeatTestMsg = ref('');
const heartbeatEnabled = ref(false);
const heartbeatEndpoint = ref('');
const heartbeatToken = ref('');
const heartbeatHasToken = ref(false);
const heartbeatBusy = ref(false);
const patchCount = ref(0);
const patchMsg = ref('');
const autostartEnabled = ref(false);

const recoveryCode = ref('');
const recoveryAck = ref(false);

function copyRecoveryCode() {
  navigator.clipboard.writeText(recoveryCode.value).then(
    () => { toast(t('profile.copied')); },
    () => { toast(t('profile.copyFailed'), 'error'); },
  );
}

async function loadHeartbeatConfig() {
  try {
    const res = await invoke<ApiResponse<{ enabled: boolean; has_token: boolean; endpoint: string }>>('get_heartbeat_config');
    if (res.success && res.data) {
      heartbeatEnabled.value = res.data.enabled;
      heartbeatHasToken.value = res.data.has_token;
      heartbeatEndpoint.value = res.data.endpoint;
    }
  } catch {
    // 读取失败时保持默认空态，用户保存时会重新写入
  }
}

async function saveHeartbeatConfig() {
  heartbeatBusy.value = true;
  heartbeatMsg.value = '';
  try {
    const res = await invoke<ApiResponse<null>>('set_heartbeat_config', {
      enabled: heartbeatEnabled.value,
      token: heartbeatToken.value,
      endpoint: heartbeatEndpoint.value,
    });
    heartbeatMsg.value = res.message;
    if (res.success) {
      heartbeatToken.value = '';
      await loadHeartbeatConfig();
    }
  } catch (e) {
    heartbeatMsg.value = String(e);
  } finally {
    heartbeatBusy.value = false;
  }
}

async function sendHeartbeat() {
  heartbeatTestMsg.value = t('profile.sending');
  try {
    const res = await invoke<ApiResponse<{ last_result: string }>>('send_heartbeat_now');
    if (res.success && res.data) {
      heartbeatTestMsg.value = res.data.last_result === 'ok'
        ? t('profile.heartbeatOk')
        : t('profile.heartbeatNotSent');
    } else {
      heartbeatTestMsg.value = res.message;
    }
  } catch (e) {
    heartbeatTestMsg.value = String(e);
  }
}

async function toggleAutostart() {
  try {
    const res = await invoke<ApiResponse<null>>('set_autostart', { enabled: autostartEnabled.value });
    if (!res.success) {
      autostartEnabled.value = !autostartEnabled.value;
      toast(res.message, 'error');
    }
  } catch (e) {
    autostartEnabled.value = !autostartEnabled.value;
    toast(String(e), 'error');
  }
}

async function changeMasterPassword() {
  changeMessage.value = '';
  if (!oldPassword.value || newPassword.value.length < 8) {
    changeMessage.value = t('profile.changeReq');
    changeOk.value = false;
    return;
  }
  if (newPassword.value !== newPassword2.value) {
    changeMessage.value = t('profile.passwordMismatch');
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
    backupMessage.value = res.success ? t('profile.backupExported', { path: res.data }) : res.message;
  } catch (e) {
    backupOk.value = false;
    backupMessage.value = String(e);
  }
}

async function importBackup() {
  backupMessage.value = '';
  if (!isTauri) {
    backupOk.value = false;
    backupMessage.value = t('profile.webImportUnsupported');
    return;
  }
  const ok = await confirmDialog({
    title: t('profile.importBackupTitle'),
    message: t('profile.importBackupMessage'),
    confirmText: t('profile.importBackupConfirm'),
    danger: true,
  });
  if (!ok) return;
  try {
    const { open } = await import('@tauri-apps/plugin-dialog');
    const { readTextFile } = await import('@tauri-apps/plugin-fs');
    const selected = await open({ multiple: false, filters: [{ name: t('profile.backupFilterName'), extensions: ['json'] }] });
    if (!selected || typeof selected !== 'string') return;
    const content = await readTextFile(selected);
    const res = await invoke<ApiResponse<string>>('import_backup', { content });
    backupOk.value = res.success;
    backupMessage.value = res.success ? `${res.message}：${res.data}` : res.message;
    if (res.success) {
      // 备份已覆盖数据库：必须让所有数据视图重新拉取，
      // 否则用户看到的是导入前的旧列表，会误判「恢复失败」并反复导入。
      await Promise.all([
        security.refresh(),
        dashboard.refreshAll(),
        assets.loadAssets(),
        platforms.loadPlatforms(true),
      ]);
    }
  } catch (e) {
    backupOk.value = false;
    backupMessage.value = t('profile.importFailed', { e: String(e) });
  }
}

async function importPatch() {
  patchMsg.value = '';
  try {
    const { open } = await import('@tauri-apps/plugin-dialog');
    const { readTextFile } = await import('@tauri-apps/plugin-fs');
    const selected = await open({ multiple: false, filters: [{ name: t('profile.patchFilterName'), extensions: ['json'] }] });
    if (!selected || typeof selected !== 'string') return;
    const content = await readTextFile(selected);
    const res = await invoke<ApiResponse<string>>('import_platform_patch', { content });
    patchMsg.value = res.success ? `${res.message}：${res.data}` : res.message;
    if (res.success) {
      await loadPatchStatus();
      await platforms.loadPlatforms();
    }
  } catch (e) {
    patchMsg.value = t('profile.importFailed', { e: String(e) });
  }
}

async function exportPatch() {
  patchMsg.value = '';
  try {
    // 后端直接写进应用数据目录（与备份导出同路径策略），无需前端写文件权限
    const res = await invoke<ApiResponse<string>>('export_platform_patch');
    patchMsg.value = res.success ? `${res.message}：${res.data}` : res.message;
  } catch (e) {
    patchMsg.value = t('profile.exportFailed', { e: String(e) });
  }
}

async function clearPatch() {
  const ok = await confirmDialog({
    title: t('profile.clearPatchTitle'),
    message: t('profile.clearPatchMessage'),
    confirmText: t('common.delete'),
    danger: true,
  });
  if (!ok) return;
  try {
    const res = await invoke<ApiResponse<number>>('clear_platform_patch');
    patchMsg.value = res.success ? t('profile.patchCleared', { count: res.data }) : res.message;
    if (res.success) {
      await loadPatchStatus();
      await platforms.loadPlatforms();
    }
  } catch (e) {
    patchMsg.value = t('profile.clearFailed', { e: String(e) });
  }
}

async function loadPatchStatus() {
  try {
    const res = await invoke<ApiResponse<number>>('platform_patch_status');
    if (res.success) patchCount.value = Number(res.data ?? 0);
  } catch {
    patchCount.value = 0;
  }
}

async function enableEncryption() {
  securityMessage.value = '';
  if (masterPassword.value.length < 8) {
    securityMessage.value = t('profile.passwordTooShort');
    securityOk.value = false;
    return;
  }
  if (masterPassword.value !== masterPassword2.value) {
    securityMessage.value = t('profile.passwordMismatch');
    securityOk.value = false;
    return;
  }
  const result = await security.setMasterPassword(masterPassword.value);
  if (!result.ok) {
    securityMessage.value = result.message;
    securityOk.value = false;
    return;
  }
  securityMessage.value = t('profile.encryptionEnabled');
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

/** DMS 配置的唯一事实源是 dms_config（安心传承页维护），档案页只读展示。 */
const dmsSummary = computed(() => {
  const st = dashboard.checkInStatus;
  if (!st) return t('profile.dmsReading');
  if (!st.dms_enabled) return t('profile.dmsDisabled');
  return t('profile.dmsSummary', { days: st.interval_days });
});

onMounted(async () => {
  await Promise.all([dashboard.loadUser(), dashboard.loadCheckInStatus()]);
  if (dashboard.user) {
    profile.value = { ...profile.value, ...dashboard.user };
    // 档案表里的 DMS 副本可能是旧值；显示与保存都跟随 dms_config
    if (dashboard.checkInStatus) {
      profile.value.dms_enabled = dashboard.checkInStatus.dms_enabled;
      profile.value.check_in_interval_days = dashboard.checkInStatus.interval_days;
    }
  }
  if (isTauri) {
    try {
      const res = await invoke<ApiResponse<boolean>>('get_recovery_seal_exists');
      recoveryExists.value = res.success ? Boolean(res.data) : null;
    } catch {
      recoveryExists.value = null;
    }
    await loadHeartbeatConfig();
    await loadPatchStatus();
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
    // 不再把档案页里的旧 DMS 值回写 dms_config：那会让用户在「安心传承」
    // 设好的 90 天周期，被一次无关的「保存档案」静默改回 30 天。
    toast(t('profile.saveOk'));
  } catch (e) {
    // 保存失败必须给用户明确反馈，不得静默
    toast(t('profile.saveFailed', { e: String(e) }), 'error');
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

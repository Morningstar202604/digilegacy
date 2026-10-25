<template>
  <div class="dms fade-in">
    <h2 style="font-size: 22px; font-weight: 700; margin-bottom: 8px;">⏰ {{ t('nav.dms') }} · Dead Man's Switch</h2>
    <p class="text-muted mb-32" style="font-size: 14px;">{{ t('dms.subtitle') }}</p>

    <!-- 状态卡片 -->
    <div class="card mb-32" :class="statusClass">
      <div class="flex items-center gap-12">
        <div class="dms-status-icon" aria-hidden="true"><component :is="statusIcon" /></div>
        <div>
          <div style="font-size: 20px; font-weight: 700;" :style="{ color: statusColor }">{{ statusTitle }}</div>
          <div class="text-muted mt-16" style="font-size: 13px;">{{ statusSubtitle }}</div>
        </div>
      </div>
    </div>

    <!-- 操作面板 -->
    <div class="grid grid-2 mb-32">
      <div class="card">
        <div class="card-title mb-16">{{ t('dms.checkInTitle') }}</div>
        <button class="btn btn-success btn-lg" @click="handleCheckIn" :disabled="checking">
          {{ checking ? t('dms.checking') : '✓ ' + t('dms.checkInBtn') }}
        </button>
        <div v-if="lastCheckIn" class="mt-16 text-muted" style="font-size: 13px;">{{ t('dms.lastCheckIn') }}：{{ lastCheckIn }}</div>
      </div>

      <div class="card">
        <div class="card-title mb-16">{{ t('dms.settingsTitle') }}</div>
        <div class="form-group">
          <label class="flex items-center gap-8" style="cursor: pointer;">
            <input type="checkbox" id="dms-enabled" v-model="settings.enabled" style="width: 18px; height: 18px;" />
            <span style="font-weight: 500; font-size: 14px;">{{ t('dms.enable') }}</span>
          </label>
          <p class="form-hint">{{ t('dms.disableHint') }}</p>
        </div>
        <div class="form-group">
          <label class="form-label" for="dms-interval">{{ t('dms.intervalLabel') }}</label>
          <select class="form-select" id="dms-interval" v-model.number="settings.interval" :disabled="!settings.enabled">
            <option v-for="opt in INTERVAL_OPTIONS" :key="opt.value" :value="opt.value">{{ t(opt.labelKey) }}</option>
          </select>
        </div>
        <button class="btn btn-primary btn-sm mt-16" @click="saveSettings">{{ t('dms.saveSettings') }}</button>
      </div>
    </div>

    <!-- 本地提醒 -->
    <div class="card mb-32">
      <div class="card-title mb-16">{{ t('dms.localReminders') }}</div>
      <ul class="dms-note-list">
        <li>{{ t('dms.reminder1a') }} <strong>{{ t('dms.reminder1b') }}</strong> {{ t('dms.reminder1c') }}</li>
        <li>{{ t('dms.reminder2a') }}<strong>{{ t('dms.reminder2b') }}</strong>{{ t('dms.reminder2c') }}</li>
        <li>{{ t('dms.reminder3a') }}<strong>{{ t('dms.reminder3b') }}</strong>{{ t('dms.reminder3c') }}</li>
      </ul>
      <div style="display: flex; align-items: center; gap: 12px; margin-top: 14px; flex-wrap: wrap;">
        <button class="btn btn-secondary btn-sm" :disabled="testingNotify" @click="handleTestNotify">{{ testingNotify ? t('dms.sending') : t('dms.testNotify') }}</button>
        <span v-if="testResult" style="font-size: 12px;" aria-live="polite" :style="{ color: testResult === 'ok' ? 'var(--success)' : 'var(--danger)' }">
          {{ testResult === 'ok' ? t('dms.testOk') : t('dms.testFail') }}
        </span>
      </div>
    </div>

    <!-- 三级确认说明 -->
    <div class="card">
      <div class="card-title mb-16">{{ t('dms.stagesTitle') }}</div>
      <div class="dms-stages">
        <div class="dms-stage" :class="{ 'dms-stage-active': dmsEvaluation?.current_stage === 1 }">
          <span class="dms-stage-num">1</span>
          <div><strong>{{ t('dms.stage1Title') }}</strong><p class="text-muted">{{ t('dms.stage1Desc') }}</p></div>
        </div>
        <div class="dms-stage" :class="{ 'dms-stage-active': dmsEvaluation?.current_stage === 2 }">
          <span class="dms-stage-num">2</span>
          <div><strong>{{ t('dms.stage2Title') }}</strong><p class="text-muted">{{ t('dms.stage2Desc') }}</p></div>
        </div>
        <div class="dms-stage" :class="{ 'dms-stage-active': dmsEvaluation?.current_stage === 3 }">
          <span class="dms-stage-num">3</span>
          <div><strong>{{ t('dms.stage3Title') }}</strong><p class="text-muted">{{ t('dms.stage3Desc') }}</p></div>
        </div>
      </div>
    </div>

    <!-- 通知记录 -->
    <div class="card" v-if="dmsEvaluation && dmsEvaluation.notifications.length > 0">
      <div class="card-title mb-16">{{ t('dms.logTitle') }}</div>
      <div v-for="(n, i) in dmsEvaluation.notifications" :key="i" class="card mb-16" style="padding: 12px 16px;">
        <div class="flex items-center gap-12">
          <span class="tag" :class="n.stage >= 3 ? 'tag-danger' : n.stage >= 2 ? 'tag-warning' : 'tag-property'">{{ t('dms.stageTag', { stage: n.stage }) }}</span>
          <div style="flex: 1;">
            <div style="font-weight: 600;">{{ n.subject }}</div>
            <div class="text-muted" style="font-size: 12px;">{{ t('dms.recipient') }}：{{ n.recipient || t('dms.notConfigured') }} · {{ t('dms.channel') }}：{{ n.channel }} · {{ n.sent_at }}</div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useI18n } from '../i18n';
import AlertCircleOutline from '@vicons/ionicons5/es/AlertCircleOutline';
import CheckmarkCircleOutline from '@vicons/ionicons5/es/CheckmarkCircleOutline';
import RemoveCircleOutline from '@vicons/ionicons5/es/RemoveCircleOutline';
import WarningOutline from '@vicons/ionicons5/es/WarningOutline';
import { useDashboardStore } from '../stores/dashboard';
import { sendTestNotification } from '../api/reminders';
import { toast } from '../composables/feedback';
import { INTERVAL_OPTIONS } from '../constants/dms';

const { t } = useI18n();
const dashboard = useDashboardStore();
const checking = ref(false);
const settings = ref({ interval: 30, enabled: false });
const testingNotify = ref(false);
const testResult = ref<'' | 'ok' | 'fail'>('');

async function handleTestNotify() {
  testingNotify.value = true;
  try {
    const ok = await sendTestNotification();
    testResult.value = ok ? 'ok' : 'fail';
  } finally {
    testingNotify.value = false;
  }
}

const status = computed(() => dashboard.checkInStatus);
const lastCheckIn = computed(() => status.value?.last_check_in);
const daysUntilDue = computed(() => status.value?.days_until_due);
const dmsEvaluation = computed(() => dashboard.dmsEvaluation);

const dmsState = computed(() => {
  if (!status.value?.dms_enabled) return 'disabled';
  if ((daysUntilDue.value ?? 999) <= 0) return 'overdue';
  if ((daysUntilDue.value ?? 999) <= 3) return 'warning';
  return 'ok';
});

const statusIcon = computed(() => ({ ok: CheckmarkCircleOutline, warning: WarningOutline, overdue: AlertCircleOutline, disabled: RemoveCircleOutline }[dmsState.value]));
const statusTitle = computed(() => ({
  ok: t('dms.stateOk'), warning: t('dms.stateWarning'), overdue: t('dms.stateOverdue'), disabled: t('dms.stateDisabled'),
}[dmsState.value]));
const statusSubtitle = computed(() => {
  const days = daysUntilDue.value;
  if (dmsState.value === 'ok') {
    return days === null || days === undefined
      ? t('dms.stateOkSub')
      : t('dms.daysLeft', { days });
  }
  if (dmsState.value === 'warning') {
    return days === null || days === undefined
      ? t('dms.confirmSoon')
      : t('dms.daysLeftUrgent', { days });
  }
  return {
    overdue: t('dms.stateOverdueSub'),
    disabled: t('dms.stateDisabledSub'),
  }[dmsState.value] || '';
});
const statusColor = computed(() => ({
  ok: 'var(--success-strong)', warning: 'var(--warning-strong)', overdue: 'var(--danger-strong)', disabled: 'var(--gray-600)',
}[dmsState.value]));
const statusClass = computed(() => ({
  ok: 'dms-ok', warning: 'dms-warning', overdue: 'dms-overdue', disabled: 'dms-disabled',
}[dmsState.value]));

async function handleCheckIn() {
  checking.value = true;
  try {
    await dashboard.checkIn();
  } finally {
    checking.value = false;
  }
}

async function saveSettings() {
  try {
    await dashboard.configureDms(settings.value.enabled, settings.value.interval);
    toast(t('dms.saved'));
  } catch (e) {
    toast(t('dms.saveFailed', { e: String(e) }), 'error');
  }
}

onMounted(async () => {
  await Promise.all([dashboard.loadCheckInStatus(), dashboard.evaluateDms()]);
  if (status.value) {
    settings.value.interval = status.value.interval_days;
    settings.value.enabled = status.value.dms_enabled;
  }
});
</script>

<style scoped>
.dms-ok { background: var(--success-light); border-color: var(--success); }
.dms-warning { background: var(--warning-light); border-color: var(--warning); }
.dms-overdue { background: var(--danger-light); border-color: var(--danger); }
.dms-disabled { background: var(--gray-50); border-color: var(--gray-200); }
.dms-stages { display: flex; flex-direction: column; gap: 16px; }
.dms-stage { display: flex; gap: 12px; padding: 12px 16px; border-radius: 8px; }
.dms-stage p { font-size: 13px; margin-top: 2px; }
.dms-note-list { margin: 0; padding-left: 18px; display: flex; flex-direction: column; gap: 6px; font-size: 13px; color: var(--gray-600, #5c6f79); line-height: 1.65; }
.dms-note-list strong { color: var(--gray-800, #37464f); }
.dms-stage-1 { background: var(--success-light); }
.dms-stage-2 { background: var(--warning-light); }
.dms-stage-3 { background: var(--danger-light); }
.dms-stage-active { outline: 2px solid var(--primary); outline-offset: 2px; }
.dms-stage-num { font-size: 24px; }
</style>

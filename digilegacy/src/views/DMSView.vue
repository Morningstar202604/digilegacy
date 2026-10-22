<template>
  <div class="dms fade-in">
    <h2 style="font-size: 22px; font-weight: 700; margin-bottom: 8px;">⏰ Dead Man's Switch · 安心传承</h2>
    <p class="text-muted mb-32" style="font-size: 14px;">确认你还安好。如果超出一段时间未确认，系统将按设定通知你关心的人。</p>

    <!-- 状态卡片 -->
    <div class="card mb-32" :class="statusClass">
      <div class="flex items-center gap-12">
        <div style="font-size: 48px;" aria-hidden="true">{{ statusIcon }}</div>
        <div>
          <div style="font-size: 20px; font-weight: 700;" :style="{ color: statusColor }">{{ statusTitle }}</div>
          <div class="text-muted mt-16" style="font-size: 13px;">{{ statusSubtitle }}</div>
        </div>
      </div>
    </div>

    <!-- 操作面板 -->
    <div class="grid grid-2 mb-32">
      <div class="card">
        <div class="card-title mb-16">👋 确认你安好</div>
        <button class="btn btn-success btn-lg" @click="handleCheckIn" :disabled="checking">
          {{ checking ? '确认中...' : '✓ 我还安好，确认检查' }}
        </button>
        <div v-if="lastCheckIn" class="mt-16 text-muted" style="font-size: 13px;">上次确认：{{ lastCheckIn }}</div>
      </div>

      <div class="card">
        <div class="card-title mb-16">⚙️ 检查设置</div>
        <div class="form-group">
          <label class="form-label" for="dms-interval">检查间隔</label>
          <select class="form-select" id="dms-interval" v-model.number="settings.interval">
            <option :value="7">每周确认一次</option>
            <option :value="14">每两周确认一次</option>
            <option :value="30">每月确认一次</option>
            <option :value="90">每季度确认一次</option>
          </select>
        </div>
        <button class="btn btn-primary btn-sm mt-16" @click="saveSettings">保存设置</button>
      </div>
    </div>

    <!-- 本地提醒 -->
    <div class="card mb-32">
      <div class="card-title mb-16">🔔 本地提醒</div>
      <ul class="dms-note-list">
        <li>应用运行期间每 30 分钟自动检查：距确认到期 <strong>3 天内或已逾期</strong>，会弹出系统通知（每天至多一条）。</li>
        <li>分级通知同步写入<strong>本地发件箱</strong>（应用数据目录 `dms_outbox`）；配置邮件环境变量 DMS_SMTP_HOST / DMS_SMTP_PORT / SMTP_FROM（可选 DMS_SMTP_USER / DMS_SMTP_PASS）后可同时发送邮件，默认要求 STARTTLS 加密。</li>
        <li>短信与跨设备推送属于后续<strong>云端计划</strong>：服务端接入后启用，本地数据已为其预留。</li>
      </ul>
      <div style="display: flex; align-items: center; gap: 12px; margin-top: 14px; flex-wrap: wrap;">
        <button class="btn btn-secondary btn-sm" :disabled="testingNotify" @click="handleTestNotify">{{ testingNotify ? '发送中...' : '发送测试提醒' }}</button>
        <span v-if="testResult" style="font-size: 12px;" aria-live="polite" :style="{ color: testResult === 'ok' ? 'var(--success)' : 'var(--danger)' }">
          {{ testResult === 'ok' ? '已发送，请查看系统通知' : '发送失败或当前环境不支持（浏览器预览不发系统通知）' }}
        </span>
      </div>
    </div>

    <!-- 三级确认说明 -->
    <div class="card">
      <div class="card-title mb-16">🔄 三级确认机制</div>
      <div class="dms-stages">
        <div class="dms-stage" :class="{ 'dms-stage-active': dmsEvaluation?.current_stage === 1 }">
          <span class="dms-stage-num">1️⃣</span>
          <div><strong>第一阶段（到期后3天）</strong><p class="text-muted">系统通知 + 邮件提醒你本人确认安全（通知先写入本地发件箱）</p></div>
        </div>
        <div class="dms-stage" :class="{ 'dms-stage-active': dmsEvaluation?.current_stage === 2 }">
          <span class="dms-stage-num">2️⃣</span>
          <div><strong>第二阶段（再7天后）</strong><p class="text-muted">写信通知紧急联系人确认你的状态（优先发至其邮箱；邮件 / 本地发件箱）</p></div>
        </div>
        <div class="dms-stage" :class="{ 'dms-stage-active': dmsEvaluation?.current_stage === 3 }">
          <span class="dms-stage-num">3️⃣</span>
          <div><strong>第三阶段（再14天后）</strong><p class="text-muted">向遗产执行人发送数字遗产继承指引邮件（不含任何账号凭据；邮件 / 本地发件箱）</p></div>
        </div>
      </div>
    </div>

    <!-- 通知记录 -->
    <div class="card" v-if="dmsEvaluation && dmsEvaluation.notifications.length > 0">
      <div class="card-title mb-16">📨 通知记录（今日）</div>
      <div v-for="(n, i) in dmsEvaluation.notifications" :key="i" class="card mb-16" style="padding: 12px 16px;">
        <div class="flex items-center gap-12">
          <span class="tag" :class="n.stage >= 3 ? 'tag-danger' : n.stage >= 2 ? 'tag-warning' : 'tag-property'">阶段 {{ n.stage }}</span>
          <div style="flex: 1;">
            <div style="font-weight: 600;">{{ n.subject }}</div>
            <div class="text-muted" style="font-size: 12px;">收件人：{{ n.recipient || '未配置' }} · 渠道：{{ n.channel }} · {{ n.sent_at }}</div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useDashboardStore } from '../stores/dashboard';
import { sendTestNotification } from '../api/reminders';
import { toast } from '../composables/feedback';

const dashboard = useDashboardStore();
const checking = ref(false);
const settings = ref({ interval: 30, enabled: true });
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

const statusIcon = computed(() => ({ ok: '✅', warning: '⚠️', overdue: '🚨', disabled: '⚪' }[dmsState.value]));
const statusTitle = computed(() => ({
  ok: '一切安好', warning: '即将需要确认', overdue: '已超过检查时间', disabled: 'Dead Man\'s Switch 未启用',
}[dmsState.value]));
const statusSubtitle = computed(() => {
  const days = daysUntilDue.value;
  if (dmsState.value === 'ok') {
    return days === null || days === undefined
      ? '已开启，等待首次确认后开始计算周期'
      : `距离下次检查还有 ${days} 天`;
  }
  if (dmsState.value === 'warning') {
    return days === null || days === undefined
      ? '请及时确认安全'
      : `仅剩 ${days} 天，请及时确认安全`;
  }
  return {
    overdue: '请立即确认你的安全状态',
    disabled: '前往「我的档案」页面开启此功能',
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
    toast('设置已保存');
  } catch (e) {
    toast('设置保存失败：' + String(e), 'error');
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

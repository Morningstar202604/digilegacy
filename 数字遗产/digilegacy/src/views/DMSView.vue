<template>
  <div class="dms fade-in">
    <h2 style="font-size: 22px; font-weight: 700; margin-bottom: 8px;">⏰ Dead Man's Switch · 安心传承</h2>
    <p class="text-muted mb-32" style="font-size: 14px;">确认你还安好。如果超出一段时间未确认，系统将按设定通知你关心的人。</p>

    <!-- 状态卡片 -->
    <div class="card mb-32" :class="statusClass">
      <div class="flex items-center gap-12">
        <div style="font-size: 48px;">{{ statusIcon }}</div>
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
          <label class="form-label">检查间隔</label>
          <select class="form-select" v-model.number="settings.interval">
            <option :value="7">每周确认一次</option>
            <option :value="14">每两周确认一次</option>
            <option :value="30">每月确认一次</option>
            <option :value="90">每季度确认一次</option>
          </select>
        </div>
        <button class="btn btn-primary btn-sm mt-16" @click="saveSettings">保存设置</button>
      </div>
    </div>

    <!-- 三级确认说明 -->
    <div class="card">
      <div class="card-title mb-16">🔄 三级确认机制</div>
      <div class="dms-stages">
        <div class="dms-stage dms-stage-1">
          <span class="dms-stage-num">1️⃣</span>
          <div><strong>第一阶段（到期后3天）</strong><p class="text-muted">发送邮件和短信提醒你本人确认安全</p></div>
        </div>
        <div class="dms-stage dms-stage-2">
          <span class="dms-stage-num">2️⃣</span>
          <div><strong>第二阶段（再7天后）</strong><p class="text-muted">通知紧急联系人确认你的状态</p></div>
        </div>
        <div class="dms-stage dms-stage-3">
          <span class="dms-stage-num">3️⃣</span>
          <div><strong>第三阶段（再14天后）</strong><p class="text-muted">向遗产执行人发送"数字遗产密钥信封"</p></div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useDashboardStore } from '../stores/dashboard';

const dashboard = useDashboardStore();
const checking = ref(false);
const settings = ref({ interval: 30, enabled: true });

const status = computed(() => dashboard.checkInStatus);
const lastCheckIn = computed(() => status.value?.last_check_in);
const daysUntilDue = computed(() => status.value?.days_until_due);

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
const statusSubtitle = computed(() => ({
  ok: `距离下次检查还有 ${daysUntilDue.value} 天`,
  warning: `仅剩 ${daysUntilDue.value} 天，请及时确认安全`,
  overdue: '请立即确认你的安全状态',
  disabled: '前往「我的档案」页面开启此功能',
}[dmsState.value]));
const statusColor = computed(() => ({
  ok: 'var(--success)', warning: 'var(--warning)', overdue: 'var(--danger)', disabled: 'var(--gray-500)',
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
  await dashboard.configureDms(settings.value.enabled, settings.value.interval);
  alert('设置已保存');
}

onMounted(async () => {
  await dashboard.loadCheckInStatus();
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
.dms-stage-1 { background: var(--success-light); }
.dms-stage-2 { background: var(--warning-light); }
.dms-stage-3 { background: var(--danger-light); }
.dms-stage-num { font-size: 24px; }
</style>

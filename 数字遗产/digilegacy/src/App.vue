<template>
  <div class="app-container">
    <!-- 侧边栏 -->
    <aside class="sidebar">
      <div class="sidebar-header">
        <div class="sidebar-logo">
          <svg class="logo-mark" viewBox="0 0 40 40" fill="none" xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
            <rect x="1" y="1" width="38" height="38" rx="11" fill="url(#digilegacy-logo)"/>
            <circle cx="20" cy="20" r="9.5" stroke="#fff" stroke-width="2.2" opacity="0.95"/>
            <circle cx="20" cy="20" r="3.2" fill="#fff"/>
            <defs>
              <linearGradient id="digilegacy-logo" x1="1" y1="1" x2="39" y2="39" gradientUnits="userSpaceOnUse">
                <stop stop-color="#c56a58"/>
                <stop offset="1" stop-color="#9a473b"/>
              </linearGradient>
            </defs>
          </svg>
          <div class="logo-wordmark">
            <span class="logo-name">归零</span>
            <span class="logo-sub">DigiLegacy</span>
          </div>
        </div>
      </div>
      
      <nav class="sidebar-nav">
        <router-link 
          v-for="route in navRoutes" 
          :key="route.path"
          :to="route.path"
          class="nav-item"
          :class="{ active: currentRoute === route.path }"
          :data-tour="'nav-' + route.path.slice(1)"
        >
          <span class="nav-icon">{{ route.icon }}</span>
          <span>{{ route.title }}</span>
        </router-link>
      </nav>
      
      <div class="sidebar-footer">
        <router-link v-if="!security.hasMasterPassword" to="/profile" style="color: var(--warning); text-decoration: none;">⚠️ 未启用加密 · 点此设置主密码</router-link>
        <span v-else>数据本地加密存储 · 已启用主密码</span>
      </div>
    </aside>

    <!-- 主内容区 -->
    <div class="main-layout">
      <header class="main-header">
        <div class="header-title">{{ currentPageTitle }}</div>
        <div class="header-actions">
          <button class="btn btn-sm btn-secondary" data-tour="help" @click="showHelp = true">❓ 帮助</button>
          <button class="btn btn-sm btn-secondary" v-if="security.hasMasterPassword && security.unlocked" @click="handleLock">🔒 锁定</button>
          <button class="btn btn-sm" data-tour="checkin" :class="checkBtnClass" @click="handleCheckIn">
            {{ checkBtnText }}
          </button>
        </div>
      </header>
      
      <main class="main-content">
        <div class="content-wrap">
          <router-view v-slot="{ Component }">
            <transition name="fade" mode="out-in">
              <component :is="Component" />
            </transition>
          </router-view>
        </div>
      </main>
    </div>

    <!-- 锁定遮罩 -->
    <div v-if="security.hasMasterPassword && !security.unlocked" class="lock-overlay">
      <div class="lock-card">
        <template v-if="!recoveryMode">
          <div style="font-size: 40px;">🔒</div>
          <h2 style="font-size: 18px; font-weight: 700; margin-top: 8px;">应用已锁定</h2>
          <p class="text-muted" style="font-size: 13px; margin-top: 8px;">输入主密码解锁。数据以 AES-256-GCM 加密存储于本机，开发者无法读取。</p>
          <input class="form-input" type="password" v-model="unlockPassword" placeholder="主密码" style="margin-top: 16px;" @keyup.enter="handleUnlock" />
          <p v-if="unlockError" style="color: var(--danger); font-size: 13px; margin-top: 8px;">{{ unlockError }}</p>
          <button class="btn btn-primary" style="margin-top: 16px; width: 100%;" :disabled="security.busy" @click="handleUnlock">{{ security.busy ? '验证中...' : '解锁' }}</button>
          <button class="btn btn-secondary btn-sm" style="margin-top: 10px; width: 100%;" @click="recoveryMode = true">忘记主密码？用恢复密钥重设</button>
          <p class="text-muted" style="font-size: 12px; margin-top: 12px;">主密码只用于本机解密；恢复密钥是设置时生成的一次性代码</p>
        </template>
        <template v-else>
          <div style="font-size: 40px;">🔑</div>
          <h2 style="font-size: 18px; font-weight: 700; margin-top: 8px;">恢复访问</h2>
          <p class="text-muted" style="font-size: 13px; margin-top: 8px;">输入设置主密码时保存的恢复密钥（形如 XXXX-XXXX-XXXX-XXXX-XXXX-XXXX）。</p>
          <input class="form-input" v-model="recoveryCode" placeholder="恢复密钥" style="margin-top: 16px;" @keyup.enter="handleStartRecovery" />
          <template v-if="recoveryUnlocked">
            <input class="form-input" type="password" v-model="recoveryNewPassword" placeholder="新主密码（至少 8 位）" style="margin-top: 12px;" />
            <input class="form-input" type="password" v-model="recoveryNewPassword2" placeholder="确认新主密码" style="margin-top: 12px;" />
            <p v-if="recoveryMessage" style="font-size: 13px; margin-top: 8px;" :style="{ color: recoveryOk ? 'var(--success)' : 'var(--danger)' }" class="recovery-msg">{{ recoveryMessage }}</p>
            <button class="btn btn-primary" style="margin-top: 12px; width: 100%;" :disabled="recoveryBusy" @click="handleCompleteRecovery">{{ recoveryBusy ? '处理中...' : '重设主密码' }}</button>
            <p v-if="recoveryNewCode" class="recovery-code-box">新的恢复密钥（立即抄写保存）：{{ recoveryNewCode }}</p>
          </template>
          <p v-else-if="recoveryError" style="color: var(--danger); font-size: 13px; margin-top: 8px;">{{ recoveryError }}</p>
          <button class="btn btn-secondary btn-sm" style="margin-top: 10px; width: 100%;" @click="backToUnlock">返回主密码解锁</button>
        </template>
      </div>
    </div>

    <!-- 新手引导与帮助中心 -->
    <OnboardingTour v-if="showTour" @close="showTour = false" />
    <HelpDrawer v-if="showHelp" @close="showHelp = false" @replay-tour="startTour" />
  </div>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue';
import { useRoute } from 'vue-router';
import { invoke } from './api/invoke';

import { useDashboardStore } from './stores/dashboard';
import { useSecurityStore } from './stores/security';
import { useAssetsStore } from './stores/assets';
import { startLocalReminderLoop } from './api/reminders';
import OnboardingTour from './components/OnboardingTour.vue';
import HelpDrawer from './components/HelpDrawer.vue';
import type { ApiResponse, SecurityStatus } from './types';

interface NavRoute {
  path: string;
  title: string;
  icon: string;
}

const route = useRoute();
const dashboard = useDashboardStore();
const security = useSecurityStore();

const unlockPassword = ref('');
const unlockError = ref('');
const showTour = ref(false);
const showHelp = ref(false);
let stopReminders: (() => void) | null = null;

function startTour() {
  showHelp.value = false;
  showTour.value = true;
}

const IDLE_LIMIT_MS = 10 * 60 * 1000;
let lastActivity = Date.now();
let idleTimer: number | undefined;

function bumpActivity() {
  lastActivity = Date.now();
}

function registerActivityWatch() {
  ['mousemove', 'keydown', 'click', 'wheel', 'touchstart'].forEach(ev =>
    window.addEventListener(ev, bumpActivity, { passive: true })
  );
  idleTimer = window.setInterval(() => {
    if (security.hasMasterPassword && security.unlocked && Date.now() - lastActivity > IDLE_LIMIT_MS) {
      handleLock();
    }
  }, 30_000);
}

function unregisterActivityWatch() {
  ['mousemove', 'keydown', 'click', 'wheel', 'touchstart'].forEach(ev =>
    window.removeEventListener(ev, bumpActivity)
  );
  if (idleTimer !== undefined) {
    window.clearInterval(idleTimer);
  }
}

async function handleUnlock() {
  unlockError.value = '';
  if (!unlockPassword.value) {
    unlockError.value = '请输入主密码';
    return;
  }
  const err = await security.unlockWith(unlockPassword.value);
  if (err) {
    unlockError.value = err;
  } else {
    unlockPassword.value = '';
    bumpActivity();
    await refreshSensitiveData();
  }
}

async function refreshSensitiveData() {
  const assetsStore = useAssetsStore();
  await Promise.all([assetsStore.loadAssets(), dashboard.loadUser()]);
}

function wipeSensitiveData() {
  const assetsStore = useAssetsStore();
  assetsStore.assets = [];
  assetsStore.currentAsset = null;
  dashboard.user = null;
}

async function handleLock() {
  await security.lockNow();
  wipeSensitiveData();
  bumpActivity();
}

const recoveryMode = ref(false);
const recoveryCode = ref('');
const recoveryError = ref('');
const recoveryUnlocked = ref(false);
const recoveryNewPassword = ref('');
const recoveryNewPassword2 = ref('');
const recoveryMessage = ref('');
const recoveryOk = ref(false);
const recoveryBusy = ref(false);
const recoveryNewCode = ref('');

async function handleStartRecovery() {
  recoveryError.value = '';
  if (!recoveryCode.value.trim()) {
    recoveryError.value = '请输入恢复密钥';
    return;
  }
  try {
    const res = await invoke<ApiResponse<SecurityStatus>>('start_recovery', { recoveryCode: recoveryCode.value });
    if (res.success) {
      recoveryUnlocked.value = true;
      recoveryMessage.value = '';
      security.unlocked = false;
    } else {
      recoveryError.value = res.message;
    }
  } catch (e) {
    recoveryError.value = String(e);
  }
}

async function handleCompleteRecovery() {
  recoveryMessage.value = '';
  recoveryBusy.value = true;
  try {
    const res = await invoke<ApiResponse<string>>('complete_recovery', {
      newPassword: recoveryNewPassword.value,
      confirmPassword: recoveryNewPassword2.value,
    });
    recoveryOk.value = res.success;
    recoveryMessage.value = res.message;
    if (res.success && res.data) {
      recoveryNewCode.value = res.data;
      recoveryUnlocked.value = false;
      recoveryCode.value = '';
      recoveryNewPassword.value = '';
      recoveryNewPassword2.value = '';
      window.setTimeout(() => {
        security.refresh();
        recoveryMode.value = false;
        recoveryNewCode.value = '';
        recoveryMessage.value = '';
        refreshSensitiveData();
      }, 6000);
    }
  } catch (e) {
    recoveryOk.value = false;
    recoveryMessage.value = String(e);
  } finally {
    recoveryBusy.value = false;
  }
}

function backToUnlock() {
  recoveryMode.value = false;
  recoveryUnlocked.value = false;
  recoveryError.value = '';
  recoveryMessage.value = '';
}

const navRoutes: NavRoute[] = [
  { path: '/dashboard', title: '仪表板', icon: '📊' },
  { path: '/discover', title: '资产发现', icon: '🔍' },
  { path: '/assets', title: '我的资产', icon: '💎' },
  { path: '/profile', title: '我的档案', icon: '👤' },
  { path: '/dms', title: '安心传承', icon: '⏰' },
  { path: '/report', title: '继承报告', icon: '📄' },
  { path: '/heir', title: '继承人向导', icon: '👥' },
  { path: '/legal', title: '法律模板', icon: '📜' },
];

const currentRoute = computed(() => route.path);
const currentPageTitle = computed(() => {
  return navRoutes.find(r => r.path === route.path)?.title || (route.meta?.title as string) || '归零';
});

const checkBtnText = computed(() => {
  const status = dashboard.checkInStatus;
  if (!status) return '检查中...';
  if (!status.dms_enabled) return 'DMS未启用';
  const days = status.days_until_due;
  if (days === null) return '点击检查';
  if (days <= 0) return '⚠️ 请确认安全';
  if (days <= 3) return `${days}天后需检查`;
  return `✓ ${days}天后检查`;
});

const checkBtnClass = computed(() => {
  const status = dashboard.checkInStatus;
  if (!status?.dms_enabled) return 'btn-secondary';
  if ((status.days_until_due ?? 999) <= 0) return 'btn-danger';
  if ((status.days_until_due ?? 999) <= 3) return 'btn-warning';
  return 'btn-success';
});

async function handleCheckIn() {
  await dashboard.checkIn();
}

onMounted(() => {
  security.refresh();
  registerActivityWatch();
  dashboard.loadCheckInStatus();
  dashboard.evaluateDms();
  stopReminders = startLocalReminderLoop();
  window.setTimeout(() => {
    const locked = security.hasMasterPassword && !security.unlocked;
    if (!locked && !localStorage.getItem('digilegacy_onboarding_done')) {
      showTour.value = true;
    }
  }, 900);
});

onBeforeUnmount(() => {
  unregisterActivityWatch();
  if (stopReminders) stopReminders();
});
</script>

<style scoped>
.fade-enter-active, .fade-leave-active {
  transition: opacity 0.15s ease, transform 0.15s ease;
}
.fade-enter-from { opacity: 0; transform: translateY(8px); }
.fade-leave-to { opacity: 0; transform: translateY(-8px); }

.lock-overlay {
  position: fixed;
  inset: 0;
  background: rgba(40, 34, 28, 0.55);
  backdrop-filter: blur(6px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}
.lock-card {
  background: var(--gray-0, #fffdfa);
  border-radius: 16px;
  padding: 32px;
  width: 380px;
  max-width: calc(100vw - 48px);
  text-align: center;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.25);
  max-height: calc(100vh - 48px);
  overflow-y: auto;
}
.recovery-msg {
  word-break: break-all;
}
.recovery-code-box {
  margin-top: 12px;
  padding: 10px 12px;
  border: 1px dashed var(--warning, #b7791f);
  border-radius: 8px;
  background: rgba(183, 121, 31, 0.08);
  font-size: 13px;
  word-break: break-all;
}
</style>

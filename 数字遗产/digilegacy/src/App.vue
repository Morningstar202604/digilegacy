<template>
  <div class="app-container">
    <!-- 侧边栏 -->
    <aside class="sidebar">
      <div class="sidebar-header">
        <div class="sidebar-logo">
          <span>归</span>
          <div>
            <div>归零</div>
            <div class="sidebar-subtitle">DigiLegacy</div>
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
        >
          <span class="nav-icon">{{ route.icon }}</span>
          <span>{{ route.title }}</span>
        </router-link>
      </nav>
      
      <div class="sidebar-footer">
        数据本地加密存储 · 隐私优先
      </div>
    </aside>

    <!-- 主内容区 -->
    <div class="main-layout">
      <header class="main-header">
        <div class="header-title">{{ currentPageTitle }}</div>
        <div class="header-actions">
          <button class="btn btn-sm" :class="checkBtnClass" @click="handleCheckIn">
            {{ checkBtnText }}
          </button>
        </div>
      </header>
      
      <main class="main-content">
        <router-view v-slot="{ Component }">
          <transition name="fade" mode="out-in">
            <component :is="Component" />
          </transition>
        </router-view>
      </main>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted } from 'vue';
import { useRoute } from 'vue-router';

import { useDashboardStore } from './stores/dashboard';

interface NavRoute {
  path: string;
  title: string;
  icon: string;
}

const route = useRoute();
const dashboard = useDashboardStore();

const navRoutes: NavRoute[] = [
  { path: '/dashboard', title: '仪表板', icon: '📊' },
  { path: '/discover', title: '资产发现', icon: '🔍' },
  { path: '/assets', title: '我的资产', icon: '💎' },
  { path: '/profile', title: '我的档案', icon: '👤' },
  { path: '/dms', title: '安心传承', icon: '⏰' },
  { path: '/report', title: '继承报告', icon: '📄' },
  { path: '/heir', title: '继承人向导', icon: '👥' },
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
  dashboard.loadCheckInStatus();
});
</script>

<style scoped>
.fade-enter-active, .fade-leave-active {
  transition: opacity 0.15s ease, transform 0.15s ease;
}
.fade-enter-from { opacity: 0; transform: translateY(8px); }
.fade-leave-to { opacity: 0; transform: translateY(-8px); }
</style>

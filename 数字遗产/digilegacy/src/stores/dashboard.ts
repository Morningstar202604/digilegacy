import { defineStore } from 'pinia';
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';

import type { DashboardStats, CheckInStatus, UserProfile, ApiResponse } from '../types';

export const useDashboardStore = defineStore('dashboard', () => {
  const stats = ref<DashboardStats | null>(null);
  const checkInStatus = ref<CheckInStatus | null>(null);
  const user = ref<UserProfile | null>(null);
  const loading = ref(false);

  async function loadStats() {
    const res = await invoke<ApiResponse<DashboardStats>>('get_dashboard_stats');
    if (res.success && res.data) {
      stats.value = res.data;
    }
  }

  async function loadCheckInStatus() {
    const res = await invoke<ApiResponse<CheckInStatus>>('get_check_in_status');
    if (res.success && res.data) {
      checkInStatus.value = res.data;
    }
  }

  async function loadUser() {
    const res = await invoke<ApiResponse<UserProfile | null>>('get_user_profile');
    if (res.success) {
      user.value = res.data || null;
    }
  }

  async function checkIn() {
    const res = await invoke<ApiResponse<string>>('update_check_in');
    if (res.success) {
      await Promise.all([loadStats(), loadCheckInStatus()]);
    }
    return res;
  }

  async function saveUser(profile: UserProfile) {
    const res = await invoke<ApiResponse<void>>('set_user_profile', { profile });
    if (res.success) {
      user.value = { ...profile, ...user.value };
    }
    return res;
  }

  async function configureDms(enabled: boolean, intervalDays: number) {
    const res = await invoke<ApiResponse<void>>('set_dead_mans_switch', {
      enabled,
      intervalDays,
    });
    if (res.success) {
      await loadCheckInStatus();
    }
    return res;
  }

  async function refreshAll() {
    loading.value = true;
    try {
      await Promise.all([loadStats(), loadCheckInStatus(), loadUser()]);
    } finally {
      loading.value = false;
    }
  }

  return {
    stats,
    checkInStatus,
    user,
    loading,
    loadStats,
    loadCheckInStatus,
    loadUser,
    checkIn,
    saveUser,
    configureDms,
    refreshAll,
  };
});

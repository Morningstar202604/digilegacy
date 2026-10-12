<template>
  <div class="profile fade-in">
    <h2 style="font-size: 22px; font-weight: 700; margin-bottom: 24px;">👤 我的档案</h2>
    <p class="text-muted mb-32" style="font-size: 14px;">完善你的个人信息，帮助继承人顺利找到并继承你的数字遗产。</p>

    <div class="grid grid-2">
      <div class="card">
        <div class="card-title mb-16">📝 基本信息</div>
        <div class="form-group">
          <label class="form-label">姓名</label>
          <input class="form-input" v-model="profile.full_name" placeholder="你的姓名" />
        </div>
        <div class="form-group">
          <label class="form-label">手机号</label>
          <input class="form-input" v-model="profile.phone" placeholder="手机号码" />
        </div>
        <div class="form-group">
          <label class="form-label">邮箱</label>
          <input class="form-input" v-model="profile.email" placeholder="电子邮箱" />
        </div>
      </div>

      <div class="card">
        <div class="card-title mb-16">🚨 紧急联系人</div>
        <div class="form-group">
          <label class="form-label">姓名</label>
          <input class="form-input" v-model="profile.emergency_contact_name" placeholder="紧急联系人姓名" />
        </div>
        <div class="form-group">
          <label class="form-label">电话</label>
          <input class="form-input" v-model="profile.emergency_contact_phone" placeholder="联系电话" />
        </div>
        <div class="form-group">
          <label class="form-label">关系</label>
          <input class="form-input" v-model="profile.emergency_contact_relation" placeholder="如：子女、配偶" />
        </div>
      </div>

      <div class="card">
        <div class="card-title mb-16">👑 主要继承人</div>
        <div class="form-group">
          <label class="form-label">姓名</label>
          <input class="form-input" v-model="profile.heir_primary_name" placeholder="继承人姓名" />
        </div>
        <div class="form-group">
          <label class="form-label">电话</label>
          <input class="form-input" v-model="profile.heir_primary_phone" placeholder="联系电话" />
        </div>
        <div class="form-group">
          <label class="form-label">关系</label>
          <input class="form-input" v-model="profile.heir_primary_relation" placeholder="如：子女、父母" />
        </div>
      </div>

      <div class="card">
        <div class="card-title mb-16">⏰ Dead Man's Switch</div>
        <div class="form-group">
          <label class="form-label">检查间隔</label>
          <select class="form-select" v-model.number="profile.check_in_interval_days">
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

    <div class="card mt-24" style="text-align: center;">
      <button class="btn btn-primary btn-lg" @click="saveProfile">💾 保存档案</button>
      <p class="text-muted mt-16" style="font-size: 12px;">所有数据均 AES-256 加密保存在本地，不上传任何服务器</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';

import { useDashboardStore } from '../stores/dashboard';
import type { UserProfile } from '../types';

const dashboard = useDashboardStore();

const profile = ref<UserProfile>({
  full_name: '', phone: '', email: '', id_last4: '',
  emergency_contact_name: '', emergency_contact_phone: '', emergency_contact_relation: '',
  heir_primary_name: '', heir_primary_phone: '', heir_primary_relation: '',
  check_in_interval_days: 30, last_check_in: null, dms_enabled: false,
  created_at: '', updated_at: '',
});

onMounted(async () => {
  await dashboard.loadUser();
  if (dashboard.user) {
    profile.value = { ...profile.value, ...dashboard.user };
  }
});

async function saveProfile() {
  await dashboard.saveUser(profile.value);
  await dashboard.configureDms(profile.value.dms_enabled, profile.value.check_in_interval_days);
  alert('档案保存成功！');
}
</script>

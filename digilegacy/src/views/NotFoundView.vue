<template>
  <div class="not-found fade-in">
    <div class="card text-center" style="padding: 56px 32px;">
      <EmptyState kind="search" :size="180" />
      <h2 style="font-size: 22px; margin: 20px 0 8px;">这个页面不存在</h2>
      <p class="text-muted mb-24" style="font-size: 14px;">
        地址可能拼错、页面已迁移，或这是从旧版本留下的链接。
      </p>
      <div class="flex gap-12" style="justify-content: center; flex-wrap: wrap;">
        <router-link to="/dashboard" class="btn btn-primary">回到仪表板</router-link>
        <button class="btn btn-secondary" @click="goBack">返回上一页</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useRouter } from 'vue-router';
import EmptyState from '../components/EmptyState.vue';

const router = useRouter();

function goBack() {
  if (window.history.length > 1) router.back();
  else router.push('/dashboard');
}
</script>

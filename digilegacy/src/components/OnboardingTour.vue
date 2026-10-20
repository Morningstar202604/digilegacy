<template>
  <div class="tour-root">
    <!-- 遮罩可点击关闭：避免用户找不到出口时被"困"在引导里 -->
    <div class="tour-blocker" @click="finish" title="点击关闭引导" aria-label="点击关闭引导"></div>

    <div v-if="rect" class="tour-spotlight" :style="spotlightStyle"></div>

    <div class="tour-card" ref="cardRef" tabindex="-1" :style="cardStyle" role="dialog" aria-modal="true" aria-labelledby="tour-title" @click.stop>
      <div class="tour-progress" aria-hidden="true">
        <span v-for="(_, i) in steps" :key="i" class="tour-dot" :class="{ active: i === index }"></span>
      </div>
      <!-- 切步时读屏自动播报（4.1.3 状态消息） -->
      <div aria-live="polite">
        <div class="tour-title" id="tour-title">{{ current.title }}</div>
        <div class="tour-desc">{{ current.desc }}</div>
      </div>
      <div class="tour-actions">
        <button class="btn btn-secondary btn-sm" @click="skip">跳过</button>
        <div style="display: flex; gap: 8px;">
          <button v-if="index > 0" class="btn btn-secondary btn-sm" @click="prev">上一步</button>
          <button class="btn btn-primary btn-sm" @click="next">{{ index === steps.length - 1 ? '完成' : '下一步' }}</button>
        </div>
      </div>
      <div class="tour-step-text">{{ index + 1 }} / {{ steps.length }}</div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue';

interface TourStep {
  target: string | null;
  title: string;
  desc: string;
}

const emit = defineEmits<{ (e: 'close'): void }>();

const cardRef = ref<HTMLElement | null>(null);
let previouslyFocused: HTMLElement | null = null;

const steps: TourStep[] = [
  { target: null, title: '欢迎使用「归零」', desc: '30 秒带你认识主要功能，之后的每一步都可以点「跳过」。' },
  { target: '[data-tour="nav-discover"]', title: '① 资产发现', desc: '回答一份自检问卷，从 152 个平台库里找出你可能已经遗忘的账号。' },
  { target: '[data-tour="nav-assets"]', title: '② 我的资产', desc: '确认并登记资产；点开任意资产，可以加密保存账号凭据、跳转平台官网。' },
  { target: '[data-tour="nav-profile"]', title: '③ 我的档案', desc: '填写本人与继承人信息；在这里设置主密码启用加密，也可以导出加密备份。' },
  { target: '[data-tour="nav-dms"]', title: '④ 安心传承', desc: '定期点一下「我还安好」。万一长时间未确认，会按三级确认链通知你关心的人。' },
  { target: '[data-tour="nav-report"]', title: '⑤ 继承报告', desc: '一键生成继承方案，导出 HTML 报告与工具包 ZIP 交给家人。' },
  { target: '[data-tour="nav-heir"]', title: '⑥ 继承人向导', desc: '家人按平台查办理流程、所需材料与变现过户路径。' },
  { target: '[data-tour="checkin"]', title: '顶部快捷确认', desc: '不想进页面？点这里的按钮就能一键完成安全确认。' },
  { target: '[data-tour="help"]', title: '需要帮助？', desc: '点「帮助」随时打开帮助中心，也可以在里面重新播放本引导。祝使用顺利！' },
];

const index = ref(0);
const rect = ref<DOMRect | null>(null);

const current = computed(() => steps[index.value]);

const spotlightStyle = computed(() => {
  if (!rect.value) return {};
  const pad = 6;
  return {
    left: `${rect.value.left - pad}px`,
    top: `${rect.value.top - pad}px`,
    width: `${rect.value.width + pad * 2}px`,
    height: `${rect.value.height + pad * 2}px`,
  };
});

const cardStyle = computed(() => {
  const CARD_W = 340;
  if (!rect.value) {
    return { left: '50%', top: '50%', transform: 'translate(-50%, -50%)' };
  }
  const r = rect.value;
  const vw = window.innerWidth;
  const vh = window.innerHeight;
  if (r.left < 320) {
    return { left: `${r.right + 16}px`, top: `${Math.min(r.top, vh - 260)}px` };
  }
  if (r.bottom + 240 < vh) {
    return { left: `${Math.max(16, Math.min(r.left, vw - CARD_W - 16))}px`, top: `${r.bottom + 14}px` };
  }
  return { left: `${Math.max(16, Math.min(r.left, vw - CARD_W - 16))}px`, top: `${Math.max(16, r.top - 230)}px` };
});

function measure() {
  const step = steps[index.value];
  if (!step.target) {
    rect.value = null;
    return;
  }
  const el = document.querySelector(step.target);
  rect.value = el ? el.getBoundingClientRect() : null;
}

function next() {
  if (index.value >= steps.length - 1) {
    finish();
    return;
  }
  index.value += 1;
  measure();
}

function prev() {
  if (index.value > 0) {
    index.value -= 1;
    measure();
  }
}

function skip() {
  finish();
}

function finish() {
  localStorage.setItem('digilegacy_onboarding_done', '1');
  emit('close');
}

function onKey(e: KeyboardEvent) {
  if (e.key === 'Escape') finish();
  if (e.key === 'ArrowRight' || e.key === 'Enter') next();
  if (e.key === 'ArrowLeft') prev();
}

onMounted(() => {
  measure();
  window.addEventListener('resize', measure);
  window.addEventListener('keydown', onKey);
  // 焦点移入引导卡片（tabindex=-1 容器，避免 Enter 触发按钮与全局快捷键双重响应）；
  // 关闭时归还焦点给打开引导的按钮
  previouslyFocused = document.activeElement as HTMLElement | null;
  cardRef.value?.focus();
});

onBeforeUnmount(() => {
  window.removeEventListener('resize', measure);
  window.removeEventListener('keydown', onKey);
  previouslyFocused?.focus?.();
});
</script>

<style scoped>
.tour-root {
  position: fixed;
  inset: 0;
  z-index: 1200;
}
.tour-blocker {
  position: absolute;
  inset: 0;
}
.tour-spotlight {
  position: fixed;
  border-radius: 12px;
  box-shadow: 0 0 0 9999px rgba(32, 26, 21, 0.6);
  border: 2px solid var(--primary, #4a7c8c);
  pointer-events: none;
  transition: all 0.18s ease;
}
.tour-card {
  position: fixed;
  width: 340px;
  background: var(--gray-0, #fffdfa);
  border-radius: 14px;
  padding: 18px 20px 14px;
  box-shadow: 0 16px 48px rgba(0, 0, 0, 0.28);
  transition: left 0.18s ease, top 0.18s ease;
}
.tour-progress {
  display: flex;
  gap: 5px;
  margin-bottom: 10px;
}
.tour-dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background: var(--gray-200, #e8e4de);
}
.tour-dot.active {
  background: var(--primary, #4a7c8c);
}
.tour-title {
  font-size: 16px;
  font-weight: 700;
  color: var(--gray-900, #24323a);
}
.tour-desc {
  font-size: 13px;
  color: var(--gray-600, #5c6f79);
  line-height: 1.65;
  margin-top: 6px;
}
.tour-actions {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-top: 14px;
}
.tour-step-text {
  text-align: right;
  font-size: 11px;
  color: var(--gray-400, #9aa8b0);
  margin-top: 6px;
}
</style>

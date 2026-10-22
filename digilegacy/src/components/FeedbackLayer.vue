<template>
  <!-- 全局反馈层：Toast 堆叠在右下，确认框居中；两者都渲染在 body 之外的固定层 -->
  <div class="toast-stack" aria-live="polite" aria-atomic="false">
    <transition-group name="toast">
      <div
        v-for="t in toasts"
        :key="t.id"
        class="toast"
        :class="`toast-${t.kind}`"
        role="status"
      >
        <span class="toast-icon" aria-hidden="true">{{ icon(t.kind) }}</span>
        <span class="toast-msg">{{ t.message }}</span>
        <button class="toast-close" aria-label="关闭提示" @click="dismissToast(t.id)">×</button>
      </div>
    </transition-group>
  </div>

  <transition name="dialog">
    <div v-if="confirmState.open" class="confirm-overlay" @click.self="settleConfirm(false)">
      <div
        class="confirm-card"
        role="alertdialog"
        aria-modal="true"
        aria-labelledby="confirm-title"
        aria-describedby="confirm-desc"
        ref="cardRef"
        @keydown.esc.stop="settleConfirm(false)"
      >
        <h2 id="confirm-title" class="confirm-title">{{ confirmState.title }}</h2>
        <p v-if="confirmState.message" id="confirm-desc" class="confirm-desc">{{ confirmState.message }}</p>
        <div class="confirm-actions">
          <button class="btn btn-secondary" @click="settleConfirm(false)">{{ confirmState.cancelText }}</button>
          <button
            class="btn"
            :class="confirmState.danger ? 'btn-danger' : 'btn-primary'"
            ref="confirmBtnRef"
            @click="settleConfirm(true)"
          >{{ confirmState.confirmText }}</button>
        </div>
      </div>
    </div>
  </transition>
</template>

<script setup lang="ts">
import { nextTick, ref, watch } from 'vue';
import { dismissToast, settleConfirm, useFeedback } from '../composables/feedback';

const { toasts, confirmState } = useFeedback();
const cardRef = ref<HTMLElement | null>(null);
const confirmBtnRef = ref<HTMLElement | null>(null);

function icon(kind: string) {
  return kind === 'success' ? '✓' : kind === 'error' ? '!' : 'i';
}

// 打开确认框时把焦点移到主按钮（危险操作则先落在取消上，避免误触回车确认）
watch(
  () => confirmState.open,
  async (open) => {
    if (!open) return;
    await nextTick();
    confirmBtnRef.value?.focus();
  }
);
</script>

<style scoped>
.toast-stack {
  position: fixed;
  right: 20px;
  bottom: 20px;
  z-index: 9000;
  display: flex;
  flex-direction: column;
  gap: 10px;
  pointer-events: none;
}

.toast {
  pointer-events: auto;
  display: flex;
  align-items: center;
  gap: 10px;
  min-width: 260px;
  max-width: 400px;
  padding: 13px 15px;
  border-radius: var(--radius-sm);
  /* 实色不透明：半透明在浅色背景上会「糊」，白字对比度也不稳定 */
  background: var(--gray-900);
  color: #fff;
  box-shadow: 0 12px 32px rgba(35, 32, 25, 0.32), 0 2px 6px rgba(35, 32, 25, 0.18);
  font-size: 14px;
  font-weight: 500;
  line-height: 1.45;
  border: 1px solid rgba(255, 255, 255, 0.08);
}
.toast-success { background: var(--success-strong); border-color: rgba(255,255,255,0.14); }
.toast-error { background: var(--danger-strong); border-color: rgba(255,255,255,0.14); }
.toast-info { background: var(--gray-800); }

.toast-icon {
  flex: none;
  width: 20px;
  height: 20px;
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.26);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  font-size: 12px;
  font-weight: 800;
  line-height: 1;
}

.toast-msg { flex: 1; }

.toast-close {
  flex: none;
  border: 0;
  background: transparent;
  color: rgba(255, 255, 255, 0.72);
  font-size: 18px;
  line-height: 1;
  cursor: pointer;
  padding: 2px 4px;
  border-radius: 4px;
}
.toast-close:hover { color: #fff; background: rgba(255, 255, 255, 0.12); }
.toast-close:focus-visible { outline: 2px solid #fff; outline-offset: 1px; }

.toast-enter-active, .toast-leave-active { transition: all 0.24s cubic-bezier(0.16, 1, 0.3, 1); }
.toast-enter-from { opacity: 0; transform: translateY(12px) scale(0.97); }
.toast-leave-to { opacity: 0; transform: translateX(16px); }

.confirm-overlay {
  position: fixed;
  inset: 0;
  z-index: 9500;
  /* 只做轻微柔化：blur 过强会让背景文字完全不可读，反而增加认知负担 */
  background: rgba(35, 32, 25, 0.38);
  backdrop-filter: blur(1px);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 24px;
}

.confirm-card {
  width: 100%;
  max-width: 420px;
  background: #fff;
  border-radius: var(--radius);
  padding: 24px;
  box-shadow: 0 24px 64px rgba(35, 32, 25, 0.28);
}

.confirm-title { font-size: 17px; font-weight: 700; color: var(--gray-900); }
.confirm-desc { margin-top: 10px; font-size: 14px; line-height: 1.6; color: var(--gray-600); }
.confirm-actions { margin-top: 22px; display: flex; justify-content: flex-end; gap: 10px; }

.dialog-enter-active, .dialog-leave-active { transition: opacity 0.18s ease; }
.dialog-enter-active .confirm-card, .dialog-leave-active .confirm-card { transition: transform 0.18s cubic-bezier(0.16, 1, 0.3, 1); }
.dialog-enter-from, .dialog-leave-to { opacity: 0; }
.dialog-enter-from .confirm-card, .dialog-leave-to .confirm-card { transform: scale(0.96) translateY(6px); }
</style>

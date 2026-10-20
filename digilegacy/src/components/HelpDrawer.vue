<template>
  <div class="drawer-mask" aria-hidden="true" @click="emit('close')"></div>
  <aside class="help-drawer" role="dialog" aria-modal="true" aria-labelledby="help-title" ref="drawerRef">
    <div class="help-header">
      <h2 id="help-title" style="font-size: 17px; font-weight: 700; margin: 0;"><span aria-hidden="true">❓</span> 帮助中心</h2>
      <button class="btn btn-secondary btn-sm" ref="closeBtnRef" @click="emit('close')">关闭</button>
    </div>

    <div class="help-body">
      <div class="help-section">
        <div class="help-section-title">🚀 快速上手</div>
        <ol class="help-list">
          <li>「资产发现」回答自检问卷，找出你的所有账号，确认后进入「我的资产」</li>
          <li>在「我的档案」设置主密码（加密生效），填写本人与继承人信息</li>
          <li>点开每项资产，设置处置意愿；需要时加密保存账号凭据</li>
          <li>「安心传承」开启定期确认，每隔一段时间点一下「我还安好」</li>
          <li>「继承报告」导出报告与工具包 ZIP，交给家人保管</li>
        </ol>
        <button class="btn btn-secondary btn-sm" @click="emit('replay-tour')">重新播放新手引导</button>
      </div>

      <div class="help-section">
        <div class="help-section-title">🔐 安全与加密</div>
        <ul class="help-list">
          <li>设置主密码后，账号、凭据、档案等敏感字段以 AES-256-GCM 加密存储在本机，开发者无法读取</li>
          <li>支持手动锁定与闲置 10 分钟自动锁定；复制凭据 30 秒后尝试自动清除剪贴板</li>
          <li>换电脑用「我的档案 → 加密备份」导出备份，导入后用同一主密码解锁</li>
          <li>设置主密码时会生成一次性的 24 位恢复密钥（忘记主密码时凭它重设密码找回数据），修改主密码会自动轮换，请同步更新手抄记录</li>
        </ul>
      </div>

      <div class="help-section">
        <div class="help-section-title">⏰ 安心传承是怎么提醒的</div>
        <ul class="help-list">
          <li>应用运行期间每 30 分钟自动检查一次：距确认到期 3 天内或已逾期，会弹出系统通知（每天至多一条）</li>
          <li>逾期后按三级确认链推进：提醒你本人 → 通知紧急联系人 → 向遗产执行人发送数字遗产继承指引（不含任何账号凭据）</li>
          <li>每条通知都会写入本地发件箱（应用数据目录 `dms_outbox`），另行配置邮件环境变量时可同时发邮件</li>
          <li>可选开启云端心跳（零内容）：应用每 30 分钟向自托管服务器上报一次令牌，服务器只在长时间收不到时按预注册联系方式提醒，接触不到任何用户数据</li>
          <li>短信与跨设备推送仍属后续云端计划：心跳服务器参考实现已就绪（server/ 目录）</li>
        </ul>
      </div>

      <div class="help-section">
        <div class="help-section-title">🧭 常见问题</div>
        <ul class="help-list">
          <li><strong>它会上传我的数据吗？</strong>不会。应用无服务器、无遥测；「打开官网」只是跳转浏览器</li>
          <li><strong>Web 预览是什么？</strong>开发时在浏览器看界面的演示模式（数据为演示数据，存于浏览器本地）</li>
          <li><strong>报告里会有密码吗？</strong>不会。凭据不会进入继承报告与工具包</li>
          <li><strong>资金类平台注意什么？</strong>建议只保存查询/登录信息，不要保存银行卡取款密码（PIN）</li>
        </ul>
      </div>

      <div class="help-section about-section">
        <div class="help-section-title">ℹ️ 关于</div>
        <p class="about-text">
          归零 DigiLegacy · 数字遗产管理及继承系统 <span class="about-ver">v{{ appVersion }}</span><br>
          开源（MIT 许可）· 本地优先 · 无服务器、无遥测<br>
          你的数据只属于你自己：即使我们（开发者）也无法读取。
        </p>
        <p class="about-shortcut">快捷键：Esc 关闭本面板 · 引导中 ← → 切换步骤</p>
      </div>
    </div>
  </aside>
</template>

<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref } from 'vue';

const emit = defineEmits<{ (e: 'close'): void; (e: 'replay-tour'): void }>();

const appVersion = __APP_VERSION__;
const drawerRef = ref<HTMLElement | null>(null);
const closeBtnRef = ref<HTMLElement | null>(null);
let previouslyFocused: HTMLElement | null = null;

function onKey(e: KeyboardEvent) {
  if (e.key === 'Escape') emit('close');
}

// Tab 困留在面板内：aria-modal 面板不允许键盘穿透到背景内容
function trapTab(e: KeyboardEvent) {
  if (e.key !== 'Tab') return;
  const panel = drawerRef.value;
  if (!panel) return;
  const focusables = Array.from(
    panel.querySelectorAll<HTMLElement>('button:not([disabled]), input, [href], textarea, select')
  );
  if (focusables.length === 0) return;
  const active = document.activeElement as HTMLElement | null;
  if (!active || !panel.contains(active)) {
    e.preventDefault();
    focusables[0].focus();
    return;
  }
  const idx = focusables.indexOf(active);
  let next = idx + (e.shiftKey ? -1 : 1);
  if (next < 0) next = focusables.length - 1;
  if (next >= focusables.length) next = 0;
  e.preventDefault();
  focusables[next].focus();
}

onMounted(() => {
  window.addEventListener('keydown', onKey);
  document.addEventListener('keydown', trapTab, true);
  // 打开时焦点移入面板；关闭时归还给触发按钮
  previouslyFocused = document.activeElement as HTMLElement | null;
  closeBtnRef.value?.focus();
});
onBeforeUnmount(() => {
  window.removeEventListener('keydown', onKey);
  document.removeEventListener('keydown', trapTab, true);
  previouslyFocused?.focus?.();
});
</script>

<style scoped>
.drawer-mask {
  position: fixed;
  inset: 0;
  background: rgba(32, 26, 21, 0.35);
  z-index: 1100;
}
.help-drawer {
  position: fixed;
  top: 0;
  right: 0;
  bottom: 0;
  width: 430px;
  max-width: calc(100vw - 48px);
  background: var(--gray-0, #fffdfa);
  z-index: 1110;
  box-shadow: -12px 0 40px rgba(0, 0, 0, 0.18);
  display: flex;
  flex-direction: column;
}
.help-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 18px 20px;
  border-bottom: 1px solid var(--gray-100, #f0ece6);
}
.help-body {
  padding: 16px 20px 28px;
  overflow-y: auto;
}
.help-section {
  margin-bottom: 22px;
}
.help-section-title {
  font-size: 14px;
  font-weight: 700;
  color: var(--gray-900, #24323a);
  margin-bottom: 8px;
}
.help-list {
  margin: 0;
  padding-left: 18px;
  display: flex;
  flex-direction: column;
  gap: 6px;
  font-size: 13px;
  color: var(--gray-600, #5c6f79);
  line-height: 1.65;
}
.help-list strong {
  color: var(--gray-800, #37464f);
}
.about-section {
  padding-top: 14px;
  border-top: 1px solid var(--gray-100, #f0ece6);
}
.about-text {
  margin: 0;
  font-size: 12.5px;
  color: var(--gray-600, #5c6f79);
  line-height: 1.75;
}
.about-ver {
  display: inline-block;
  padding: 0 6px;
  border-radius: 4px;
  background: var(--gray-100, #f0ece6);
  font-size: 11px;
  color: var(--gray-700, #4a5560);
}
.about-shortcut {
  margin: 8px 0 0;
  font-size: 11.5px;
  color: var(--gray-400, #9aa8b0);
}
</style>

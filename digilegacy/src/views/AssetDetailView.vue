<template>
  <div class="asset-detail fade-in">
    <div class="mb-24">
      <button class="btn btn-secondary btn-sm" @click="$router.back()">← 返回</button>
    </div>

    <div v-if="asset" class="card">
      <div class="flex items-center gap-12 mb-24">
        <div style="width: 56px; height: 56px; border-radius: 14px; background: var(--primary-light); display: flex; align-items: center; justify-content: center; font-size: 28px;">{{ platformIcon(asset.platform_name) }}</div>
        <div><h2 style="font-size: 20px; font-weight: 700;">{{ asset.platform_name }}</h2><p class="text-muted" style="font-size: 14px;">{{ asset.account_identifier }}</p></div>
      </div>

      <div class="card mb-24" style="background: var(--gray-50);">
        <div class="card-title">📋 该平台的继承策略</div>
        <p style="font-size: 14px; color: var(--gray-700); line-height: 1.6; margin-top: 8px;">{{ platformInfo?.process_description || '暂无详细信息' }}</p>
        <div v-if="platformInfo?.required_documents" class="mt-16"><strong style="font-size: 13px;">所需材料：</strong><span class="text-muted" style="font-size: 13px; margin-left: 8px;">{{ platformInfo.required_documents }}</span></div>
      </div>

      <div class="card mb-24" style="background: var(--gray-50);">
        <div class="card-title">🔐 账号凭据保管（本地加密）</div>
        <template v-if="security.hasMasterPassword">
          <div class="flex items-center gap-8 mt-16">
            <input class="form-input" :type="showCredential ? 'text' : 'password'" v-model="credentialInput" placeholder="账号 / 密码 / 密保信息（一行内填写，仅保存在本机）" style="flex: 1;" />
            <button class="btn btn-secondary btn-sm" @click="showCredential = !showCredential">{{ showCredential ? '隐藏' : '显示' }}</button>
          </div>
          <div class="flex items-center gap-8 mt-12" style="flex-wrap: wrap;">
            <button class="btn btn-primary btn-sm" @click="saveCredential">加密保存</button>
            <button class="btn btn-secondary btn-sm" @click="copyCredential">复制凭据</button>
            <button class="btn btn-danger btn-sm" v-if="asset?.has_credential" @click="clearCredential">清除</button>
            <span class="tag tag-success" v-if="asset?.has_credential">已加密保存</span>
            <span class="text-muted" style="font-size: 12px;">{{ credentialStatus }}</span>
          </div>
        </template>
        <template v-else>
          <p class="text-muted" style="font-size: 13px; margin-top: 8px;">
            尚未启用加密。请先在
            <router-link to="/profile" style="color: var(--primary);">我的档案 · 安全与加密</router-link>
            设置主密码，之后即可加密保存账号与密码。
          </p>
        </template>
        <p v-if="isFinancialPlatform" style="font-size: 12px; color: var(--warning); margin-top: 12px;">⚠️ 涉及资金类平台：建议只保存查询/登录信息，不要保存银行卡取款密码（PIN）。</p>
        <div class="mt-16" style="padding-top: 12px; border-top: 1px solid var(--gray-200, #e8e4de);">
          <button class="btn btn-secondary btn-sm" @click="openOfficial">🔗 打开平台官网</button>
          <span class="text-muted" style="font-size: 12px; margin-left: 8px;">在浏览器中注册 / 找回密码后，把账号信息填回上方加密保存</span>
          <div v-if="linkStatus" class="text-muted" style="font-size: 12px; margin-top: 6px;">{{ linkStatus }}</div>
        </div>
      </div>

      <div class="grid grid-2">
        <div class="form-group"><label class="form-label">账号标识</label><input class="form-input" v-model="asset.account_identifier" /></div>
        <div class="form-group"><label class="form-label">资产类别</label><select class="form-select" v-model="asset.category"><option value="property">💰 财产型</option><option value="personality">📷 人格型</option><option value="mixed">🔗 混合型</option></select></div>
        <div class="form-group"><label class="form-label">预估价值（元）</label><input class="form-input" type="number" v-model.number="asset.estimated_value" /></div>
        <div class="form-group"><label class="form-label">处置意愿</label><select class="form-select" v-model="asset.disposal_method"><option value="inherit">指定继承人继承</option><option value="liquidate">变现后分配</option><option value="memorialize">保留为纪念</option><option value="authorize_view">授权特定人查阅</option><option value="delete">彻底删除</option><option value="no_action">暂不设定</option></select></div>
      </div>

      <div v-if="asset.disposal_method === 'inherit'" class="grid grid-2" style="margin-top: 16px;">
        <div class="form-group"><label class="form-label">继承人姓名</label><input class="form-input" v-model="asset.heir_name" placeholder="继承人姓名" /></div>
        <div class="form-group"><label class="form-label">继承人联系方式</label><input class="form-input" v-model="asset.heir_contact" placeholder="电话或邮箱" /></div>
      </div>

      <div class="form-group mt-16"><label class="form-label">备注</label><textarea class="form-textarea" v-model="asset.notes" placeholder="关于此资产的其他说明..."></textarea></div>

      <div style="display: flex; gap: 12px; justify-content: space-between; margin-top: 24px;">
        <button class="btn btn-danger btn-sm" @click="handleDelete">删除此资产</button>
        <div style="display: flex; gap: 12px;">
          <button class="btn btn-secondary btn-sm" @click="$router.back()">取消</button>
          <button class="btn btn-primary btn-sm" @click="handleSave">保存修改</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { invoke } from '../api/invoke';

import { useAssetsStore } from '../stores/assets';
import { usePlatformsStore } from '../stores/platforms';
import { useSecurityStore } from '../stores/security';
import type { ApiResponse, DigitalAsset } from '../types';

const route = useRoute();
const router = useRouter();
const assetsStore = useAssetsStore();
const platformsStore = usePlatformsStore();
const security = useSecurityStore();

const asset = ref<DigitalAsset | null>(null);
const credentialInput = ref('');
const showCredential = ref(false);
const credentialStatus = ref('');
const linkStatus = ref('');

const platformInfo = computed(() => {
  if (!asset.value) return undefined;
  return platformsStore.getById(asset.value.platform_id);
});

const isFinancialPlatform = computed(() =>
  ['支付金融', '金融证券'].includes(platformInfo.value?.category || '')
);

async function refreshAsset() {
  if (!asset.value?.id) return;
  const res = await invoke<ApiResponse<DigitalAsset | null>>('get_asset_by_id', { id: asset.value.id });
  if (res.success && res.data && asset.value) {
    asset.value.has_credential = res.data.has_credential;
  }
}

async function saveCredential() {
  if (!asset.value?.id) return;
  const text = credentialInput.value.trim();
  if (!text) {
    credentialStatus.value = '请输入要保存的内容';
    return;
  }
  try {
    const res = await invoke<ApiResponse<null>>('set_asset_credential', { id: asset.value.id, credential: text });
    credentialStatus.value = res.message;
    if (res.success) {
      credentialInput.value = '';
      showCredential.value = false;
      await refreshAsset();
    }
  } catch (e) {
    credentialStatus.value = String(e);
  }
}

async function copyCredential() {
  if (!asset.value?.id) return;
  try {
    const res = await invoke<ApiResponse<string>>('get_asset_credential', { id: asset.value.id });
    const text = res.data || '';
    if (!text) {
      credentialStatus.value = '尚未保存凭据';
      return;
    }
    await navigator.clipboard.writeText(text);
    credentialStatus.value = '已复制，30 秒后将尝试自动清除剪贴板';
    window.setTimeout(async () => {
      try {
        const current = await navigator.clipboard.readText();
        if (current === text) {
          await navigator.clipboard.writeText('');
        }
      } catch {
        credentialStatus.value = '提示：如不再需要，请手动清空剪贴板';
      }
    }, 30_000);
  } catch (e) {
    credentialStatus.value = '复制失败：' + String(e);
  }
}

async function clearCredential() {
  if (!asset.value?.id || !confirm('确定清除该资产的已保存凭据吗？')) return;
  try {
    const res = await invoke<ApiResponse<null>>('set_asset_credential', { id: asset.value.id, credential: '' });
    credentialStatus.value = res.message;
    await refreshAsset();
  } catch (e) {
    credentialStatus.value = String(e);
  }
}

async function openOfficial() {
  const url = platformInfo.value?.official_policy_url;
  if (!url) {
    linkStatus.value = '暂无该平台官方链接';
    return;
  }
  try {
    const res = await invoke<ApiResponse<null>>('open_external', { url });
    linkStatus.value = res.success ? '' : res.message;
  } catch (e) {
    linkStatus.value = '打开失败：' + String(e);
  }
}

function platformIcon(name: string): string {
  return { '支付宝': '💳', '微信': '💬', '淘宝': '🛒' }[name] || '📱';
}

async function handleSave() {
  if (!asset.value?.id) return;
  await assetsStore.updateAsset(asset.value.id, {
    platform_id: asset.value.platform_id,
    account_identifier: asset.value.account_identifier,
    account_description: asset.value.account_description,
    category: asset.value.category,
    estimated_value: asset.value.estimated_value,
    disposal_method: asset.value.disposal_method,
    heir_name: asset.value.heir_name || undefined,
    heir_contact: asset.value.heir_contact || undefined,
    notes: asset.value.notes,
  });
  router.back();
}

async function handleDelete() {
  if (asset.value?.id && confirm('确定要删除此资产吗？')) {
    await assetsStore.deleteAsset(asset.value.id);
    router.back();
  }
}

onMounted(async () => {
  await Promise.all([assetsStore.loadAssets(), platformsStore.loadPlatforms(), security.refresh()]);
  const id = Number(route.params.id);
  asset.value = assetsStore.assets.find(a => a.id === id) || null;
});
</script>

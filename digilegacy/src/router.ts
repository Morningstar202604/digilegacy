import { createRouter, createWebHashHistory, createWebHistory } from 'vue-router';

const APP_NAME = '归零 DigiLegacy';

const router = createRouter({
  // 桌面版用 history 模式（Tauri 内部协议会回退到 index.html）；
  // 在线演示站点是纯静态托管，缺少服务端 rewrite，必须用 hash 模式，
  // 否则用户刷新 /dashboard 就会拿到 404。
  history: __DEMO_MODE__
    ? createWebHashHistory(import.meta.env.BASE_URL)
    : createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/',
      redirect: '/dashboard'
    },
    {
      path: '/dashboard',
      name: 'Dashboard',
      component: () => import('./views/DashboardView.vue'),
      meta: { title: '仪表板', icon: '📊' }
    },
    {
      path: '/discover',
      name: 'Discover',
      component: () => import('./views/DiscoverView.vue'),
      meta: { title: '资产发现', icon: '🔍' }
    },
    {
      path: '/assets',
      name: 'Assets',
      component: () => import('./views/AssetsView.vue'),
      meta: { title: '我的资产', icon: '💎' }
    },
    {
      path: '/assets/:id',
      name: 'AssetDetail',
      component: () => import('./views/AssetDetailView.vue'),
      meta: { title: '资产详情', icon: '📋' }
    },
    {
      path: '/profile',
      name: 'Profile',
      component: () => import('./views/ProfileView.vue'),
      meta: { title: '我的档案', icon: '👤' }
    },
    {
      path: '/report',
      name: 'Report',
      component: () => import('./views/ReportView.vue'),
      meta: { title: '继承报告', icon: '📄' }
    },
    {
      path: '/dms',
      name: 'DMS',
      component: () => import('./views/DMSView.vue'),
      meta: { title: '安心传承', icon: '⏰' }
    },
    {
      path: '/heir',
      name: 'HeirGuide',
      component: () => import('./views/HeirGuideView.vue'),
      meta: { title: '继承人向导', icon: '👥' }
    },
    {
      path: '/legal',
      name: 'LegalTemplates',
      component: () => import('./views/LegalTemplates.vue'),
      meta: { title: '法律模板', icon: '📜' }
    },
    {
      // 兜底路由：手输 URL / 旧链接 / 演示站点刷新深层路径时，
      // 不能落到空白页；给一个明确的「页面不存在」出口。
      path: '/:pathMatch(.*)*',
      name: 'NotFound',
      component: () => import('./views/NotFoundView.vue'),
      meta: { title: '页面不存在', icon: '🧭' }
    },
  ],
  scrollBehavior(_to, _from, saved) {
    return saved ?? { top: 0 };
  },
});

router.afterEach((to) => {
  const title = (to.meta?.title as string | undefined) ?? '';
  document.title = title ? `${title} · ${APP_NAME}` : APP_NAME;
});

export default router;

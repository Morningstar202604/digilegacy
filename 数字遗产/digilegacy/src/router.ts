import { createRouter, createWebHistory } from 'vue-router';

const router = createRouter({
  history: createWebHistory(),
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
  ]
});

export default router;

<template>
  <span
    class="platform-icon"
    :style="{ fontSize: sizePx }"
    role="img"
    :aria-label="name"
  >
    <component :is="iconComponent" />
  </span>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import type { Component } from 'vue';
import Alipay from '@vicons/fa/es/Alipay';
import BookOpen from '@vicons/fa/es/BookOpen';
import Car from '@vicons/fa/es/Car';
import ChartLine from '@vicons/fa/es/ChartLine';
import Cloud from '@vicons/fa/es/Cloud';
import CreditCard from '@vicons/fa/es/CreditCard';
import FileAlt from '@vicons/fa/es/FileAlt';
import Film from '@vicons/fa/es/Film';
import Gamepad from '@vicons/fa/es/Gamepad';
import Github from '@vicons/fa/es/Github';
import GraduationCap from '@vicons/fa/es/GraduationCap';
import Home from '@vicons/fa/es/Home';
import Landmark from '@vicons/fa/es/Landmark';
import Pen from '@vicons/fa/es/Pen';
import Plane from '@vicons/fa/es/Plane';
import Qq from '@vicons/fa/es/Qq';
import ShieldAlt from '@vicons/fa/es/ShieldAlt';
import ShoppingCart from '@vicons/fa/es/ShoppingCart';
import Steam from '@vicons/fa/es/Steam';
import Tools from '@vicons/fa/es/Tools';
import Utensils from '@vicons/fa/es/Utensils';
import Users from '@vicons/fa/es/Users';
import Weibo from '@vicons/fa/es/Weibo';
import Weixin from '@vicons/fa/es/Weixin';

const props = withDefaults(defineProps<{ name: string; size?: number }>(), {
  size: 18,
});

/** 平台名 → 品牌/语义图标（与 utils/format.ts 的 emoji 表同键，供 UI 层使用） */
const PLATFORM_ICON_MAP: Record<string, Component> = {
  支付宝: Alipay,
  微信: Weixin,
  微博: Weibo,
  QQ: Qq,
  GitHub: Github,
  Steam: Steam,
  淘宝: ShoppingCart,
  王者荣耀: Gamepad,
  原神: Gamepad,
  同花顺: ChartLine,
};

/** 数据分类 → 语义图标（与 platform_data.rs 的 16 个分类一致） */
const CATEGORY_ICON_MAP: Record<string, Component> = {
  支付金融: CreditCard,
  社交通讯: Weixin,
  游戏: Gamepad,
  电商: ShoppingCart,
  金融证券: ChartLine,
  知识付费: BookOpen,
  工具订阅: Tools,
  出行住宿: Plane,
  生活工具: Home,
  内容创作: Pen,
  存储云服务: Cloud,
  政务服务: Landmark,
  音视频: Film,
  学习与考试: GraduationCap,
  汽车服务: Car,
  生活服务: Utensils,
  '账号': Users,
  '账户': Users,
};

const iconComponent = computed<Component>(
  () => PLATFORM_ICON_MAP[props.name] ?? CATEGORY_ICON_MAP[props.name] ?? ShieldAlt
);

const sizePx = computed(() => `${props.size}px`);
</script>

<style scoped>
.platform-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  line-height: 1;
  color: inherit;
}
</style>

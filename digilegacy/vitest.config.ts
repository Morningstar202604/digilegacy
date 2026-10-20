import { defineConfig } from 'vitest/config';
import vue from '@vitejs/plugin-vue';

// 前端单元测试配置（vitest 5 · jsdom）
// 说明：测试文件内显式 import { describe, it, expect } from 'vitest'，
// 不依赖 tsconfig 的 globals 类型注入，避免影响 vue-tsc 对业务代码的检查。
export default defineConfig({
  plugins: [vue()],
  test: {
    environment: 'jsdom',
    include: ['src/**/*.{test,spec}.ts'],
  },
});

import type { GlobalThemeOverrides } from 'naive-ui';

/**
 * Naive UI 主题桥接：把项目既有的暖陶土品牌 token 映射给 Naive 组件，
 * 使第三方组件与自绘样式（assets/main.css）共用同一套视觉语言。
 * 数值与 main.css 的 :root token 保持一致；深色模式由 NConfigProvider 的
 * darkTheme 叠加处理，Naive 会自行接管自身的明暗。
 */
export const themeOverrides: GlobalThemeOverrides = {
  common: {
    primaryColor: '#b4594a',
    primaryColorHover: '#a04b3d',
    primaryColorPressed: '#9a473b',
    primaryColorSuppl: '#c46a5c',
    successColor: '#4f8a6a',
    successColorHover: '#3a6e50',
    successColorPressed: '#2f6b4a',
    warningColor: '#c08a3e',
    warningColorHover: '#82561d',
    warningColorPressed: '#6d4d18',
    errorColor: '#c0503f',
    errorColorHover: '#9c3a2b',
    errorColorPressed: '#8f3324',
    infoColor: '#3f6f6a',
    infoColorHover: '#2f5855',
    infoColorPressed: '#284a47',
    bodyColor: 'transparent',
    cardColor: '#ffffff',
    modalColor: '#ffffff',
    popoverColor: '#ffffff',
    borderRadius: '10px',
    borderRadiusSmall: '8px',
    fontSize: '14px',
    fontFamily:
      "-apple-system, BlinkMacSystemFont, 'Segoe UI', 'PingFang SC', 'Microsoft YaHei', sans-serif",
  },
  Button: {
    borderRadiusMedium: '8px',
    borderRadiusSmall: '7px',
    fontWeight: '500',
  },
  Card: {
    borderRadius: '14px',
    borderColor: '#e4ddd3',
  },
  Input: {
    borderRadius: '8px',
  },
  Select: {
    peers: {
      InternalSelection: {
        borderRadius: '8px',
      },
    },
  },
  Switch: {
    railColorActive: '#b4594a',
  },
  Dialog: {
    borderRadius: '14px',
  },
  Tag: {
    borderRadius: '6px',
  },
  Drawer: {
    bodyPadding: '24px',
  },
};

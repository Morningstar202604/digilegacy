# 归零 DigiLegacy · 数字遗产管理系统

> 广东省「挑战杯」大学生创业计划竞赛项目 · 赛道：社会治理和公共服务
>
> 数字有数，归处有路。

一款本地优先、隐私优先的桌面端数字遗产规划工具。生前帮助用户**发现、登记、规划处置**自己的数字资产；身后帮助继承人**有序、合规地继承**。

---

## 一、项目定位

| 项 | 内容 |
|---|---|
| 中文名 | 归零（"归"=回归归属，"零"=归零处理） |
| 英文名 | DigiLegacy |
| 形态 | 桌面应用（Tauri 2 + Vue 3 + TypeScript + SQLite） |
| 核心主张 | **即便开发者也无法读取用户的数字遗产数据**——无服务器、无云端同步，数据仅存本地 |

## 二、功能模块

| # | 模块 | 说明 | 状态 |
|---|---|---|---|
| 1 | 数字资产发现引擎 | 基于内置 56 个中国主流平台库，引导式发现用户可能遗忘的数字资产 | MVP |
| 2 | 资产分类与登记 | 财产型 / 人格型 / 混合型三维自动归类，用户确认后入库 | MVP |
| 3 | 处置意愿设定 | 指定继承、变现分配、纪念化、授权查阅、彻底删除、捐赠、不做处理 | MVP |
| 4 | 继承报告生成 | 汇总资产清单与处置方案，导出 HTML 报告 | MVP |
| 5 | 安心传承（DMS） | Dead Man's Switch 三级确认链 + 定期签到 | 配置层就绪，通知链路待接 |
| 6 | 继承人向导 | 面向继承人的平台办理导航与进度追踪 | 前端就绪 |

## 三、技术架构

```
Vue 3 + TypeScript + Pinia + Vue Router      前端
        │  Tauri invoke (17 个 command)
Tauri 2 (Rust)                              桌面壳层
        │
rusqlite (SQLite, bundled)                  本地存储
aes-gcm / rand / base64                     加密基础设施
```

### 目录结构

```
挑战杯/
├── .gitignore
├── 数字遗产/
│   ├── 项目深度分析报告.md          商业与产品分析文档
│   ├── 项目深度分析报告_数据来源.md   数据出处核验
│   └── digilegacy/                  应用本体
│       ├── src/                     Vue 前端
│       │   ├── views/               8 个页面
│       │   ├── stores/              Pinia 状态
│       │   ├── types/               类型定义
│       │   └── assets/main.css      设计令牌与全局样式
│       ├── src-tauri/               Rust 后端
│       │   ├── src/
│       │   │   ├── platform_data.rs 56 平台政策数据库
│       │   │   ├── db.rs            SQLite schema 与全部查询
│       │   │   ├── commands.rs      17 个 Tauri command
│       │   │   ├── crypto.rs        AES-256-GCM 基础设施
│       │   │   └── models.rs        数据模型
│       │   └── capabilities/        权限白名单
│       ├── package.json
│       └── vite.config.ts
└── LICENSE
```

## 四、开发

### 环境要求

- Node.js ≥ 18
- Rust 工具链（stable）
- Tauri 2 平台依赖（Windows 需 WebView2 + MSVC Build Tools）

### 常用命令

```bash
cd 数字遗产/digilegacy

npm install

npm run dev            # 仅前端开发服务器（http://localhost:1420）
npm run tauri:dev      # 完整桌面应用开发模式
npm run build          # 类型检查 + 前端生产构建
npm run tauri:build    # 打包桌面安装包
```

### 验证状态

- `npx vue-tsc --noEmit` 通过
- `npm run build` 通过（57 modules，产物 gzip 约 55 KB）

## 五、数据存储

数据库位于系统应用数据目录下的 `digilegacy.db`（Windows：`%APPDATA%\com.digilegacy.app`），包含 5 张表：

`platforms`、`digital_assets`、`user_profile`、`check_in_log`、`dms_config`

首次启动时自动建表并写入 56 个平台的政策数据。

## 六、隐私设计

1. 无服务器、无遥测、无云端同步
2. 应用层权限白名单收敛在 `src-tauri/capabilities/default.json`
3. `crypto.rs` 已实现 AES-256-GCM 加密基础设施；MVP 阶段数据库依赖操作系统文件级访问控制，字段级加密为下一阶段目标

## 七、许可

MIT，见 [LICENSE](LICENSE)。
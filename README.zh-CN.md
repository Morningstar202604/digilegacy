# 归零 DigiLegacy — 开源数字遗产管理与继承系统

[![CI](https://github.com/badhope/digilegacy/actions/workflows/ci.yml/badge.svg)](https://github.com/badhope/digilegacy/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/cargo%20test-86%20passed-brightgreen.svg)](#开发与验证)
[![Platform](https://img.shields.io/badge/platform-Windows%2010%2F11-lightgrey.svg)](#开始使用)
[![No servers](https://img.shields.io/badge/servers-none-4a7c8c.svg)](#隐私模型)
[![No telemetry](https://img.shields.io/badge/telemetry-none-4a7c8c.svg)](#隐私模型)

[English](README.md) · **简体中文**

**官网：[badhope.github.io/digilegacy](https://badhope.github.io/digilegacy/)（[中文站](https://badhope.github.io/digilegacy/zh/)）** · **[在线演示](https://badhope.github.io/digilegacy/demo/)** · [截图](#截图)

> **规划你的数字遗产。给家人留下工具包，而不是一堆谜题。**

归零 DigiLegacy 是一款免费、开源、**本地优先的桌面应用**，用于**数字遗产规划**与**数字资产继承管理**。它帮你理清自己拥有的每一个网络账号，逐项决定它们应如何处置，并为家人产出一份**可以直接照做的继承工具包**——所有数据都加密保存在你自己的电脑上。

**开源（MIT）· 无服务器 · 无遥测 · 连开发者也无法读取你的数据。**

### 主题标签

`digital-legacy` · `digital-inheritance` · `digital-assets` · `estate-planning` · `dead-mans-switch` · `local-first` · `privacy-first` · `zero-knowledge` · `aes-256-gcm` · `encrypted-vault` · `rust` · `tauri` · `vue3` · `sqlite` · `open-source` · `desktop-app`

---

## 目录

- [为什么做这个](#为什么做这个)
- [功能](#功能)
- [截图](#截图)
- [安全模型](#安全模型)
- [横向对比](#横向对比)
- [开始使用](#开始使用)
- [架构](#架构)
- [开发与验证](#开发与验证)
- [数据存储](#数据存储)
- [隐私模型](#隐私模型)
- [常见问题](#常见问题)
- [镜像](#镜像)
- [许可](#许可)

---

## 为什么做这个

当一个人去世，家人往往既找不到、也拿不到他留在网上的东西。三个问题层层叠加：

1. **没人知道有什么。** 支付余额、游戏资产、订阅额度、创作者收入——散落在几十个平台。连本人在世时都列不全。
2. **没人知道哪些能继承。** 钱包余额可以继承，账号本身往往不行。游戏账号已有支持财产权的判例。规则因平台而异，且都埋在用户协议里。
3. **没人知道怎么去要。** 死循环：平台要继承公证书；公证处要先知道账户里有什么；家人不知道。遗产就这么沉睡下去。

现有工具各自只回答一个片段：Google 与 Apple 只覆盖自家生态；密码管理器交接的是凭据，不是继承权；线下遗嘱服务从不触碰账号本身。**完整链路——发现、登记、规划、文书、触发、执行——目前没有同类产品覆盖。**

## 功能

| # | 模块 | 做什么 |
|---|---|---|
| 1 | **数字资产发现** | 内置 **152 个平台、16 个分类**的继承政策库，配一份启发式自检问卷，专门挖出你早已忘记的账号；已登记的条目会自动去重 |
| 2 | **分类与登记** | 每项资产归为*财产型*、*人格型*或*混合型*。平台密码从不存储——应用只记录账号*在哪里*，不记录*怎么登进去* |
| 3 | **处置意愿** | 逐项选择：指定某人继承 · 变现后分配 · 保留纪念 · 授权查阅 · 彻底删除 · 捐赠 · 暂不设定 |
| 4 | **继承报告与工具包** | 一键生成 HTML 报告，并导出 ZIP 工具包（报告、资产清单、统计摘要、使用说明）。开放格式，不装本应用也能读 |
| 5 | **安心传承（Dead Man's Switch）** | 定期确认安全，超期后按**三级确认链**推进（提醒你本人 → 通知紧急联系人 → 向遗产执行人发送继承指引）。应用驻留托盘，每 30 分钟复评一次；可选**零内容**云端心跳 |
| 6 | **继承人向导** | 按平台带家人走完办理流程、所需材料与过户路径 |
| 7 | **法律文书模板库** | 六类法律文书模板（遗嘱条款、继承声明、授权委托、宣誓书、申请书、协议书） |
| 8 | **加密保险库与恢复密钥** | 字段级 AES-256-GCM 加密、主密码锁定与闲置自动锁定、一次性恢复密钥、加密备份导出/导入 |
| 9 | **新手引导与帮助中心** | 首次启动的引导；内置帮助讲清加密模型与提醒链**真实**的能力边界 |
| 9b | **浅色 / 深色主题** | 默认跟随系统，一键切换；所有表面色走设计 token，两套主题保持一致。尊重 `prefers-reduced-motion`，并附带打印样式 |
| 10 | **凭据保管** | 可选的账号凭据加密存储。**报告与工具包**导出不含凭据；**加密备份**携带密文，换机后可继续使用（没有主密码依然不可读） |
| 11 | **搜索、排序与构成图** | 跨平台名、账号与备注的全文搜索；按时间 / 价值 / 平台名 / 处置状态排序；ECharts 环形图展示财产型 / 人格型 / 混合型占比 |
| 12 | **可交互网页演示** | 同一套界面跑在模拟数据上的静态构建，随官网一起部署——不安装也能点遍每个界面 |

## 截图

| | |
|---|---|
| ![仪表板](docs/screenshots/01-dashboard.png) | ![资产发现](docs/screenshots/02-discover.png) |
| *仪表板——统计、规划完成度与资产构成* | *启发式发现问卷* |
| ![我的资产](docs/screenshots/03-assets.png) | ![资产详情](docs/screenshots/04-asset-detail.png) |
| *已登记资产：搜索、排序与处置状态* | *资产详情与加密凭据保管* |
| ![我的档案](docs/screenshots/05-profile.png) | ![安心传承](docs/screenshots/06-dms.png) |
| *档案、加密与备份* | *安心传承与本地提醒* |
| ![继承报告](docs/screenshots/07-report.png) | ![继承人向导](docs/screenshots/08-heir-guide.png) |
| *生成的继承报告* | *继承人向导* |
| ![法律模板](docs/screenshots/09-legal.png) | ![深色模式](docs/screenshots/10-dashboard-dark.png) |
| *法律文书模板库* | *深色主题（token 驱动，WCAG AA 对比度）* |

**演示视频：** [`docs/media/demo-walkthrough.mp4`](docs/media/demo-walkthrough.mp4)——32 秒点遍全流程（发现 → 登记 → 处置 → 确认 → 报告 → 继承人向导）。

> 桌面界面目前只有简体中文（主要目标语言）；英文界面在路线图上，尚未开始。官网、README 与浏览器演示均为中英双语。

## 安全模型

数字遗产是一个人最私密的数据——它既包含财产信息，也包含「谁可以看什么」的意愿。架构的做法是取消这个问题，而不是给你一个承诺：

| 层次 | 实现 |
|---|---|
| **字段级加密** | 敏感字段在写入数据库**之前**就用 **AES-256-GCM** 加密（`enc:v1:` 前缀的密文）。主密钥用 **PBKDF2-SHA256，60 万次迭代**派生（v2 之前的保险库为 10 万次；盐自带版本前缀，旧库仍可解锁） |
| **密钥生命周期** | 派生密钥只驻留内存。手动锁定或闲置 10 分钟后清零并丢弃；锁定期间凭据解密在 Rust 层就被拒绝 |
| **恢复** | 一次性 **24 位恢复密钥**封存主密钥，忘记密码仍可找回；每次修改主密码都会自动轮换恢复密钥 |
| **无服务器、无遥测** | 没有后端。除了你自己配置的可选心跳，应用不发任何网络请求——「打开平台官网」只是把 URL 交给系统浏览器（仅限 http/https） |
| **零内容心跳（可选、自托管）** | 想在关机时也收到提醒，心跳**只发送令牌**。你必须指向自己运行的服务器（`server/heartbeat-server.mjs` 是完整参考实现），在**我的档案 → 云端心跳**中配置。项目不提供默认端点——我们不运营公共服务 |
| **导出** | 报告与工具包是开放的 HTML/JSON/ZIP，**不含任何凭据**，工具包使用说明里也明确写了这一点。单独的**备份文件**确实包含凭据*密文*（绝无明文），以便恢复后保险库完整可用 |

**诚实的边界。** 本地加密保护的是静态数据；它无法阻止以你的身份运行的恶意软件，也无法阻止知道主密码的人。你选择保存的凭据就存在同一台机器上——这是所有密码管理器共同的取舍。报告与工具包是明文文件：请像对待纸质文件一样对待它。

## 横向对比

| 工具 | 覆盖范围 | 缺什么 |
|---|---|---|
| **归零 DigiLegacy** | 发现 · 登记 · 规划 · 文书 · 触发 · 执行 | — |
| Google 不活跃账户管理器 | 只覆盖 Google 账号 | 任何非 Google 平台；没有资产清单；没有规划流程 |
| Apple 数字遗产计划 | 遗产联系人访问 Apple ID | 没有清单；没有逐平台流程 |
| Meta 纪念账号 | Facebook / Instagram | 只是被动标记 |
| 1Password / Bitwarden 紧急访问 | 凭据交接 | 回答「谁拿到密码」，不回答「能不能继承、怎么继承」 |
| 死亡开关类工具（LastSignal、Aeterna 等） | 定时投递消息 | 没有资产清单、没有平台政策、没有逐项处置方案 |
| 线下遗嘱服务 / 公证处 | 纸质遗嘱 | 没有数字资产发现；没有平台级指引 |

## 开始使用

### 使用应用（Windows 10/11）

从 [Releases](https://github.com/badhope/digilegacy/releases) 下载安装包。无需账号、无需注册、无需联网。

| 安装包 | 体积（v0.2.0，x64） | 说明 |
|---|---|---|
| `DigiLegacy_0.2.0_x64-setup.exe`（NSIS） | 约 3.7 MB | 推荐；按用户安装，无需管理员权限 |
| `DigiLegacy_0.2.0_x64_zh-CN.msi`（WiX） | 约 5.2 MB | 适合集中部署 / 组策略 |

```
安装包 : DigiLegacy_*_x64-setup.exe（文件名为 ASCII；窗口标题仍为「归零 DigiLegacy」）
数据库 : %APPDATA%\com.digilegacy.app\digilegacy.db
```

首次启动有 30 秒引导介绍每个界面。内置帮助中心讲清加密模型，以及提醒链真实的边界。

### 从源码构建

```bash
git clone https://github.com/badhope/digilegacy.git
cd digilegacy

npm install
npm run tauri:dev        # 桌面开发模式
npm run tauri:build      # 产出安装包

npm run dev              # 只跑前端——浏览器预览 http://localhost:1420
npm run build:demo       # 静态演示构建（mock 后端，hash 路由）
node scripts/demo-media.mjs   # 重新生成 docs/screenshots 与演示视频（需先启动 npm run dev）
cd src-tauri && cargo test
```

### 可选：自托管心跳服务器

```bash
cd digilegacy/server
# 1. 注册客户端将发送的令牌
cat > registrations.json <<'JSON'
{ "replace-with-a-long-random-token": { "grace_days": 21, "notify_webhook": "https://your-gateway.example/hook" } }
JSON
# 2. 带管理令牌启动（/check 必需）
HEARTBEAT_ADMIN_TOKEN="$(openssl rand -hex 24)" node heartbeat-server.mjs
```

在**我的档案 → 云端心跳**中把客户端指向它（端点 + 令牌，保存在本地数据库）。环境变量 `DMS_HEARTBEAT_ENABLED` / `DMS_HEARTBEAT_TOKEN` / `DMS_HEARTBEAT_ENDPOINT` 仍作为无头部署的兜底，但应用内表单优先。

参考服务器做三件事：记录每个令牌的最后心跳时间；令牌静默超期（默认 21 天）时产出告警载荷；把载荷交给你的定时任务投递。**它从不接收或存储任何用户内容。**

对外暴露前值得知道的三处加固：`/check` 需要 `Authorization: Bearer $HEARTBEAT_ADMIN_TOKEN`，该变量未设置时返回 503（fail-closed，而不是泄漏全部令牌）；未注册令牌以 403 拒绝，除非本地实验时设置 `HEARTBEAT_ALLOW_ANY=1`；请求体上限 4 KB；告警状态持久化并在下次心跳时复位，因此重启既不会重复轰炸，也不会压掉下一次失联。`node heartbeat-server.test.mjs` 覆盖了全部这些。

```bash
# 由 cron / systemd timer 轮询
curl -sf -H "Authorization: Bearer $HEARTBEAT_ADMIN_TOKEN" http://127.0.0.1:8080/check
```

## 架构

逐模块的分解、分层规则与加密数据流见 [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md)。

```
Vue 3 + TypeScript + Pinia + Vue Router        前端
        │  Tauri invoke（42 个注册命令）
Tauri 2 (Rust)                                 桌面外壳
        │
rusqlite（内置 SQLite）                         本地存储
aes-gcm · pbkdf2-sha2 · rand · base64          加密层
```

```
.
├── docs/                                  # 项目官网（GitHub Pages）
├── digilegacy/
│   ├── src/                               # Vue 前端
│   │   ├── views/                         # 9 个导航界面 + 404
│   │   ├── stores/                        # Pinia 状态
│   │   ├── api/                           # invoke 封装 + 浏览器预览 mock
│   │   ├── components/                    # 新手引导、帮助抽屉、构成图
│   │   └── assets/main.css                # 设计 token
│   ├── src-tauri/                         # Rust 后端
│   │   ├── src/
│   │   │   ├── platform_data.rs           # 152 平台政策库
│   │   │   ├── db.rs                      # schema、查询、迁移
│   │   │   ├── commands.rs                # 41 个命令 + notify::evaluate_dms
│   │   │   ├── vault.rs                   # 主密码与字段加密
│   │   │   ├── recovery.rs                # 一次性恢复密钥
│   │   │   ├── heartbeat.rs               # 零内容云端心跳
│   │   │   ├── crypto.rs                  # AES-256-GCM 原语
│   │   │   └── notify.rs                  # 安心传承通知链
│   │   └── capabilities/                  # 权限白名单
│   ├── server/heartbeat-server.mjs        # 参考心跳服务器（+ .test.mjs）
│   └── scripts/demo-media.mjs             # 截图与演示视频生成器
└── LICENSE
```

## 开发与验证

| 检查项 | 命令 | 状态 |
|---|---|---|
| Rust 单元 + 集成测试 | `cargo test`（在 `src-tauri/`） | **86 通过，1 忽略** |
| 前端单元测试 | `npm test`（在 `digilegacy/`） | **265 通过** |
| 前端 E2E | `npm run test:e2e`（在 `digilegacy/`） | **37 通过** |
| 心跳服务器行为测试 | `node server/heartbeat-server.test.mjs` | **9 项断言通过** |
| 前端类型检查 | `npx vue-tsc --noEmit` | 通过 |
| 生产构建 | `npm run build` | 通过 |
| 覆盖率（语句/分支/函数/行） | `npm run test:coverage` | **79.4 / 79.0 / 73.2 / 80.8**（阈值 78/78/71/79） |

> `.github/workflows/ci.yml` 在每次推送 `main` 与每个 PR 时运行，共五个 job：
>
> 1. **前端** — `npm ci` → `vue-tsc --noEmit` → `npm run test:coverage`（含覆盖率门禁）→ `npm run build` → `npm audit`
> 2. **E2E** — `npm ci` → Playwright Chromium → `npx playwright test` → E2E 覆盖率报告（不阻断）
> 3. **Rust** — `cargo fmt --check` → `cargo clippy --all-targets -- -D warnings` → `cargo test --locked` → `cargo audit`
> 4. **心跳服务器** — `node --check` + `heartbeat-server.test.mjs`（9 项断言：鉴权、未注册令牌、413、健康探针）
> 5. **发布构建**（windows-latest）— `npm run tauri:build`，断言 MSI 与 NSIS 产物都存在并上传
>
> 本地等价校验用 `bash scripts/verify.sh`（会显式记录每个命令的退出码，避免 `cmd | tail` 吞掉失败）。
> 发布由 `.github/workflows/release.yml` 负责：推送 `v*` 标签时先校验三处版本号一致，再构建安装包、生成 SHA-256 校验和，并在发布后重新下载产物校验。

### 测试覆盖率——这些数字到底覆盖了什么

前端覆盖率在 `vitest.config.ts` 中设了阈值（语句 78 / 分支 78 / 函数 71 / 行 79），统计口径是**整个** `src/**` 目录（`coverage.include`），因此没被测到的视图会以 0% 计入分母，而不是从报表里消失。当前实测单元覆盖率为 79.4% 语句（分支 79.0%、函数 73.2%、行 80.8%）。另有 Playwright E2E 套件覆盖视图层端到端行为：

- **`src/api/mock.ts` 只是开发预览代码。** 它不在生产包里（`npm run build` 后 `dist/assets` 中没有任何 `mock-*.js`）。它的 14 个测试验证的是浏览器演示后端，不是发布的应用；它们是有用的回归网，但不能算作产品覆盖率。
- **视图层**有两层保护：
  - *单元/渲染测试*覆盖 `App.vue`、`AssetsView`、`ProfileView`、`ReportView`、`DiscoverView`、`AssetDetailView`、`CollapseGroup` 等；`DMSView` 97%、`ReportView` 88%、`DiscoverView` 86%、`AssetsView` 84%、`AssetDetailView` 76%、`ProfileView` 57%，外加 `DashboardView` / `HeirGuideView` / `LegalTemplates` 的筛选、分组与预览逻辑。
  - *Playwright E2E* 覆盖单元测试够不到的视图：`DiscoverView`、`DMSView`、`DashboardView`、`ReportView`、`HeirGuideView`、`LegalTemplates`、`AssetDetailView`。它驱动真实 Chromium 走真实组件、store 与路由（只有 IPC 后端是预览 mock），覆盖完整发现链路、DMS 签到、报告导出、继承向导 Tab、法律模板预览与资产详情跳转。

  E2E 不计入上面的 v8 覆盖率数字——它是行为网，不是行覆盖工具。运行方式：`npm run test:e2e`（首次先 `npx playwright install chromium`）。

由新增测试发现并修复的四个真实缺陷：

1. `dashboard.saveUser` 在写入被拒（保险库锁定）时仍报成功，界面显示「已保存」而实际什么都没写。
2. `CollapseGroup` 的 `defaultOpen` 被 Vue 的布尔 prop 转换吞掉——发现页的平台分组本应默认展开，实际是折叠的。
3. 恢复流程成功后会清空 `recoveryUnlocked`，把**新的**恢复密钥和表单一起隐藏；旧码此时已失效，用户永远拿不到新码。
4. 恢复界面根本没有提交按钮——只有回车能用，鼠标用户被困住。

## 数据存储

单个本地文件：应用数据目录下的 `digilegacy.db`（Windows 为 `%APPDATA%\com.digilegacy.app`），共八张表：

`platforms` · `digital_assets` · `user_profile` · `check_in_log` · `dms_config` · `security_config` · `heartbeat_config` · `platform_overrides`

首次启动时创建数据库并写入平台政策库；平台表**每次启动都幂等同步**，因此政策与链接修正能到达存量安装，而不触碰用户数据。旧数据库会自动迁移（`credential_enc`、`recovery_seal`、`emergency_contact_email`、`heir_primary_email` 列），schema 版本记录在 `PRAGMA user_version`（当前为 `2`）。

`heartbeat_config` 保存可选的云端心跳端点与令牌；`platform_overrides` 保存用户导入的政策补丁，单独存放是因为内置平台表每次启动都会被重写。两者自备份格式 v2 起都会进入备份；导入旧版 v1 备份时不会清空本地副本。

## 隐私模型

1. **字段级加密。** 账号标识、凭据、档案字段与继承人联系方式在存储前用 AES-256-GCM 加密；主密钥由 PBKDF2-SHA256 派生，只驻留内存。
2. **主密码锁定。** 手动锁定，或闲置 10 分钟后自动锁定。锁定期间凭据解密在 Rust 层被拒绝——UI 遮罩只是可见的那一半。
3. **凭据保管。** 可选的加密存储与复制到剪贴板。凭据绝不进入报告或工具包；加密备份只保存密文，没有主密码不可读。
4. **无服务器、无遥测、无云同步。** 外部链接仅限 `http`/`https`。
5. 应用权限被收窄到 `src-tauri/capabilities/default.json` 中的白名单。

## 常见问题

<details>
<summary><strong>真的免费吗？</strong></summary>

真的。桌面应用采用 MIT 许可，永久免费，全部源码开放。可选的云端便利功能（加密备份、代配置服务）计划作为付费服务；本地应用始终免费。
</details>

<details>
<summary><strong>如果项目停止维护，我的数据怎么办？</strong></summary>

什么都不会坏。没有服务器需要关停：数据是一个本地 SQLite 文件，导出物是开放格式，代码是公开的——任何人都可以 fork 继续维护。这正是这样设计架构的原因。
</details>

<details>
<summary><strong>它会保存我的密码吗？</strong></summary>

绝不保存平台密码明文。若你主动保存凭据，它们会用你的主密码在本机加密，并被刻意排除在所有导出物之外。
</details>

<details>
<summary><strong>它能自动扫描出我全部账号吗？</strong></summary>

不能。任何声称能做到的工具，要么在说谎，要么需要你的账号凭据。归零做的是**引导式发现**：它认识 152 个主流平台，用问题帮你回忆，再由你确认。最终清单永远由你决定。
</details>

<details>
<summary><strong>应用关掉后提醒还会生效吗？</strong></summary>

托盘常驻（默认）时，应用每 30 分钟复评一次，并可设置开机自启。机器关机期间想继续提醒，可启用可选的零内容云端心跳——服务器只看得到一个令牌。
</details>

<details>
<summary><strong>为什么政策库以中国平台为主？</strong></summary>

因为那里的平台继承规则最不透明、最少公开资料——库内覆盖 152 个中国大陆常用平台（微信、支付宝、淘宝、B 站、Steam 等），高价值条目在 `data_source` 中带有来源引用。架构本身并不绑定地区。
</details>

<details>
<summary><strong>怎么参与贡献？</strong></summary>

欢迎提 Issue 与 PR，尤其欢迎政策纠错（附官方政策链接与来源引用）和翻译。政策库是纯 Rust 数据，扩展很容易。
</details>

## 镜像

| 平台 | 仓库 | 网站 |
|---|---|---|
| GitHub · badhope | [github.com/badhope/digilegacy](https://github.com/badhope/digilegacy) | [badhope.github.io/digilegacy](https://badhope.github.io/digilegacy/) |
| GitHub · x33834 | [github.com/x33834/digilegacy](https://github.com/x33834/digilegacy) | [x33834.github.io/digilegacy](https://x33834.github.io/digilegacy/) |
| GitHub · Morningstar202604 | [github.com/Morningstar202604/digilegacy](https://github.com/Morningstar202604/digilegacy) | [morningstar202604.github.io/digilegacy](https://morningstar202604.github.io/digilegacy/) |
| Gitee · badhope | [gitee.com/badhope/digilegacy](https://gitee.com/badhope/digilegacy) | — |
| GitCode · badhope | [gitcode.com/badhope/digilegacy](https://gitcode.com/badhope/digilegacy) | — |

## 许可

MIT——见 [LICENSE](LICENSE)。

*数字有数，归处有路。*

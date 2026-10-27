<div align="center">

<img src="docs/screenshots/01-dashboard.png" alt="归零仪表板：资产总数、预估价值、规划完成度与资产构成" width="900">

<h1>归零 DigiLegacy</h1>

<p><strong>规划你的数字遗产。给家人留下工具包，而不是一堆谜题。</strong></p>

<p>
  <a href="https://github.com/badhope/digilegacy/actions/workflows/ci.yml"><img src="https://github.com/badhope/digilegacy/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="MIT 许可">
  <img src="https://img.shields.io/badge/tests-193%20Rust%20%C2%B7%20404%20frontend%20%C2%B7%2080%20E2E-brightgreen.svg" alt="测试">
  <img src="https://img.shields.io/badge/platform-Windows%2010%2F11-lightgrey.svg" alt="Windows">
  <img src="https://img.shields.io/badge/servers-none-4a7c8c.svg" alt="无服务器">
  <img src="https://img.shields.io/badge/telemetry-none-4a7c8c.svg" alt="无遥测">
</p>

<p>
  <a href="https://badhope.github.io/digilegacy/zh/"><strong>中文官网</strong></a> ·
  <a href="https://badhope.github.io/digilegacy/">English site</a> ·
  <a href="https://badhope.github.io/digilegacy/demo/"><strong>在线演示</strong></a> ·
  <a href="https://github.com/badhope/digilegacy/releases"><strong>下载</strong></a>
</p>

<p><strong>简体中文</strong> · <a href="README.md">English</a></p>

</div>

---

## 一条完整链路，装进一个应用

其他工具大多只解决一个片段：Google 与 Apple 只覆盖自家账号，密码管理器只交接密码，线下遗嘱服务从不触碰账号本身。归零把整条路走完——**资产发现 → 分类登记 → 处置规划 → 法律文书 → 到期触发 → 家人执行**——全部落在同一个本地数据库里。

| | |
|---|---|
| 内置 **152** 个平台、**16** 个分类的继承政策库 | **67** 个 IPC 命令，Rust 内核 + Vue 前端 |
| **AES-256-GCM** 字段级加密，PBKDF2-SHA256 60 万次迭代 | **0** 服务器、**0** 遥测、**0** 云端调用 |

## 走一遍界面

<table>
<tr>
<td width="50%"><img src="docs/screenshots/02-discover.png" alt="启发式资产发现"><br><sub><b>资产发现</b>——问卷帮你想起早已忘记的账号</sub></td>
<td width="50%"><img src="docs/screenshots/03-assets.png" alt="资产列表与搜索排序"><br><sub><b>我的资产</b>——搜索、四种排序、处置状态一眼可见</sub></td>
</tr>
<tr>
<td><img src="docs/screenshots/04-asset-detail.png" alt="资产详情与加密凭据保管"><br><sub><b>资产详情</b>——平台政策、处置意愿、加密凭据保管</sub></td>
<td><img src="docs/screenshots/06-dms.png" alt="安心传承"><br><sub><b>安心传承</b>——三级链条：本人 → 紧急联系人 → 执行人</sub></td>
</tr>
<tr>
<td><img src="docs/screenshots/07-report.png" alt="继承报告"><br><sub><b>继承报告</b>——可打印文书 + ZIP 工具包</sub></td>
<td><img src="docs/screenshots/08-heir-guide.png" alt="继承人向导"><br><sub><b>继承人向导</b>——按平台的流程、材料与过户路径</sub></td>
</tr>
<tr>
<td><img src="docs/screenshots/09-legal.png" alt="法律模板"><br><sub><b>法律模板</b>——六类文书，填上就能用</sub></td>
<td><img src="docs/screenshots/10-dashboard-dark.png" alt="深色模式"><br><sub><b>深色模式</b>——token 驱动，WCAG AA 对比度</sub></td>
</tr>
</table>

**[▶ 看 32 秒全流程演示](docs/media/demo-walkthrough.mp4)**——不用安装，直接看完全程。

## 你会得到什么

| | | |
|---|---|---|
| 🔍 **资产发现** | 152 平台政策库、启发式问卷、自动去重 | |
| 🗂 **分类登记** | 财产型 / 人格型 / 混合型，逐项登记 | |
| 📝 **处置规划** | 继承 · 变现 · 纪念 · 授权查阅 · 删除 · 捐赠 | |
| 📄 **报告与工具包** | 可打印 HTML + ZIP（报告、清单、摘要、说明） | |
| ⏰ **安心传承** | 三级确认链、托盘常驻、可选零内容心跳 | |
| 👥 **继承人向导** | 按平台的材料、流程、过户路径与风险提示 | |
| 🔐 **加密保险库** | AES-256-GCM、闲置自动锁定、一次性恢复密钥、加密备份 | |
| 🎨 **双主题** | 浅色与深色共用一套设计 token，尊重减动效偏好 | |
| 🔎 **搜索与图表** | 全文搜索、多级排序、ECharts 构成环形图 | |
| 🧪 **在线演示** | 浏览器里点遍全部界面，装之前先看 | |

## 按产品标准做

| 检查项 | 结果 |
|---|---|
| Rust 测试 | **118 通过**，1 ignored（`cargo test --locked`；源码声明 120，含 1 条仅 Unix 编译） |
| 前端单元测试 | **404 通过**，32 个文件 |
| 端到端测试 | **80 通过**，真实 Chromium |
| 覆盖率 | **85.4% 语句**、86.9% 行，阈值写进 CI |
| 类型安全 | `vue-tsc --noEmit` 无错误，Rust `clippy -D warnings` 无警告 |
| 供应链 | `cargo audit` 与 `npm audit`：0 漏洞 |
| CI | 5 个 job——前端、E2E、Rust、心跳服务器、Windows 安装包构建 |
| 发布 | 带 SHA-256 校验和的构建流水线，发布后重新下载校验 |

**生产级细节：** 演示后端不会进入安装包（CI 断言无 `mock-*` 分块）；数据库 schema 带 `PRAGMA user_version`；IPC 入口对输入做长度与取值范围校验；备份格式（v4）覆盖心跳、政策补丁、恢复封印、继承人、SMTP、保管位置与助手配置；备份/报告/工具包/SMTP 凭据文件按仅本人可读权限落盘；导入备份前自动落快照，默认合并而非覆盖。

## 开始使用

**Windows 10/11 · 约 3.5 MB · 无需账号、无需注册、无需联网**

```bash
# 从 Releases 下载安装包，双击安装。
# 数据保存在：%APPDATA%\com.digilegacy.app\digilegacy.db
```

[⬇ 下载安装包](https://github.com/badhope/digilegacy/releases) · [▶ 打开在线演示](https://badhope.github.io/digilegacy/demo/)

从源码构建：

```bash
git clone https://github.com/badhope/digilegacy.git
cd digilegacy && npm install
npm run tauri:dev      # 桌面开发
npm run tauri:build    # 产出安装包
```

## 技术栈

`Tauri 2` · `Rust` · `rusqlite` · `aes-gcm` · `pbkdf2` · `Vue 3` · `TypeScript` · `Pinia` · `Vite` · `ECharts` · `Playwright` · `Vitest`

## 诚实的边界

本地加密保护的是静态数据——它挡不住以你的身份运行的恶意软件，也挡不住知道主密码的人。报告与工具包是明文文件，请像对待纸质文件一样对待它。桌面界面提供简体中文与英文（标题栏切换）；官网、文档与演示同样为双语。

**「安心传承」是尽力而为，应用里也这么写。** 它只在应用运行时检测；电脑关机、睡眠或应用被彻底退出期间不检查，也不会补发错过的轮次。邮件需要你自己配置 SMTP 服务器——没配时通知只进本地（明文的）发件箱，家人什么都收不到。可选的「零内容心跳」是唯一在应用关闭后仍然有效的通道，默认关闭。

## 深入了解

- **架构与加密数据流** — [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md)
- **路线图**（定位复核、缺口清单、优先级与验收标准）— [`docs/ROADMAP.md`](docs/ROADMAP.md)
- **验证证据**（命令、数字、已知缺口）— [`docs/VERIFICATION.md`](docs/VERIFICATION.md)
- **横向对比、安全说明、常见问题** — [官网](https://badhope.github.io/digilegacy/zh/#compare) · [English](https://badhope.github.io/digilegacy/#compare)
- **参与贡献** — [`CONTRIBUTING.md`](CONTRIBUTING.md) · **安全策略** — [`SECURITY.md`](SECURITY.md)

## 许可

MIT。五个镜像：[GitHub](https://github.com/badhope/digilegacy) · [x33834](https://github.com/x33834/digilegacy) · [Morningstar202604](https://github.com/Morningstar202604/digilegacy) · [Gitee](https://gitee.com/badhope/digilegacy) · [GitCode](https://gitcode.com/badhope/digilegacy)。

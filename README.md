# 归零 DigiLegacy · 数字遗产管理系统

[![CI](https://github.com/badhope/digilegacy/actions/workflows/ci.yml/badge.svg)](https://github.com/badhope/digilegacy/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust Tests](https://img.shields.io/badge/cargo%20test-37%20passed-brightgreen.svg)](#八开发与验证)
[![Platform](https://img.shields.io/badge/platform-Windows%2010%2F11-lightgrey.svg)](#六开发)
[![Local-first](https://img.shields.io/badge/privacy-local--first-4a7c8c.svg)](#九隐私设计)

> 广东省「挑战杯」大学生创业计划竞赛项目 · 赛道：社会治理和公共服务
>
> 数字有数，归处有路。

一款本地优先、隐私优先的桌面端数字遗产规划工具。生前帮助用户**发现、登记、规划处置**自己的数字资产；身后帮助继承人**有序、合规地继承**。

**开源（MIT 许可）· 无服务器 · 无遥测 · 数据只属于用户**

---

## 一、项目定位

| 项 | 内容 |
|---|---|
| 中文名 | 归零（"归"=回归归属，"零"=归零处理） |
| 英文名 | DigiLegacy |
| 形态 | 桌面应用（Tauri 2 + Vue 3 + TypeScript + SQLite） |
| 核心主张 | **即便开发者也无法读取用户的数字遗产数据**——无服务器、无云端同步，数据仅存本地 |

## 二、界面截图

| | |
|---|---|
| ![仪表板](数字遗产/digilegacy/screenshots/01-dashboard.png) | ![资产发现](数字遗产/digilegacy/screenshots/02-discover.png) |
| *仪表板* | *资产发现* |
| ![我的资产](数字遗产/digilegacy/screenshots/03-assets.png) | ![我的档案](数字遗产/digilegacy/screenshots/04-profile.png) |
| *我的资产* | *我的档案* |
| ![安心传承](数字遗产/digilegacy/screenshots/05-dms.png) | ![继承报告](数字遗产/digilegacy/screenshots/08-report.png) |
| *安心传承（DMS）* | *继承报告* |
| ![继承人向导](数字遗产/digilegacy/screenshots/06-heir-guide.png) | ![法律模板](数字遗产/digilegacy/screenshots/07-legal.png) |
| *继承人向导* | *法律模板* |

## 三、使用指南

### 场景 A：资产持有人（生前规划）

首次启动会自动播放约 30 秒的新手引导（可跳过；右上角「帮助」中可随时重播，帮助中心还汇总了加密、备份与通知的真实能力说明）。

1. **资产发现**：回答自检问卷，应用从内置的 152 个平台库（16 个分类）中引导式发现你可能持有的数字资产，并自动排除已登记项
2. **我的资产**：确认发现结果、补登遗漏；每项资产自动归入财产型 / 人格型 / 混合型三维分类
3. **处置意愿**：为每项资产选择处置方式——指定继承、变现分配、纪念化、授权查阅、彻底删除、捐赠或不做处理
4. **我的档案**：完善本人与继承人基础信息，作为报告与文书的数据来源；在此设置主密码启用加密
5. **安心传承（DMS）**：开启定期签到（顶部按钮或托盘菜单一键检查）；应用驻留托盘期间每 30 分钟自动评估，到期前 3 天或逾期时发送系统通知（按日去重），逾期后沿三级确认链逐级通知；可选开启云端心跳（零内容）
6. **继承报告**：一键生成汇总报告，导出 HTML 报告与继承工具包 ZIP（报告 / 清单 / 统计三件套）
7. **法律模板**：下载遗嘱、继承声明、授权委托等 6 类文书模板，配合线下公证使用
8. **安全与加密（我的档案页）**：设置 / 修改主密码；导出加密备份或导入备份文件（换电脑迁移，同密码可解锁）

资产详情页还提供两项配套能力：**官方入口跳转**（一键在浏览器打开平台官网，注册 / 找回密码 / 办理业务）与**凭据加密保管**（账号密码等敏感信息加密保存，随时复制，30 秒后尝试自动清除剪贴板）。

### 场景 B：继承人（身后执行）

1. **继承人向导**：按平台查询办理流程、变现 / 过户路径卡与常见困难应对方案
2. **继承工具包**：使用持有人导出的 ZIP 与法律模板，按清单逐平台办理

### 数据安全

数据仅存储在本机 SQLite 数据库中，应用无遥测；设置主密码后，账号、凭据、档案等敏感字段以 AES-256-GCM 加密落库，开发者从架构上无法读取；卸载应用即完全删除数据。云端心跳为可选项且采用零内容设计——服务器只接收令牌，永远接触不到用户的资产与凭据数据。

## 四、功能模块

| # | 模块 | 说明 | 状态 |
|---|---|---|---|
| 1 | 数字资产发现引擎 | 基于内置 152 个中国主流平台库（16 个分类），启发式自检问卷 + 引导式发现，自动去重已登记平台；每个平台附官方入口链接 | MVP |
| 2 | 资产分类与登记 | 财产型 / 人格型 / 混合型三维自动归类，用户确认后入库 | MVP |
| 3 | 处置意愿设定 | 指定继承、变现分配、纪念化、授权查阅、彻底删除、捐赠、不做处理 | MVP |
| 4 | 继承报告生成 | 汇总资产清单与处置方案，导出 HTML 报告 + 继承工具包 ZIP（报告/清单/统计三件套） | MVP |
| 5 | 安心传承（DMS） | Dead Man's Switch 三级确认链 + 定期签到；运行期间每 30 分钟自动评估，到期/逾期发系统通知（按日去重）；通知落本地发件箱（离线兜底），可选 SMTP 邮件；短信与跨设备推送为云端计划 | MVP |
| 6 | 继承人向导 | 面向继承人的平台办理导航、变现/过户路径卡、常见困难解决方案 | MVP |
| 7 | 法律模板库 | 6 类法律文书模板（遗嘱/继承声明/授权委托/宣誓书/申请/协议），预览与下载 | MVP |
| 8 | 安全与加密 | 主密码（PBKDF2-SHA256）→ 敏感字段级 AES-256-GCM 加密；修改主密码（全库重加密）；手动锁定 + 闲置 10 分钟自动锁定；凭据加密保管、复制后 30 秒尝试清除剪贴板；加密备份导出/导入；外部跳转仅放行 http/https | MVP |
| 9 | Web 开发预览 | 浏览器打开 `npm run dev` 即为预览模式：自动启用本地 mock（内置 152 个平台真实数据 + 演示资产），用于开发时查看前端外观；仅开发环境生效，桌面版与生产构建不受影响 | 开发工具 |
| 10 | 新手引导与帮助中心 | 首次启动分步引导（聚光高亮导航与关键入口、可跳过、可在帮助中心重播）；常驻「帮助」入口：快速上手、加密说明、DMS 真实能力边界、常见问题 | MVP |
| 11 | 托盘常驻与后台推进 | 关闭窗口驻留系统托盘（菜单：显示 / 我还安好 / 锁定 / 退出），后台每 30 分钟推进 DMS 评估与云端心跳；可选开机自启 | MVP |
| 12 | 恢复密钥 | 设置主密码时生成一次性 24 位恢复码（4 位一组）；忘记主密码时凭恢复码重设密码并全库重加密；修改主密码自动轮换恢复码 | MVP |
| 13 | 云端心跳（可选） | 零内容设计：仅上报「我还安好」令牌，不携带任何资产/凭据/档案数据；失联容忍不影响本地；服务器参考实现见 `server/heartbeat-server.mjs` | MVP（客户端）/ 参考实现（服务端） |

## 五、技术架构

```
Vue 3 + TypeScript + Pinia + Vue Router      前端
        │  Tauri invoke (29 个 command)
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
│   └── digilegacy/                  应用本体
│       ├── src/                     Vue 前端
│       │   ├── views/               9 个页面
│       │   ├── stores/              Pinia 状态
│       │   ├── api/                 invoke 包装与 Web 预览 mock
│       │   ├── types/               类型定义
│       │   └── assets/main.css      设计令牌与全局样式
│       ├── src-tauri/               Rust 后端
│       │   ├── src/
│       │   │   ├── platform_data.rs 152 平台 / 16 分类政策数据库
│       │   │   ├── db.rs            SQLite schema 与全部查询
│       │   │   ├── commands.rs      29 个 Tauri command
│       │   │   ├── vault.rs         主密码与字段级加密（锁定即清钥）
│       │   │   ├── crypto.rs        AES-256-GCM 基础设施
│       │   │   └── models.rs        数据模型
│       │   └── capabilities/        权限白名单
│       ├── package.json
│       └── vite.config.ts
└── LICENSE
```

## 六、开发

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

### 云端心跳服务器（自托管，可选）

```bash
cd 数字遗产/digilegacy/server
node heartbeat-server.mjs          # 默认 8080 端口
# 客户端环境变量：DMS_HEARTBEAT_ENABLED=1  DMS_HEARTBEAT_TOKEN=<令牌>
#                DMS_HEARTBEAT_ENDPOINT=http://<host>:8080
```

服务器只做三件事：记录令牌最后心跳时间、超期（默认 21 天）输出提醒载荷、由定时任务把提醒转发到邮件网关。**不接收、不存储任何用户内容数据。**

### Web 开发预览（浏览器看前端）

`npm run dev` 后直接在浏览器打开 http://localhost:1420 即可预览全部 9 个页面：

- 非 Tauri 环境下自动启用 `src/api/mock.ts` 本地 mock：内置 152 个平台真实数据与演示资产，支持增删改、凭据、备份等交互（数据存 localStorage，重启浏览器不丢）；
- mock 仅在开发环境生效，`npm run build` 的生产构建不包含 mock 代码；
- 桌面端行为不受影响（`src/api/invoke.ts` 自动检测运行环境路由）。

### 验证状态

- `npx vue-tsc --noEmit` 通过
- `npm run build` 通过（68 modules，含类型检查）
- `cargo check` 通过
- `cargo test` 通过（37 / 37）

## 七、数据存储

数据库位于系统应用数据目录下的 `digilegacy.db`（Windows：`%APPDATA%\com.digilegacy.app`），包含 6 张表：

`platforms`、`digital_assets`、`user_profile`、`check_in_log`、`dms_config`、`security_config`

首次启动时自动建表并写入 152 个平台的政策数据；旧版本数据库会在启动时自动迁移（新增 `credential_enc` 列）。

## 八、隐私设计

1. **字段级加密**：设置主密码后，资产账号、凭据、档案、继承人联系方式等敏感字段在写入数据库前以 AES-256-GCM 加密（密文带 `enc:v1:` 前缀）；主密码经 PBKDF2-SHA256（10 万次迭代）派生密钥，仅驻留内存，锁定即清除
2. **主密码锁**：可手动锁定或闲置 10 分钟自动锁定；锁定后应用被遮罩拦截，凭据读取在解密层直接拒绝
3. **凭据保管**：账号密码等凭据单独加密存储，支持一键复制（30 秒后尝试自动清除剪贴板）；继承报告与工具包不包含任何凭据
4. **无服务器、无遥测、无云端同步**；外部链接跳转仅允许 http/https 协议
5. 应用层权限白名单收敛在 `src-tauri/capabilities/default.json`

## 九、许可

MIT，见 [LICENSE](LICENSE)。
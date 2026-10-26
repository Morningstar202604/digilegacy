# 验证记录

本文件记录最终验证证据，避免"看起来绿"与"真的绿"混淆。

## 2026-10-26 扩展升级轮

| 命令 | 结果 |
|---|---|
| `npx vue-tsc --noEmit` | exit 0 |
| `npm run test:coverage` | **330 tests / 30 files**，statements 85.07%（branches 78.43 / functions 76.57 / lines 86.53，阈值 78/78/71/79） |
| `npm run build` | PASS；`dist/assets` 无 `mock-*.js` 分块 |
| `npx playwright test` | **37 passed**（27.0s） |
| `cargo test --locked` | **95 passed, 0 failed, 1 ignored** |
| `cargo clippy --all-targets -- -D warnings` | 0 warnings |
| `cargo fmt --check` | PASS |
| `node server/heartbeat-server.test.mjs` | all assertions passed |

本轮修复与新增（每条都有回归测试）：

| 项目 | 级别 | 回归测试 |
|---|---|---|
| 打开仪表板即调用 `evaluate_dms`，会真的执行一轮 SMTP 投递（读页面产生副作用） | P0 | `DMSView.test.ts`「打开页面只读状态，绝不触发投递命令」、`dashboard.test.ts`「evaluateDms 走只读命令」 |
| 明文备份导入已加密库时绕过加密、直接写明文列 | P0 | `db.rs::plaintext_backup_never_lands_as_plaintext_in_encrypted_vault` |
| 备份不含恢复封印，换机恢复后恢复码失效 | P1 | `db.rs::import_backup_preserves_v3_seal_and_clears_legacy_stale_seal` |
| 剩余天数用 `num_days()` 截断，到期前不足一天被误判逾期 | P1 | `db.rs::days_until_due_rounds_up_instead_of_truncating` |
| 政策补丁导入后 `platformsExtended` 仍是旧缓存，继承人向导显示旧政策 | P1 | `platforms.test.ts::refreshPlatforms` |
| ZIP 工具包里的报告是页内片段，缺 DOCTYPE 与打印样式 | P1 | `ReportView.test.ts::ZIP 里的报告必须是独立 HTML` |
| `set_disposal_wish` 绕过全部输入校验（任意长度文本/非法枚举） | P2 | `commands.rs::disposal_wish_validates_enum_and_size` |
| `import_backup` / `export_report_to_file` 无体积上限 | P2 | `commands.rs::export_commands_reject_oversized_payloads` |
| 心跳地址经环境变量可绕过 http(s) 校验 | P2 | `heartbeat.rs::post_heartbeat_rejects_non_http_schemes` |
| 备份与 SMTP netrc 在 Unix 上按 0644 落盘 | P2 | `fsutil.rs::unix_file_is_owner_only` |
| 英文界面金额仍按「万」显示 | P2 | `format.test.ts::英文界面按 k/M/B 缩写` |
| 词典存在中英不对称、界面残留 emoji | P3 | `i18n/index.test.ts` 键集对齐 + emoji 门禁 |
| 自动备份是静默功能，界面不可见 | P3 | `ProfileView.test.ts::自动备份状态可见`、`backup.rs::latest_summary_reports_newest_file_and_size` |
| 通知历史只看本轮评估结果，刷新即消失 | P2 | `DMSView.test.ts::历史投递持久可见`、`notify.rs::outbox_history_reads_newest_first_and_skips_bad_lines` |

## 本地门禁（Windows，`bash scripts/verify.sh`；历史轮 · 2026-10-24 数字）

| 步骤 | 命令 | 结果 |
|---|---|---|
| 依赖就绪 | `npm install`（本地；CI 用 `npm ci`） | PASS |
| 类型检查 | `npx vue-tsc --noEmit` | PASS (exit 0) |
| 测试 + 覆盖率门禁 | `npm run test:coverage` | 265 tests / 26 files，statements 79.41%（branches 79.00 / functions 73.17 / lines 80.78，阈值 78/78/71/79） |
| 生产构建 | `npm run build` | PASS |
| E2E（Playwright） | `npx playwright test` | **37 passed**（7 视图 + 发现链路 + 全站 UI 健全性 + 深色模式 + 视觉验收） |
| 格式 | `cargo fmt --check` | PASS |
| Lint | `cargo clippy --all-targets -- -D warnings` | 0 warnings |
| Rust 测试 | `cargo test --locked` | 86 passed, 0 failed, 1 ignored |
| 心跳服务器行为测试 | `node server/heartbeat-server.test.mjs` | 9 assertions PASS |

## 跨平台验证（Ubuntu 24.04 容器，真实 Linux）

CI 的 Rust job 在 linux 上跑，本地是 Windows。为避免"本地绿、CI 红"，在
Ubuntu 24.04 容器内用 Rust 1.99.1 完整复跑：

| 步骤 | 结果 |
|---|---|
| `cargo fmt --check` | PASS |
| `cargo clippy --all-targets -- -D warnings` | PASS，0 warnings |
| `cargo test --locked` | 59 passed, 0 failed, 1 ignored（68.04s）— **历史记录**，当时套件规模；此后用例增至 86，容器未重跑 |

## E2E 跨平台验证（Ubuntu 24.04 容器）

E2E 首次在 Linux 容器内运行时 18 个用例全失败，排查后确认**不是测试代码问题**：
容器内 Playwright 浏览器二进制版本（1243）与 lockfile 要求（1248）不匹配，
`npx playwright install` 因 191MB 包在容器网络下超时未完成。

改用容器内已存在的 chromium 显式指定 `executablePath` 后复跑：

| 结果 | 耗时 |
|---|---|
| **18 passed / 0 failed** | 27.5s |

结论：E2E 用例本身跨平台可跑通；CI 上浏览器由 `npx playwright install --with-deps chromium`
安装，job 已设 `timeout-minutes: 15` 避免大包下载被截断。该记录为当时 18 例规模，当前套件为 37 例。

## E2E 变异验证

E2E 的价值在于能抓住视图层变异（单测覆盖不到）：

| 变异 | 结果 |
|---|---|
| DashboardView 统计卡片 class 改名 | 失败（被抓住） |
| ReportView 报告正文容器改名 | 失败（被抓住） |
| LegalTemplates 预览按钮去掉 click 绑定 | 失败（被抓住） |

## 心跳服务器 job

与 CI 中完全相同的脚本序列本地执行：

| 断言 | 结果 |
|---|---|
| `node --check heartbeat-server.mjs` | PASS |
| `POST /heartbeat` 返回 `{"ok":true}` | PASS |
| 非法请求（空 body）返回 400 | PASS |
| `GET /check` 返回 alerts/checked_at | PASS |

## Node 版本

依赖链 engines 要求 `^22.22.2 || ^24.15.0 || >=26`（jsdom 及其传递依赖）。本机 Node 24.14.0
**不满足**该范围（`npm ci` 会打印 EBADENGINE，但可运行），CI 固定 `22.22.2`，并在
`digilegacy/package.json` 的 `engines` 字段与本文件同步声明，`.nvmrc` 固定版本。

## 未能在本机验证的部分

- **GitHub Actions 运行结果**：`api.github.com` 在本机 TLS 不可达（curl exit 35），
  代理 `ghfast.top` 仅支持 git 协议。首次成功运行已由独立评审通过 GitHub API
  核实：run `39558924027`（sha `3fc3b68`）**completed / success**，当时三个 job 全绿。
  CI 现为五个 job（前端 / E2E / Rust / 心跳服务器 / 发布构建），本次改动后的运行结果
  仍需在 Actions 页面确认：
  https://github.com/badhope/digilegacy/actions

## 视觉与品牌验证

| 项目 | 验证方式 | 结果 |
|---|---|---|
| 深色模式 | Playwright 设 `data-theme=dark` 后截图检查标题/背景/卡片层次 | 通过（3 页截图） |
| 主题切换 | 点按钮后断言 `data-theme` 由 light 变 dark | 通过 |
| Toast / 确认框 | 单元测试 12 例（自动消失、错误样式、Promise 语义、alertdialog） | 通过 |
| 空态插画 | 资产页空数据渲染 SVG | 通过 |
| 报告文书版式 | 页眉/编号/页脚/签名区渲染 | 通过 |

## 可靠性与供应链

| 项目 | 验证 | 结果 |
|---|---|---|
| 后台任务 panic 隔离 | 单轮 panic 不终止循环（2 例） | 通过 |
| 每日自动备份 | 当日幂等、保留 7 份、裁剪顺序、无临时残留（4 例） | 通过 |
| Rust 依赖扫描 | `cargo audit --deny vulnerabilities` | 0 漏洞（5 条 unmaintained/unsound 警告来自 Tauri 上游） |
| npm 依赖扫描 | `npm audit --audit-level=high` | 0 漏洞 |
| E2E 覆盖率 | `npm run test:e2e:coverage` | 语句 58.9% / 行 60.1%（26 文件，与单测口径互补） |

## 发布构建验证（Windows）

此前从未验证过安装包能否产出。实测发现并修复了一个发布阻塞缺陷：
WiX 默认 codepage 1252 无法编码中文产品名「归零」，MSI 打包直接失败
（代码全绿但用户装不上）。

修复：`tauri.conf.json` 指定 `bundle.windows.wix.language = ["zh-CN"]`
+ `nsis.languages = ["SimpChinese"]`。

| 产物 | 大小 | 校验 |
|---|---|---|
| `归零_0.1.0_x64_zh-CN.msi` | 4.79 MB | ProductName=归零, ProductVersion=0.1.0, ProductLanguage=2052 |
| `归零_0.1.0_x64-setup.exe` (NSIS) | 3.34 MB | 生成成功 |

CI 新增 `release-build` job（windows-latest）断言两个产物都存在并上传 artifact。

## 2026-10-23 审计修复验证（历史轮 · 数字为当时状态）

本轮审计修复的每一项都带真实命令与结果。凡是未重跑的历史结论，均标注为「历史」。

### 修复项与回归网

| 缺陷 | 级别 | 回归测试 | 结果 |
|---|---|---|---|
| `encrypt_existing_data_db` 档案分支索引越界 panic（先填档案再设主密码必崩） | P0 | `cargo test --lib encrypt_existing_data_covers_all_profile_fields` | 修复前 `index out of bounds: the len is 10 but the index is 10`；修复后 PASS |
| 恢复成功后新恢复密钥 6 秒自动消失（旧码已失效） | P0 | `npx vitest run src/App.test.ts -t "新恢复密钥在用户确认抄写前不得自动消失"` | 修复前 FAIL；修复后 20 例 PASS |
| 托盘「锁定」事件无监听，界面仍显示已解密数据 | P1 | `npx vitest run src/App.test.ts` | 20 例 PASS（`bindTrayEvents` 在非 Tauri 环境为 no-op） |
| 深色模式三处硬编码白底（`.category-card` / `.modal-content` / `.confirm-card`） | P1 | `npx playwright test`（theme.spec + visual.spec） | 37 passed |
| `get_all_platforms_extended` O(n²) 重建平台表 | P2 | `cargo test --locked` | 81 passed |
| 报告加载失败静默（只有 console.error） | P2 | `npx vitest run src/views/ReportView.test.ts` | 8 例 PASS |

### 全量命令

| 命令 | 结果 |
|---|---|
| `npx vue-tsc --noEmit` | exit 0 |
| `npm test` | 22 files / **241 passed** |
| `npm run test:coverage` | statements 79.43 / branches 78.75 / functions 73.64 / lines 80.63（阈值 78/78/71/79） |
| `npx playwright test` | **37 passed**（31.4s） |
| `cargo fmt --check` | PASS |
| `cargo clippy --all-targets -- -D warnings` | PASS，0 warning |
| `cargo test --locked` | **81 passed, 0 failed, 1 ignored**（92.93s） |

### 文档与治理补齐

新增 `CONTRIBUTING.md`、`SECURITY.md`、`CODE_OF_CONDUCT.md`、`CHANGELOG.md`、`docs/ARCHITECTURE.md`、
`.gitattributes`、`.editorconfig`、`.nvmrc`、`.github/dependabot.yml`、
`.github/ISSUE_TEMPLATE/{bug_report,policy_correction,config}.yml`、`.github/PULL_REQUEST_TEMPLATE.md`、
`.github/workflows/release.yml`。

数字核对（以代码为准，逐条 grep 验证）：
- Tauri 注册命令 **42**（`commands.rs` 41 个 `#[command]` + `notify::evaluate_dms`）
- SQLite 表 **8**（`grep -c "CREATE TABLE" src-tauri/src/db.rs`）
- 内置平台 **152** 条 / **16** 分类（`PlatformEntry::{new,with_source}` 计数与 id 去重）
- 前端 mock 与 Rust 平台库 id 全集一致（152 vs 152，diff 为空）

### 明确未验证

- GitHub Actions 本轮改动后的运行结果：本机 `api.github.com` TLS 不可达，需在 Actions 页面确认。
- Windows 安装包未在本轮重新构建（`cargo test` 与前端构建产物已验证，`tauri:build` 未重跑）。
- 深色模式三处修复的**像素级**观感：由 `theme.spec.ts` 截图产出，未逐张人工比对。

## 2026-10-24 交付轮（v0.2.0）

### 修复与新增能力

| 项目 | 级别 | 回归网 | 结果 |
|---|---|---|---|
| CI Rust job 连续两次失败：`cargo audit --deny vulnerabilities` 是 0.22 已移除的取值 | 阻断 | 本地 `cargo audit`（默认遇漏洞 exit 1） | 0 漏洞、5 条上游 unmaintained/unsound 警告 |
| 资产列表加载失败被渲染成「还没有登记任何数字资产」 | P1 | `AssetsView.test.ts`（mock reject） | 12 例 PASS |
| 资产详情直接改 store，取消不撤销、保存失败无提示 | P1 | `AssetDetailView.test.ts`（失败留在页面） | 16 例 PASS |
| 危险确认框默认焦点落在「删除」上 | P1 | `FeedbackLayer.test.ts` | PASS |
| 导入备份后 store 全是旧数据 | P1 | `ProfileView.test.ts`（断言重取资产/统计） | 19 例 PASS |
| DMS 配置双事实源（保存档案会覆盖 DMS 页的间隔） | P1 | `ProfileView.test.ts`（断言不调用 `set_dead_mans_switch`） | PASS |
| 暗色三处浮层 `--gray-0` 未定义、侧栏 1.72:1、实心按钮 2.26–3.49:1、继承人提示卡硬编码浅黄 | P1 | Playwright theme/visual spec + 手工对比度计算 | 5.88–9.22:1 |
| 报告页加载中为空白壳、导出按钮可点但静默 | P2 | `ReportView.test.ts` | 8 例 PASS |
| 备份 v1 缺 heartbeat_config / platform_overrides | P2 | `backup_v2_covers_heartbeat_and_platform_overrides`、`legacy_v1_backup_imports_without_wiping_v2_config` | PASS |
| 命令层无输入上限（8KB 文本 / 估值有限性 / 平台 id / DMS 1–365 天 / 档案 500 字符） | P2 | `asset_commands_reject_oversized_and_invalid_payloads`、`dms_interval_must_be_within_range` | PASS |
| 心跳服务器 `/check` 无鉴权、任意令牌可写状态、`_alerted` 只在内存 | P2 | `server/heartbeat-server.test.mjs` | 9 断言 PASS |

### 本轮全量命令

| 命令 | 结果 |
|---|---|
| `npx vue-tsc --noEmit` | exit 0 |
| `npm test` | 26 files / **265 passed**，stderr 无未预期错误 |
| `npm run test:coverage` | 79.41 / 79.00 / 73.17 / 80.78（阈值 78/78/71/79） |
| `npm run build` | PASS；**产物中已无 mock 分块**（`dist/assets` 不含 `mock-*.js`） |
| `npm run build:demo` | PASS；`docs/demo/` 0.91 MB，hash 路由，浏览器实测无控制台错误 |
| `npx playwright test` | **37 passed** |
| `node server/heartbeat-server.test.mjs` | 9 assertions PASS |
| `cargo test --locked` | **86 passed, 0 failed, 1 ignored** |
| `cargo audit` | 0 漏洞（5 条 unmaintained/unsound 警告，来自 Tauri 上游） |

### 明确未验证

- macOS / Linux 安装包构建：仅 Windows 目标在 CI 中产出。
- 演示视频为 32 秒 1200×800 mp4，未做多语言字幕。

CI 已确认：run `40072253561`（sha `9a55a80`）5 个 job 全绿；Release run `40075221850` 的
build 与 verify 两个 job 全绿，Release `v0.2.0` 的 MSI/NSIS 已独立复核 SHA-256 一致。

### 收尾轮补充（路由与 IPC 入口的回归网）

`src/router.ts` 与 `src/api/invoke.ts` 是导航与 IPC 的唯一入口，此前 router 为 0% 覆盖。
本轮补 `src/router.test.ts`（5 例：404 兜底、深层未知路径、`document.title` 同步、
根路径重定向、scrollBehavior 恢复）与 `invoke.test.ts` 扩展（+3 例：纯前端生产构建报错、
`IS_WEB_PREVIEW` 在 dev/Tauri 两种环境下的取值）。

| 命令 | 结果 |
|---|---|
| `npx vitest run src/router.test.ts` | 5 passed |
| `npx vitest run src/api/invoke.test.ts` | 8 passed，invoke.ts 覆盖率 100% |
| `npm test` | 26 files / **265 passed** |
| `npm run test:coverage` | 79.41 / 79.00 / 73.17 / 80.78（阈值 78/78/71/79） |
| `node scripts/demo-media.mjs`（SKIP_VIDEO=1） | 10 张截图重出，标题断言全部匹配 |

`scripts/demo-media.mjs` 的 `title` 字段此前未被断言（只检查标题非空），现在会在标题与
路由预期不符时直接失败，避免交付错图。

### 双语官网与截图交付（2026-10-24 追加）

| 检查项 | 命令 / 方法 | 结果 |
|---|---|---|
| 内部链接完整性 | 解析 `docs/index.html` 与 `docs/zh/index.html` 的全部 href/src/poster，逐个 stat | 两页各 62 条，缺失 0 |
| HTML 结构 | 自写 HTMLParser 栈式校验 | 两页均无未闭合/错配标签 |
| 语言切换 | Playwright 点击 `.lang a`，双向验证 | `zh/#screenshots → en/#screenshots`、`en/#security → zh/#security` |
| 首屏截图 | `img.naturalWidth` 断言 | 2400（真实加载，非占位） |
| 截图区 | 逐段滚动触发 lazy load 后统计 | 10/10 加载成功 |
| 演示视频 | `video.readyState` + 元数据 | 32.28s / 1200×800，可播放 |
| 双语内容纯度 | 去标签后统计可见文本 | 英文页残留中文仅品牌名与「中文官网」入口；中文页仅代码块含英文 |
| README 锚点 | 标题 slug 与 `](#…)` 比对 | 两个 README 各 0 失效 |
| 线上部署 | `curl -o /dev/null -w %{http_code}` | `/`、`/zh/`、`/demo/` 均 200；三套镜像的 `/zh/` 均 200 |
| CI | GitHub Actions run | `2cf8aa4` 5 job 全绿 |

新增文件：`docs/site.css`（两站共用）、`docs/zh/index.html`（完整中文站）、`README.zh-CN.md`。

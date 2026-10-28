# 验证记录

本文件记录最终验证证据，避免"看起来绿"与"真的绿"混淆。

## 2026-10-28 发版核对轮（v0.2.2）

用户要求严格核对五个远端的上传/CI/官网/Release/分支五项事实。核对结论与修复如下。

### 核对时发现的真实问题（均已修复）

| 问题 | 证据 | 修复 |
|---|---|---|
| Gitee 落后 51 个提交 | `git ls-remote` gitee main = `49a78a9`，本地 `331c650` | 推送至 `b0d2cfd` |
| 三个 GitHub 镜像落后 51 个提交 | API `repos/*/digilegacy` `pushed_at` 停在 2026-10-26 | 全部推送至 `b0d2cfd` |
| 官网线上内容是 2026-10-24 的旧版 | 线上 `docs/index.html` sha256 == `49a78a9:docs/index.html`，仍写「86 Rust / 265 frontend」「AI 接口已预留」 | 更新中英双站并随本次提交部署 |
| 在线演示缺审核页等 4 轮功能 | `docs/demo` 停在 `a27f70f`（10-26 19:32），`grep ReviewView` 零命中 | `npm run build:demo` 重建并覆盖 |
| x33834 与 Morningstar 的 Pages 部署失败 | x33834 `deploy` job 报 `in progress deployment` 冲突；Pages 状态曾为 `errored` | 本次推送触发重部署，两个站均已 `built` 并返回 200 |
| Gitee / GitCode 只有 v0.1.0 Release | API `releases` 长度 1 | 见「未能修复」 |
| README 中英测试数与覆盖率过期 | 写 118 passing / 85.4%，实测 202 passed / 85.23% | 改为 202 / 85.2% |
| README 写「Signed build pipeline」但无签名配置 | `grep signtool\|certificate` 全空 | 改为「发布 MSI+NSIS + SHA-256 校验，未做代码签名」 |
| 分支不只有 main | GitHub 镜像有 `backup-before-v0.2.0` + 6 个 dependabot 分支，Gitee 有 1 个 | 备份分支先打 `archive-backup-before-v0.2.0` tag 再删；dependabot 分支随 PR 关闭清理 |

### 本轮真实执行

| 命令 | 结果 |
|---|---|
| `bash scripts/verify.sh --with-e2e` | **ALL GREEN**（12 项 PASS） |
| `npm run test:coverage` | 424 passed / 32 files；statements 85.23 / branches 79.34 / functions 77.73 / lines 86.73 |
| `npx playwright test` | **80 passed** |
| `cargo test --locked` | **202 passed / 1 ignored**（源码声明 204） |
| `node scripts/check-doc-facts.mjs` | exit 0（Rust 204 / 前端 424 / E2E 80 / IPC 69 / 表 13 / 平台 152） |
| 站点内部链接 | 中英各 59 条，缺失 0 |

### 发布产物（v0.2.2，badhope/digilegacy）

| 检查 | 结果 |
|---|---|
| Release workflow | build + verify-release 两个 job 全 success |
| 资产 | `DigiLegacy_0.2.2_x64-setup.exe` (3,915,816 B)、`DigiLegacy_0.2.2_x64_zh-CN.msi` (5,525,504 B)、`digest-sha256.txt` |
| 下载 + 校验 | 三个文件全部 200 下载成功，`sha256sum -c` **两个 OK** |
| CI on `b0d2cfd` | 5 个 job 全 success |

### 明确未能修复 / 未验证

- **Gitee 与 GitCode 的 Release 仍只有 v0.1.0 及其两个 EXE**：`v0.2.0/0.2.1/0.2.2` 的构建产物由 GitHub Actions 产出，Gitee/GitCode 无等价 CI；把 5.5MB MSI 与 3.9MB EXE 上传到这两个平台需要手工建 Release 并传附件（接口已确认可用，但本轮以「官网下载指向 GitHub」为准）。官网所有下载按钮指向 `github.com/badhope/digilegacy/releases`，因此用户下载路径不受影响。
- **代码签名**：无证书，安装包未签名，Windows SmartScreen 会提示未知发布者。README 已改为如实表述。
- **GitHub HTTPS 主 IP 不可达**：`github.com:443`（20.205.243.166）超时，`api.github.com` 间歇超时；git 推送改用 `http.curloptResolve` 指向 `140.82.121.4` 或 SSH（`ssh.github.com:443`）。这是本机网络环境，不是仓库问题。

## 2026-10-26 扩展升级轮（历史快照 · 数字为当时状态，最终数字见第三轮门禁）

| 命令 | 结果 |
|---|---|
| `npx vue-tsc --noEmit` | exit 0 |
| `npm run test:coverage` | **349 tests / 31 files**，statements 85.40%（branches 78.74 / functions 76.88 / lines 86.88，阈值 78/78/71/79） |
| `npm run build` | PASS；`dist/assets` 无 `mock-*.js` 分块 |
| `npx playwright test` | **76 passed**（52.2s，含场景矩阵 39 个） |
| `cargo test --locked` | **103 passed, 0 failed, 1 ignored** |
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
| 签到日志只写不读，用户无法自查上次确认 | P3 | `DMSView.test.ts::显示最近几次确认记录`、`db.rs::check_in_history_is_newest_first_and_capped` |
| `get_dms_status` 是界面唯一 DMS 数据源，若混入投递逻辑会重演 P0 | P1 | `commands.rs::get_dms_status_is_read_only_and_reports_stage`（含 dms_config 不被改动） |

## 2026-10-26 加固收尾（第三轮）

按上一轮评审的推荐逐项落地。

| 项 | 做法 | 验证 |
|---|---|---|
| CI 假绿灯 | `cargo-audit` 去 `|| true`、钉 0.22.1、打印 `--version` | `ci.yml:154-165`；本地 `cargo audit` 0 漏洞 |
| 文档数字漂移 | `doc-facts-lib.mjs` 单一推导 + `check-doc-facts.mjs` 门禁 + `sync-doc-facts.mjs` 修正 | `node scripts/check-doc-facts.mjs` exit 0（Rust 声明 120 / 前端 352 / E2E 76 / IPC 46） |
| 报告抓 DOM | 新增 `report.rs` 后端渲染，单文件与 ZIP 同源；含占位符拒绝导出 | `report.rs` 5 组测试 + `ReportView.test.ts` 2 例 + `commands.rs::render_report_response_refuses_placeholder_and_returns_full_document` |
| 「已解锁」不可信 | `security_status` 用验证器校验密钥；写路径同样拒绝错配密钥 | `commands.rs::security_status_is_false_when_key_does_not_match_verifier` |
| DMS 边界未披露 | 安心传承页新增边界区块 + 实际通道显示 | `DMSView.test.ts` 3 例 |
| `db.rs` 3.9k 行 | 拆 `db_platforms.rs` / `db_tests.rs`，生产代码 1882 行 | `cargo test --locked` 117 passed（纯移动；该数字为拆分当时快照，此后新增测试使其增长） |
| 导入层缺鲁棒性测试 | 513 个确定性畸形输入 | `scenarios.rs::backup_import_survives_500_malformed_inputs` |
| 死配置 | `stage3_wait_days` 补充说明 | `notify.rs` 注释 |
| 遗留截图 | 删除根目录 14 张（2.9MB） | `git ls-files screenshots` 为空 |

### 本轮新发现并修复

| 缺陷 | 级别 | 复现 | 回归 |
|---|---|---|---|
| `security_status` 只看「内存有 key」，恢复中途退出后密钥错配仍宣称已解锁 → 界面展示占位符并允许写回 | P0 入口 | 设正确密钥→换错密钥→`security_status().unlocked == true` | `security_status_is_false_when_key_does_not_match_verifier` |
| 备份 `user_profile` / `check_in_log` 非数组时被静默当空数组 → 导入"成功"却清空档案 | P1 | `{"user_profile":"x"}` 返回 Ok | `backup_import_survives_500_malformed_inputs` |

### 本轮验证

| 命令 | 结果 |
|---|---|
| `cargo test --locked` | **118 passed, 0 failed, 1 ignored**（源码声明 120，含 1 条 `#[cfg(unix)]` 在 Windows 不编译） |
| `cargo clippy --all-targets -- -D warnings` | 0 warning |
| `cargo fmt --check` | PASS |
| `npx vue-tsc --noEmit` | exit 0 |
| `npx vitest run` | **352 passed / 31 files** |
| `node scripts/check-doc-facts.mjs` | exit 0（文档与源码一致；反向验证：手改一个数字后 exit 1） |

## 2026-10-26 对抗式 debug（第二轮）

用「先复现、再修复、留回归」的方式逐条验证，不复述实现。

| 缺陷 | 级别 | 复现证据 | 回归测试 |
|---|---|---|---|
| 解密占位符被写回，真实密文被永久覆盖（资产详情 + 我的档案） | **P0** | 用错钥匙读出 `【加密数据 · 需解锁】` 后保存 → 用正确钥匙再读仍是占位符 | `db.rs::asset_placeholder_cannot_be_written_back`、`db.rs::profile_placeholder_cannot_be_written_back`、`AssetDetailView.test.ts`、`ProfileView.test.ts` |
| 锁定态可删除资产（锁定只防读不防破坏） | **P0** | `delete_asset_db` 在 `vault::clear_key()` 后仍返回 `Ok(())` | `db.rs::delete_asset_rejected_while_locked` |
| 政策补丁新增平台无法登记资产 | **P1** | `add_asset(platform_id=9001)` → `FOREIGN KEY constraint failed` | `db.rs::patch_only_platform_can_hold_assets` |
| 补丁平台上有资产时清除补丁直接失败 | **P1** | `clear_platform_overrides_db()` → `FOREIGN KEY constraint failed` | `scenarios.rs::clearing_patch_keeps_assets_on_patch_only_platforms` |
| 启用 DMS 但从未签到 → 永不触发 | **P1** | `enabled=true, last_check_in=NULL` → `overdue=false, stage=0` 恒成立 | `notify.rs::enabled_dms_anchors_on_first_enable_instead_of_never_firing` |
| 外部链接 userinfo 欺骗 | **P2** | `https://trusted.example.com@evil.example.com/steal` 通过校验 | `vault.rs::url_validation_rejects_userinfo_and_control_chars` |
| 发件箱按默认权限落盘（含收件人联系方式） | **P2** | 用 `OpenOptions` 直接创建，无非 Unix 0600 | `fsutil.rs` + `notify.rs` 写入路径 |
| 仪表板/报告总价值未过滤负值与超大值 | **P2** | `SUM(estimated_value)` 直接累加 | `db.rs::totals_ignore_negative_and_absurd_values` |
| 继承人向导「可继承/难继承」硬编码中文 | **P3** | 英文界面仍显示中文 | i18n 键已存在，改为 `t()` |

本轮验证：`cargo test --locked` **110 passed / 1 ignored**；`cargo clippy --all-targets -- -D warnings` 0 warning；`cargo fmt --check` PASS；`npx vitest run` **347 passed / 31 files**；`npm run test:coverage` exit 0（statements 85.40 / branches 78.74 / functions 76.88 / lines 86.88）；`npx playwright test` **76 passed**。

## 2026-10-26 真实使用场景模拟（大规模）

除回归单测外，本轮新增两套「按真实使用路径做矩阵遍历」的场景模拟，逐检查点断言不变量。

### 后端（真实 SQLite + 真实 AES，`src-tauri/src/scenarios.rs`）

| 场景族 | 规模 | 检查点 | 覆盖内容 |
|---|---|---|---|
| 资产生命周期矩阵 | 3 类别 × 7 处置 × 3 金额形态 × 加密/明文 = 126 | **1386** | 新增→读回→更新→分类查询→统计→删除，每步校验字段未被改写、统计自洽 |
| 备份往返矩阵 | 30 轮 × 5 资产组合 | **692** | v3 导出→清空→导入逐字段比对；v1 旧格式兼容；4 类损坏输入 fail-closed 且数据不变 |
| 凭据保险库矩阵 | 20 资产 × 加密/明文 | **180** | 锁定态拒绝、密文以 `enc:v1:` 落库、`has_credential` 同步、清除后不可读 |
| DMS 时间线矩阵 | 7 间隔 × 10 偏移 = 70 | **350** | 逾期判定、三级分级、签到后归零，与界面判定口径一致 |
| 政策补丁矩阵 | 30 轮 | **216** | 新增/覆盖判定、导出、清除回落到内置 152 条 |
| **合计** | **五个矩阵族** | **2824** | |

> 口径说明：组合数按「轮次 × 资产数」等不同定义，各矩阵族的「组合」不可加总，因此合计只报可执行的检查点数。

### 前端（真实 Pinia store + 真实 mock 后端，`src/scenarios.test.ts`）

| 场景族 | 规模 | 检查点 |
|---|---|---|
| 资产 store × 类别 × 处置 × 金额 | 3×7×8 = 168 | **1008** |
| 筛选/排序/搜索 | 4 类别 × 5 关键词 × 4 排序 = 80 | **324** |
| 格式化 × 语言 × 金额 | 2×12 | **88** |
| 枚举文案 × 语言 | 2×(3+7+2) | **24** |
| 锁定态 × 写路径 | 5 条命令 | **12** |
| 锁定态批量读取 | 12 资产 | **36** |
| mock 资产生命周期 | 20 轮 | **160** |
| 占位符识别（资产 5 字段） | 5 字段 × 正常/占位 | **10** |
| 档案占位符（12 字段） | 12 字段 | **12** |
| 平台缓存失效 | 10 轮 × 3 态 | **40** |
| **合计** | **十个矩阵族** | **1714** |

### 浏览器（真实 Chromium，`e2e/scenarios.spec.ts`）

| 场景族 | 规模 | 断言要点 |
|---|---|---|
| 路由 × 语言 × 主题 | 8 × 2 × 2 = **32** | 标题随语言、`data-theme` 落地、`html[lang]` 同步、无控制台错误、无横向溢出、主体非空 |
| 平台搜索关键词 | **20** | 每个关键词结果都包含关键词，不出现不相关平台 |
| 资产筛选 × 排序 | 4 × 4 = **16** | 组合后行数合法、无重复行 |
| 法律模板分类 | **6** | 每类都有模板且预览可开可关 |
| 继承人向导 Tab × 难度 | **2×4 + 折叠** | Tab 选中态、难度筛选、展开/收起确实改变条目数 |
| 安心传承幂等 | **10** | 连续签到 5 次 + 保存设置 5 次无错误 |
| 继承报告 | **6** | 正文完整、导出/工具包按钮可用、打印真实触发 |
| 语言/主题往返 | 8 × 2 = **16** | 每页切语言标题更新、主题切换生效 |
| **合计** | **39 个 E2E 场景** | 全绿（48.9s） |

### 本轮场景模拟抓出的真实缺陷

| 缺陷 | 级别 | 修复 | 回归 |
|---|---|---|---|
| 备份导入完全不校验 `category` / `disposal_method` / `estimated_value`，手改 JSON 可把非法枚举与负金额写进统计与报告 | P1 | `db.rs` 导入循环加枚举与金额校验，事务内拒绝且不留半截数据 | `backup_import_rejects_invalid_asset_fields` |
| `assets.totalValue` 用 `|| 0` 累加，负金额会原样相加，总价值可显示为负数 | P2 | 只累加有限且非负的估值，与后端统计口径一致 | `scenarios.test.ts::资产 store 矩阵` |
| Web 演示 mock 的 `add_asset` 不校验类别/处置/金额，与桌面版行为不一致 | P2 | mock 补同等边界；合法值仍放行 | `mock.test.ts::非法类别/处置/负金额…被拒绝` |

### 明确未覆盖

- 真实 SMTP 投递（需外部邮件服务器）与真实 Tauri WebView 自动化（需桌面窗口）不在场景模拟内；DMS 的分级与发件箱落盘已由 Rust 场景与单测覆盖。
- 场景模拟使用确定性伪随机序列（可复现），不是全量状态空间穷举。

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

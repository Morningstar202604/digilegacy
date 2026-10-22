# 验证记录

本文件记录最终验证证据，避免"看起来绿"与"真的绿"混淆。

## 本地门禁（Windows，`bash scripts/verify.sh`）

| 步骤 | 命令 | 结果 |
|---|---|---|
| 依赖一致性 | `npm ci` | PASS |
| 类型检查 | `npx vue-tsc --noEmit` | PASS (exit 0) |
| 测试 + 覆盖率门禁 | `npm run test:coverage` | 216 tests / 21 files，statements 76.6%（77.4 / 69.6 / 77.5，阈值 75/76/68/76） |
| 生产构建 | `npm run build` | PASS |
| E2E（Playwright） | `npx playwright test` | **31 passed**（7 个视图 + 完整发现链路 + 13 项全站 UI 健全性） |
| 格式 | `cargo fmt --check` | PASS |
| Lint | `cargo clippy --all-targets -- -D warnings` | 0 warnings |
| Rust 测试 | `cargo test --locked` | 74 passed, 0 failed, 1 ignored |

## 跨平台验证（Ubuntu 24.04 容器，真实 Linux）

CI 的 Rust job 在 linux 上跑，本地是 Windows。为避免"本地绿、CI 红"，在
Ubuntu 24.04 容器内用 Rust 1.99.1 完整复跑：

| 步骤 | 结果 |
|---|---|
| `cargo fmt --check` | PASS |
| `cargo clippy --all-targets -- -D warnings` | PASS，0 warnings |
| `cargo test --locked` | 59 passed, 0 failed, 1 ignored（68.04s，当时套件；此后用例增加至 66，容器未重跑） |

## E2E 跨平台验证（Ubuntu 24.04 容器）

E2E 首次在 Linux 容器内运行时 18 个用例全失败，排查后确认**不是测试代码问题**：
容器内 Playwright 浏览器二进制版本（1243）与 lockfile 要求（1248）不匹配，
`npx playwright install` 因 191MB 包在容器网络下超时未完成。

改用容器内已存在的 chromium 显式指定 `executablePath` 后复跑：

| 结果 | 耗时 |
|---|---|
| **18 passed / 0 failed** | 27.5s |

结论：E2E 用例本身跨平台可跑通；CI 上浏览器由 `npx playwright install --with-deps chromium`
安装，job 已设 `timeout-minutes: 15` 避免大包下载被截断。

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

依赖链 engines 要求 `^22.22.2 || ^24.15.0 || >=26`。本机 Node 24.14.0 **不满足**，
CI 固定 `22.22.2`。已在 Node 22.22.2 下复跑前端全量：110 tests 通过、覆盖率一致（补测后为 122）。

## 未能在本机验证的部分

- **GitHub Actions 运行结果**：`api.github.com` 在本机 TLS 不可达（curl exit 35），
  代理 `ghfast.top` 仅支持 git 协议。首次成功运行已由独立评审通过 GitHub API
  核实：run `39558924027`（sha `3fc3b68`）**completed / success**，三个 job 全绿。
  后续提交的构建结果需在 Actions 页面确认：
  https://github.com/badhope/digilegacy/actions

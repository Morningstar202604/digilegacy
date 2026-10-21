# 验证记录

本文件记录最终验证证据，避免"看起来绿"与"真的绿"混淆。

## 本地门禁（Windows，`bash scripts/verify.sh`）

| 步骤 | 命令 | 结果 |
|---|---|---|
| 依赖一致性 | `npm ci` | PASS |
| 类型检查 | `npx vue-tsc --noEmit` | PASS (exit 0) |
| 测试 + 覆盖率门禁 | `npm run test:coverage` | 110 tests / 11 files，statements 34.33%（阈值 33） |
| 生产构建 | `npm run build` | PASS |
| 格式 | `cargo fmt --check` | PASS |
| Lint | `cargo clippy --all-targets -- -D warnings` | 0 warnings |
| Rust 测试 | `cargo test --locked` | 59 passed, 0 failed, 1 ignored |

## 跨平台验证（Ubuntu 24.04 容器，真实 Linux）

CI 的 Rust job 在 linux 上跑，本地是 Windows。为避免"本地绿、CI 红"，在
Ubuntu 24.04 容器内用 Rust 1.99.1 完整复跑：

| 步骤 | 结果 |
|---|---|
| `cargo fmt --check` | PASS |
| `cargo clippy --all-targets -- -D warnings` | PASS，0 warnings |
| `cargo test --locked` | **59 passed, 0 failed, 1 ignored**（68.04s） |

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
CI 固定 `22.22.2`。已在 Node 22.22.2 下复跑前端全量：110 tests 通过、覆盖率一致。

## 未能在本机验证的部分

- **GitHub Actions 运行结果**：`api.github.com` 在本机 TLS 不可达（curl exit 35），
  代理 `ghfast.top` 仅支持 git 协议，不支持 HTTPS API/网页查询。
  已确认代码推送到 `github.com/badhope/digilegacy` 的 main（`995056c..3fc3b68`），
  工作流会在推送时触发，但**运行结论需要在 Actions 页面确认**：
  https://github.com/badhope/digilegacy/actions

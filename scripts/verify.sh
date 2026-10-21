#!/usr/bin/env bash
# 本地门禁：与 CI 完全一致的检查序列，任一失败立即退出非零。
# 用法：bash scripts/verify.sh [--rust-only|--web-only]
#
# 关键约定：不使用 `cmd | tail` 这种写法判断成败——管道的退出码取自最后一个命令，
# 会把失败掩盖成 0（本项目曾因此误判「全绿」）。这里统一显式记录退出码。

set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WEB="$ROOT/digilegacy"
TAURI="$WEB/src-tauri"
MODE="${1:-all}"
FAILED=0

run() {
  local label="$1"; shift
  printf '\n==> %s\n' "$label"
  if "$@"; then
    printf '  PASS: %s\n' "$label"
  else
    local code=$?
    printf '  FAIL(%d): %s\n' "$code" "$label"
    FAILED=1
  fi
}

have() { command -v "$1" >/dev/null 2>&1; }

if [ "$MODE" != "--rust-only" ]; then
  echo "################ 前端 ################"
  run "依赖安装（ci 一致性）" bash -c "cd '$WEB' && npm ci --silent"
  run "类型检查" bash -c "cd '$WEB' && npx vue-tsc --noEmit"
  run "单元测试 + 覆盖率门禁" bash -c "cd '$WEB' && npm run test:coverage"
  run "生产构建" bash -c "cd '$WEB' && npm run build"
  # E2E 默认跳过（需先 npx playwright install chromium）；--with-e2e 显式开启
  if [ "$MODE" = "--with-e2e" ]; then
    run "E2E（Playwright）" bash -c "cd '$WEB' && npx playwright test"
  fi
fi

if [ "$MODE" != "--web-only" ]; then
  echo "################ Rust ################"
  CARGO="cargo"
  have cargo || { [ -x "$HOME/.cargo/bin/cargo" ] && CARGO="$HOME/.cargo/bin/cargo"; }
  run "格式检查" bash -c "cd '$TAURI' && '$CARGO' fmt --check"
  run "Clippy（-D warnings）" bash -c "cd '$TAURI' && '$CARGO' clippy --all-targets -- -D warnings"
  run "测试" bash -c "cd '$TAURI' && '$CARGO' test --locked"
fi

echo
if [ "$FAILED" -ne 0 ]; then
  echo "结果：FAILED"
  exit 1
fi
echo "结果：ALL GREEN"

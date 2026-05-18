#!/bin/bash
set -euo pipefail

# 一键部署后端到 jd（systemd + cargo build 方式）
# 用法：cd football_insight_service_backend_rs && bash deploy/deploy-backend-binary.sh

MONOREPO_DIR="$(cd "$(dirname "$0")/../.." && pwd)"
BACKEND_DIR="$MONOREPO_DIR/football_insight_service_backend_rs"
JD_HOST="jd"
JD_MONOREPO="/root/projects/football_insight"

echo "=== Football Insight Backend Deploy (Binary) ==="
echo ""

# 1. 检查本地变更
cd "$MONOREPO_DIR"
git fetch origin main --quiet 2>/dev/null || true

LOCAL=$(git rev-parse HEAD)
REMOTE=$(git rev-parse origin/main 2>/dev/null || echo "")

if [ -n "$REMOTE" ] && [ "$LOCAL" != "$REMOTE" ]; then
    AHEAD=$(git rev-list --count origin/main..HEAD)
    BEHIND=$(git rev-list --count HEAD..origin/main)
    if [ "$AHEAD" -gt 0 ] && [ "$BEHIND" -eq 0 ]; then
        echo "本地已 commit 但未 push（ahead $AHEAD）"
        git push
        echo "已 push"
    elif [ "$BEHIND" -gt 0 ]; then
        echo "⚠️ 远程比本地新（behind $BEHIND），请先 pull 合并"
        exit 1
    fi
fi

if git diff --quiet HEAD && git diff --quiet --cached; then
    echo "本地没有变更，跳过 commit"
    VERSION=$(git rev-parse --short HEAD)
else
    echo "本地有未提交变更"
    git status --short
    echo ""
    read -rp "请输入 commit message（直接回车使用默认）: " msg
    if [ -z "$msg" ]; then
        msg="deploy(backend): update binary"
    fi
    git add -A
    git commit -m "$msg"
    git push
    VERSION=$(git rev-parse --short HEAD)
    echo "已 push commit: $VERSION"
fi

echo ""
echo "=== Step 1/3: Commit & Push OK ==="
echo ""

# 2. jd 拉取 + 编译 + 重启
ssh "$JD_HOST" "
    set -e
    cd $JD_MONOREPO
    git pull
    echo '代码已更新'

    cd $JD_MONOREPO/football_insight_service_backend_rs
    echo '开始编译...'
    cargo build --release
    echo '编译完成'

    systemctl restart football-insight.service
    echo '服务已重启'

    sleep 2
    systemctl status football-insight.service --no-pager
"

echo ""
echo "=== Step 2/3: Build & Restart OK ==="
echo ""

# 3. 健康检查
echo "等待服务启动..."
sleep 5

API=$(ssh "$JD_HOST" "curl -fsS http://127.0.0.1:8092/api/health 2>/dev/null || echo 'unhealthy'")

echo ""
echo "=== Step 3/3: Health Check ==="
echo "API health: $API"
echo ""

if echo "$API" | grep -q '"status":"ok"'; then
    echo "部署完成 ✅  commit: $VERSION"
else
    echo "部署可能有问题，请检查日志 ⚠️"
    ssh "$JD_HOST" "journalctl -u football-insight.service --no-pager -n 20"
    exit 1
fi

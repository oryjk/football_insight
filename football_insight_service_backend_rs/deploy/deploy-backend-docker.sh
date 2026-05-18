#!/bin/bash
set -euo pipefail

# 一键部署后端到 jd
# 用法：cd football_insight_service_backend_rs && bash deploy/deploy-backend.sh

MONOREPO_DIR="$(cd "$(dirname "$0")/../.." && pwd)"
BACKEND_DIR="$MONOREPO_DIR/football_insight_service_backend_rs"
OUT109_HOST="out109"
JD_HOST="jd"
HARBOR_IMAGE="harbor.oryjk.cn:82/library/football-insight-service-backend-rs"
JD_ENV_FILE="/root/projects/football_insight/football_insight_service_backend_rs/.env"
JD_LOGS_DIR="/root/docker_data/football_insight/logs"

echo "=== Football Insight Backend Deploy ==="
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
        msg="deploy(backend): update backend image"
    fi
    git add -A
    git commit -m "$msg"
    git push
    VERSION=$(git rev-parse --short HEAD)
    echo "已 push commit: $VERSION"
fi

echo ""
echo "=== Step 1/4: Commit & Push OK ==="
echo ""

# 2. out109 拉取 + build
ssh "$OUT109_HOST" "
    set -e
    cd ~/projects/football_insight_monorepo
    git pull
    echo 'Build image: $VERSION'
    cd football_insight_service_backend_rs
    docker build -t $HARBOR_IMAGE:$VERSION -t $HARBOR_IMAGE:latest .
    docker push $HARBOR_IMAGE:$VERSION
    docker push $HARBOR_IMAGE:latest
    echo 'Push OK'
"

echo ""
echo "=== Step 2/4: Build & Push OK ==="
echo ""

# 3. jd 拉取 + 重启
ssh "$JD_HOST" "
    set -e
    docker pull $HARBOR_IMAGE:$VERSION
    echo 'Pull OK'

    # 停止旧容器
    docker stop football-insight-service-backend-rs 2>/dev/null || true
    docker rm football-insight-service-backend-rs 2>/dev/null || true

    # 修复日志目录权限
    mkdir -p $JD_LOGS_DIR
    chown -R 10001:10001 $JD_LOGS_DIR

    # 启动新容器
    docker run -d \\
        --name football-insight-service-backend-rs \\
        --restart unless-stopped \\
        --network host \\
        --env-file $JD_ENV_FILE \\
        -v $JD_LOGS_DIR:/app/logs \\
        $HARBOR_IMAGE:$VERSION

    echo 'Container started'
    sleep 3
    docker ps --filter name=football-insight-service-backend-rs --format 'table {{.Names}}\t{{.Status}}'
"

echo ""
echo "=== Step 3/4: Deploy OK ==="
echo ""

# 4. 健康检查
echo "等待 health check (约 30s)..."
sleep 30

HEALTH=$(ssh "$JD_HOST" "docker inspect --format='{{.State.Health.Status}}' football-insight-service-backend-rs 2>/dev/null || echo 'unknown'")
API=$(ssh "$JD_HOST" "curl -fsS http://127.0.0.1:8092/api/health 2>/dev/null || echo 'unhealthy'")

echo ""
echo "=== Step 4/4: Health Check ==="
echo "Docker health: $HEALTH"
echo "API health:    $API"
echo ""

if [ "$HEALTH" = "healthy" ] || [ "$HEALTH" = "starting" ]; then
    echo "部署完成 ✅  镜像: $HARBOR_IMAGE:$VERSION"
else
    echo "部署可能有问题，请检查日志 ⚠️"
    ssh "$JD_HOST" "docker logs --tail 20 football-insight-service-backend-rs"
    exit 1
fi

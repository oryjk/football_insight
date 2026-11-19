#!/usr/bin/env bash
set -euo pipefail

# Build the Rust backend Docker image on local233, stream it to peiqian via SSH,
# optionally run embedded SQLx migrations before cutover, restart the production
# container, verify health, and print a rollback command.

SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
REPO_ROOT="$SCRIPT_DIR"
BACKEND_DIR="$REPO_ROOT/football_insight_service_backend_rs"

BRANCH="${DEPLOY_BRANCH:-main}"
DEPLOY_HOST="${DEPLOY_HOST:-peiqian}"
DEPLOY_MONOREPO_DIR="${DEPLOY_MONOREPO_DIR:-/root/projects/football_insight}"
DEPLOY_DIR="${DEPLOY_DIR:-${DEPLOY_MONOREPO_DIR}/football_insight_service_backend_rs}"
DEPLOY_RUNTIME_ENV_FILE="${DEPLOY_RUNTIME_ENV_FILE:-${DEPLOY_MONOREPO_DIR}/football-insight-service-backend-rs.env}"
DEPLOY_LOGS_DIR="${DEPLOY_LOGS_DIR:-${DEPLOY_DIR}/logs}"
IMAGE_NAME="${IMAGE_NAME:-football-insight-service-backend-rs}"
CONTAINER_NAME="${CONTAINER_NAME:-football-insight-service-backend-rs}"
PORT="${PORT:-8092}"
KEEP_IMAGES="${KEEP_IMAGES:-5}"
PUBLIC_CHECK_URL="${PUBLIC_CHECK_URL:-https://match.oryjk.cn/api/v1/system/public-config}"

RUN_MIGRATIONS="${RUN_MIGRATIONS:-0}"
SKIP_PULL="${SKIP_PULL:-0}"
DRY_RUN=0

usage() {
    cat <<'EOF'
在 local233 构建后端 Docker 镜像并部署到 peiqian 生产环境。

用法:
  ./deploy_peiqian_from_local233.sh [选项]

选项:
  --migrate     切换容器前使用新镜像执行 /usr/local/bin/run_migrations（幂等）
  --skip-pull   跳过 git pull --ff-only（仍会校验工作区干净且与 origin/main 一致）
  --dry-run     仅打印将执行的步骤和镜像 tag，不实际构建或部署
  -h, --help    显示本帮助

常用环境变量:
  DEPLOY_HOST=peiqian
  DEPLOY_BRANCH=main
  KEEP_IMAGES=5
  PUBLIC_CHECK_URL=https://match.oryjk.cn/api/v1/system/public-config
EOF
}

while (($# > 0)); do
    case "$1" in
        --migrate)
            RUN_MIGRATIONS=1
            shift
            ;;
        --skip-pull)
            SKIP_PULL=1
            shift
            ;;
        --dry-run)
            DRY_RUN=1
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            echo "❌ 未知参数: $1" >&2
            usage >&2
            exit 2
            ;;
    esac
done

for cmd in git docker gzip ssh curl; do
    command -v "$cmd" >/dev/null 2>&1 || {
        echo "❌ 缺少必需命令: $cmd" >&2
        exit 1
    }
done

cd "$REPO_ROOT"

CURRENT_BRANCH=$(git branch --show-current)
if [[ "$CURRENT_BRANCH" != "$BRANCH" ]]; then
    echo "❌ 当前分支为 '${CURRENT_BRANCH:-detached}'，请先切到 '${BRANCH}'" >&2
    exit 1
fi

if ! git diff --quiet || ! git diff --cached --quiet; then
    echo "❌ 本地工作区有未提交修改，请先提交或清理后再部署生产" >&2
    git status -s >&2
    exit 1
fi

echo "🔎 同步远端 origin/${BRANCH}..."
git fetch origin "$BRANCH"
if [[ "$SKIP_PULL" != "1" ]]; then
    git pull --ff-only origin "$BRANCH"
fi

LOCAL_HEAD=$(git rev-parse HEAD)
REMOTE_HEAD=$(git rev-parse "origin/${BRANCH}")
if [[ "$LOCAL_HEAD" != "$REMOTE_HEAD" ]]; then
    echo "❌ 本地 HEAD 与 origin/${BRANCH} 不一致，拒绝部署未推送/落后的代码" >&2
    echo "local : $LOCAL_HEAD" >&2
    echo "remote: $REMOTE_HEAD" >&2
    exit 1
fi

IMAGE_TAG="${IMAGE_TAG:-$(git rev-parse --short HEAD)}"
IMAGE_REF="${IMAGE_NAME}:${IMAGE_TAG}"
LATEST_REF="${IMAGE_NAME}:latest"

echo "🚀 准备部署后端生产镜像"
echo "   目标主机 : ${DEPLOY_HOST}"
echo "   分支提交 : ${BRANCH} @ ${IMAGE_TAG}"
echo "   镜像标签 : ${IMAGE_REF}"
echo "   执行迁移 : $([[ "$RUN_MIGRATIONS" == "1" ]] && echo "是 (--migrate)" || echo "否")"

if [[ "$DRY_RUN" == "1" ]]; then
    echo "🧪 [dry-run] 校验通过，未执行构建与部署。"
    exit 0
fi

PREV_IMAGE=$(ssh "$DEPLOY_HOST" "docker inspect -f '{{.Config.Image}}' '${CONTAINER_NAME}' 2>/dev/null || true")
if [[ -n "$PREV_IMAGE" ]]; then
    echo "📌 当前线上运行镜像: ${PREV_IMAGE}"
else
    echo "📌 当前线上未检测到运行中的容器: ${CONTAINER_NAME}"
fi

echo "📦 [1/4] 在 local233 构建 Docker 镜像 ${IMAGE_REF}..."
docker build -t "$IMAGE_REF" -t "$LATEST_REF" "$BACKEND_DIR"

echo "🚚 [2/4] 压缩传输镜像到 ${DEPLOY_HOST} 并加载..."
docker save "$IMAGE_REF" "$LATEST_REF" | gzip | ssh "$DEPLOY_HOST" 'gunzip | docker load'

if [[ "$RUN_MIGRATIONS" == "1" ]]; then
    echo "🗄️  [3/4] 切容器前在 ${DEPLOY_HOST} 执行数据库迁移..."
    ssh "$DEPLOY_HOST" "docker run --rm --network host --env-file '${DEPLOY_RUNTIME_ENV_FILE}' --entrypoint /usr/local/bin/run_migrations '${IMAGE_REF}'"
else
    echo "⏭️  [3/4] 跳过数据库迁移（如需运行请加 --migrate）"
fi

echo "🔄 [4/4] 在 ${DEPLOY_HOST} 重启容器并做健康检查..."
ssh "$DEPLOY_HOST" \
    "DEPLOY_RUNTIME_ENV_FILE='${DEPLOY_RUNTIME_ENV_FILE}' DEPLOY_LOGS_DIR='${DEPLOY_LOGS_DIR}' IMAGE_NAME='${IMAGE_NAME}' IMAGE_REF='${IMAGE_REF}' IMAGE_TAG='${IMAGE_TAG}' KEEP_IMAGES='${KEEP_IMAGES}' CONTAINER_NAME='${CONTAINER_NAME}' PORT='${PORT}' PREV_IMAGE='${PREV_IMAGE}' bash -s" << 'EOF'
set -euo pipefail

if [[ ! -f "${DEPLOY_RUNTIME_ENV_FILE}" ]]; then
    echo "❌ 未找到生产运行时 env 文件: ${DEPLOY_RUNTIME_ENV_FILE}" >&2
    exit 1
fi

mkdir -p "${DEPLOY_LOGS_DIR}"
chown -R 10001:10001 "${DEPLOY_LOGS_DIR}" || true

docker rm -f "${CONTAINER_NAME}" >/dev/null 2>&1 || true

docker run -d \
    --name "${CONTAINER_NAME}" \
    --restart unless-stopped \
    --network host \
    --env-file "${DEPLOY_RUNTIME_ENV_FILE}" \
    -e PORT="${PORT}" \
    -v "${DEPLOY_LOGS_DIR}:/app/logs" \
    "${IMAGE_REF}"

healthy=0
for i in $(seq 1 15); do
    if curl -fsS "http://127.0.0.1:${PORT}/api/health" >/dev/null 2>&1; then
        healthy=1
        break
    fi
    sleep 2
done

if [[ "$healthy" != "1" ]]; then
    echo "❌ 容器健康检查失败，最近日志：" >&2
    docker logs --tail 50 "${CONTAINER_NAME}" >&2 || true
    exit 1
fi

docker ps --filter "name=${CONTAINER_NAME}"

echo "🧹 清理旧镜像（保留最近 ${KEEP_IMAGES} 个 tag）..."
PREV_TAG=""
if [[ "${PREV_IMAGE}" == "${IMAGE_NAME}:"* ]]; then
    PREV_TAG="${PREV_IMAGE#${IMAGE_NAME}:}"
fi
docker images --format '{{.CreatedAt}}\t{{.Tag}}' "${IMAGE_NAME}" \
    | grep -v $'\tlatest$' \
    | sort -r \
    | tail -n +$((KEEP_IMAGES + 1)) \
    | cut -f2 \
    | while IFS= read -r old_tag; do
        [[ -z "$old_tag" || "$old_tag" == "$IMAGE_TAG" || "$old_tag" == "$PREV_TAG" ]] && continue
        docker rmi "${IMAGE_NAME}:${old_tag}" >/dev/null 2>&1 || true
    done
EOF

echo "🌐 验证线上公网入口: ${PUBLIC_CHECK_URL}"
curl -fsS "$PUBLIC_CHECK_URL" >/dev/null

echo "✅ 生产部署完成: ${IMAGE_REF} 已在 ${DEPLOY_HOST} 健康运行"
if [[ -n "$PREV_IMAGE" && "$PREV_IMAGE" != "$IMAGE_REF" ]]; then
    echo "↩️  如需回滚到上一版本，可执行:"
    echo "   ssh ${DEPLOY_HOST} \"docker rm -f ${CONTAINER_NAME} && docker run -d --name ${CONTAINER_NAME} --restart unless-stopped --network host --env-file ${DEPLOY_RUNTIME_ENV_FILE} -e PORT=${PORT} -v ${DEPLOY_LOGS_DIR}:/app/logs ${PREV_IMAGE}\""
fi

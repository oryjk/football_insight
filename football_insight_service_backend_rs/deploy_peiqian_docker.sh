#!/bin/bash

# Build the football insight backend image on peiqian itself and deploy it
# straight from the local image — no Harbor round trip. Images stay on
# peiqian; every deploy prunes history to keep the newest 10 tags.
# The Dockerfile already pulls Rust dependencies through rsproxy + China
# mirrors. Git sync goes through the local clash proxy (proxyOn) because
# direct GitHub access is unreliable from peiqian.
#
# Required flow:
#   1. Commit and push locally
#   2. peiqian pulls the pushed commit into /root/projects/football_insight
#   3. peiqian builds the image locally and prunes old tags (keep newest 10)
#   4. peiqian restarts the container from the local image and health-checks

set -euo pipefail

BRANCH="${DEPLOY_BRANCH:-main}"
DEPLOY_HOST="${DEPLOY_HOST:-peiqian}"
DEPLOY_REPO_URL="${DEPLOY_REPO_URL:-https://github.com/oryjk/football_insight.git}"
DEPLOY_MONOREPO_DIR="${DEPLOY_MONOREPO_DIR:-/root/projects/football_insight}"
DEPLOY_DIR="${DEPLOY_DIR:-${DEPLOY_MONOREPO_DIR}/football_insight_service_backend_rs}"
DEPLOY_RUNTIME_ENV_FILE="${DEPLOY_RUNTIME_ENV_FILE:-${DEPLOY_MONOREPO_DIR}/football-insight-service-backend-rs.env}"
DEPLOY_LOGS_DIR="${DEPLOY_LOGS_DIR:-${DEPLOY_DIR}/logs}"

IMAGE_NAME="${IMAGE_NAME:-football-insight-service-backend-rs}"
IMAGE_TAG="${IMAGE_TAG:-$(git rev-parse --short HEAD)}"
IMAGE_REF="${IMAGE_NAME}:${IMAGE_TAG}"
LATEST_REF="${IMAGE_NAME}:latest"
KEEP_IMAGES="${KEEP_IMAGES:-10}"
CONTAINER_NAME="${CONTAINER_NAME:-football-insight-service-backend-rs}"
PORT="${PORT:-8092}"

echo "🚀 Docker 本地镜像部署到 ${DEPLOY_HOST}"
echo "image: ${IMAGE_REF}（仅保留本地，最多 ${KEEP_IMAGES} 个历史 tag）"

echo "🔎 检查本地提交是否已 push..."
git -c http.version=HTTP/1.1 fetch origin "${BRANCH}"
LOCAL_HEAD="$(git rev-parse HEAD)"
REMOTE_HEAD="$(git rev-parse "origin/${BRANCH}")"

if [ "${LOCAL_HEAD}" != "${REMOTE_HEAD}" ]; then
    echo "❌ 当前 HEAD 尚未推送到 origin/${BRANCH}"
    echo "local : ${LOCAL_HEAD}"
    echo "remote: ${REMOTE_HEAD}"
    exit 1
fi

ssh "${DEPLOY_HOST}" \
    "BRANCH='${BRANCH}' DEPLOY_REPO_URL='${DEPLOY_REPO_URL}' DEPLOY_MONOREPO_DIR='${DEPLOY_MONOREPO_DIR}' DEPLOY_DIR='${DEPLOY_DIR}' DEPLOY_RUNTIME_ENV_FILE='${DEPLOY_RUNTIME_ENV_FILE}' DEPLOY_LOGS_DIR='${DEPLOY_LOGS_DIR}' IMAGE_NAME='${IMAGE_NAME}' IMAGE_REF='${IMAGE_REF}' LATEST_REF='${LATEST_REF}' KEEP_IMAGES='${KEEP_IMAGES}' CONTAINER_NAME='${CONTAINER_NAME}' PORT='${PORT}' bash -s" << 'EOF'
set -euo pipefail

git_with_proxy() {
    if command -v zsh >/dev/null 2>&1; then
        zsh -ic 'proxyOn >/dev/null 2>&1 || true; cd -- "$1"; shift; git -c http.version=HTTP/1.1 "$@"' \
            git-with-proxy "${DEPLOY_MONOREPO_DIR}" "$@"
    else
        git -c http.version=HTTP/1.1 "$@"
    fi
}

ensure_origin_remote() {
    local current_url

    current_url="$(git remote get-url origin 2>/dev/null || true)"
    if [ "${current_url}" = "${DEPLOY_REPO_URL}" ]; then
        return 0
    fi

    echo "🔧 修正生产机 Git origin: ${current_url:-<missing>} -> ${DEPLOY_REPO_URL}"
    if git remote get-url origin >/dev/null 2>&1; then
        git remote set-url origin "${DEPLOY_REPO_URL}"
    else
        git remote add origin "${DEPLOY_REPO_URL}"
    fi
}

git_sync_branch() {
    local branch="$1"

    ensure_origin_remote

    if git_with_proxy fetch origin "${branch}"; then
        git checkout "${branch}"
        git_with_proxy pull --ff-only origin "${branch}"
        return 0
    fi

    echo "⚠️ git 同步首次失败，5 秒后重试一次..."
    sleep 5
    ensure_origin_remote
    git_with_proxy fetch origin "${branch}"
    git checkout "${branch}"
    git_with_proxy pull --ff-only origin "${branch}"
}

if [ ! -d "${DEPLOY_MONOREPO_DIR}/.git" ]; then
    echo "📥 首次初始化 ${DEPLOY_MONOREPO_DIR}..."
    mkdir -p "${DEPLOY_MONOREPO_DIR}"
    TEMP_CLONE_DIR="$(mktemp -d /tmp/football-insight-monorepo-XXXXXX)"
    git_with_proxy clone --branch "${BRANCH}" "${DEPLOY_REPO_URL}" "${TEMP_CLONE_DIR}"
    shopt -s dotglob nullglob
    mv "${TEMP_CLONE_DIR}"/* "${DEPLOY_MONOREPO_DIR}"/
    rmdir "${TEMP_CLONE_DIR}"
fi

cd "${DEPLOY_MONOREPO_DIR}"

if ! git diff --quiet || ! git diff --cached --quiet; then
    echo "⚠️ 生产机工作区有已跟踪文件改动，先 stash 保存"
    git stash push -m "deploy-docker-auto-stash-$(date +%Y%m%d%H%M%S)"
fi

git_sync_branch "${BRANCH}"

if [ ! -f "${DEPLOY_RUNTIME_ENV_FILE}" ]; then
    echo "❌ 未找到运行时环境文件: ${DEPLOY_RUNTIME_ENV_FILE}"
    exit 1
fi

cd "${DEPLOY_DIR}"

echo "📦 构建本地镜像（rsproxy 加速拉取依赖）..."
docker build --pull -t "${IMAGE_REF}" -t "${LATEST_REF}" .

echo "🧹 清理历史镜像，只保留最近 ${KEEP_IMAGES} 个 tag..."
docker images --format '{{.CreatedAt}}\t{{.Tag}}' "${IMAGE_NAME}" \
    | grep -v $'\tlatest$' \
    | sort -r \
    | tail -n +$((KEEP_IMAGES + 1)) \
    | cut -f2 \
    | xargs -r docker rmi || true
docker image prune -f >/dev/null

mkdir -p "${DEPLOY_LOGS_DIR}"

if systemctl list-unit-files football-insight.service >/dev/null 2>&1; then
    systemctl stop football-insight.service || true
    systemctl disable football-insight.service || true
fi

if command -v lsof >/dev/null 2>&1; then
    lsof -ti:"${PORT}" | xargs -r kill -9 || true
fi

docker rm -f "${CONTAINER_NAME}" >/dev/null 2>&1 || true
chown -R 10001:10001 "${DEPLOY_LOGS_DIR}" || true

docker run -d \
    --name "${CONTAINER_NAME}" \
    --restart unless-stopped \
    --network host \
    --env-file "${DEPLOY_RUNTIME_ENV_FILE}" \
    -e PORT="${PORT}" \
    -v "${DEPLOY_LOGS_DIR}:/app/logs" \
    "${IMAGE_REF}"

sleep 8
docker ps --filter "name=${CONTAINER_NAME}"
curl -fsS "http://127.0.0.1:${PORT}/api/health" >/dev/null
echo "✅ Docker 容器部署成功（本地镜像 ${IMAGE_REF}）"
EOF

echo "🎉 Docker 部署完成: ${IMAGE_REF}"

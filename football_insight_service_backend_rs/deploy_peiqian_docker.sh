#!/bin/bash

# Build the football insight backend image on peiqian itself, push it to
# Harbor, then deploy the image on the peiqian host. The Dockerfile already
# pulls Rust dependencies through rsproxy (and base images / apt through
# China mirrors), so no extra build-host setup is required.
#
# Required flow:
#   1. Commit and push locally
#   2. peiqian pulls the pushed commit into /root/projects/football_insight
#   3. peiqian builds the Docker image (rsproxy-accelerated) and pushes to Harbor
#   4. peiqian pulls the image and restarts the container
#
# Required secret:
#   Put HARBOR_PASSWORD in .env, or export it before running this script.
#
# Optional:
#   BUILD_HOST / BUILD_REPO_DIR override the build machine (e.g. out109).
#   BUILD_USE_PROXY=1 runs the build inside `zsh -ic 'proxy_on'` (out109 only).

set -euo pipefail

DEPLOY_ENV_FILE="${DEPLOY_ENV_FILE:-.env}"

load_env_file() {
    if [ -f "${DEPLOY_ENV_FILE}" ]; then
        set -a
        # shellcheck disable=SC1090
        . "${DEPLOY_ENV_FILE}"
        set +a
    fi
}

load_env_file

git_sync_branch() {
    local branch="$1"

    if git -c http.version=HTTP/1.1 fetch origin "${branch}"; then
        git checkout "${branch}"
        git -c http.version=HTTP/1.1 pull --ff-only origin "${branch}"
        return 0
    fi

    echo "⚠️ git 同步首次失败，5 秒后重试一次..."
    sleep 5
    git -c http.version=HTTP/1.1 fetch origin "${branch}"
    git checkout "${branch}"
    git -c http.version=HTTP/1.1 pull --ff-only origin "${branch}"
}

BRANCH="${DEPLOY_BRANCH:-main}"
BUILD_HOST="${BUILD_HOST:-peiqian}"
BUILD_REPO_URL="${BUILD_REPO_URL:-https://github.com/oryjk/football_insight.git}"
BUILD_REPO_DIR="${BUILD_REPO_DIR:-/root/projects/football_insight}"
BUILD_DIR="${BUILD_DIR:-${BUILD_REPO_DIR}/football_insight_service_backend_rs}"
DEPLOY_HOST="${DEPLOY_HOST:-peiqian}"
DEPLOY_REPO_URL="${DEPLOY_REPO_URL:-${BUILD_REPO_URL}}"
DEPLOY_MONOREPO_DIR="${DEPLOY_MONOREPO_DIR:-/root/projects/football_insight}"
DEPLOY_DIR="${DEPLOY_DIR:-${DEPLOY_MONOREPO_DIR}/football_insight_service_backend_rs}"
DEPLOY_RUNTIME_ENV_FILE="${DEPLOY_RUNTIME_ENV_FILE:-${DEPLOY_MONOREPO_DIR}/football-insight-service-backend-rs.env}"
DEPLOY_LOGS_DIR="${DEPLOY_LOGS_DIR:-${DEPLOY_DIR}/logs}"

HARBOR_REGISTRY="${HARBOR_REGISTRY:-harbor.oryjk.cn:82}"
HARBOR_PROJECT="${HARBOR_PROJECT:-library}"
HARBOR_USERNAME="${HARBOR_USERNAME:-admin}"
HARBOR_PASSWORD="${HARBOR_PASSWORD:-}"

IMAGE_NAME="${IMAGE_NAME:-football-insight-service-backend-rs}"
IMAGE_TAG="${IMAGE_TAG:-$(git rev-parse --short HEAD)}"
IMAGE_REF="${HARBOR_REGISTRY}/${HARBOR_PROJECT}/${IMAGE_NAME}:${IMAGE_TAG}"
LATEST_REF="${HARBOR_REGISTRY}/${HARBOR_PROJECT}/${IMAGE_NAME}:latest"
CONTAINER_NAME="${CONTAINER_NAME:-football-insight-service-backend-rs}"
PORT="${PORT:-8092}"
DEPLOY_RUN_ID="${DEPLOY_RUN_ID:-$(date +%Y%m%d%H%M%S)-$$}"
BUILD_DOCKER_CONFIG="${BUILD_DOCKER_CONFIG:-/tmp/football-insight-docker-auth-${DEPLOY_RUN_ID}-build}"
DEPLOY_DOCKER_CONFIG="${DEPLOY_DOCKER_CONFIG:-/tmp/football-insight-docker-auth-${DEPLOY_RUN_ID}-deploy}"
BUILD_ENV_FILE="${BUILD_ENV_FILE:-${BUILD_DIR}/.env}"

cleanup_docker_auth() {
    case "${BUILD_DOCKER_CONFIG}" in
        /tmp/football-insight-docker-auth*)
            ssh "${BUILD_HOST}" "rm -rf '${BUILD_DOCKER_CONFIG}'" >/dev/null 2>&1 || true
            ;;
    esac

    case "${DEPLOY_DOCKER_CONFIG}" in
        /tmp/football-insight-docker-auth*)
            ssh "${DEPLOY_HOST}" "rm -rf '${DEPLOY_DOCKER_CONFIG}'" >/dev/null 2>&1 || true
            ;;
    esac
}

trap cleanup_docker_auth EXIT

if [ -z "${HARBOR_PASSWORD}" ]; then
    if [ -t 0 ]; then
        read -r -s -p "请输入 Harbor 密码: " HARBOR_PASSWORD
        echo
    else
        echo "❌ 请通过 HARBOR_PASSWORD 环境变量传入 Harbor 密码"
        exit 1
    fi
fi

echo "🚀 Docker 镜像部署到 ${DEPLOY_HOST}"
echo "image: ${IMAGE_REF}"

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

echo "🔧 准备 ${BUILD_HOST} 构建仓库..."
ssh "${BUILD_HOST}" \
    "BUILD_REPO_URL='${BUILD_REPO_URL}' BUILD_REPO_DIR='${BUILD_REPO_DIR}' BRANCH='${BRANCH}' bash -s" << 'EOF'
set -euo pipefail

git_with_proxy() {
    if command -v zsh >/dev/null 2>&1; then
        zsh -ic 'proxyOn >/dev/null 2>&1 || true; cd -- "$1"; shift; git -c http.version=HTTP/1.1 "$@"' \
            git-with-proxy "${BUILD_REPO_DIR}" "$@"
    else
        git -c http.version=HTTP/1.1 "$@"
    fi
}

git_sync_branch() {
    local branch="$1"

    if git_with_proxy fetch origin "${branch}"; then
        git checkout "${branch}"
        git_with_proxy pull --ff-only origin "${branch}"
        return 0
    fi

    echo "⚠️ git 同步首次失败，5 秒后重试一次..."
    sleep 5
    git_with_proxy fetch origin "${branch}"
    git checkout "${branch}"
    git_with_proxy pull --ff-only origin "${branch}"
}

if [ ! -d "${BUILD_REPO_DIR}/.git" ]; then
    rm -rf "${BUILD_REPO_DIR}"
    mkdir -p "${BUILD_REPO_DIR}"
    TEMP_CLONE_DIR="$(mktemp -d /tmp/football-insight-build-repo-XXXXXX)"
    git_with_proxy clone --branch "${BRANCH}" "${BUILD_REPO_URL}" "${TEMP_CLONE_DIR}"
    shopt -s dotglob nullglob
    mv "${TEMP_CLONE_DIR}"/* "${BUILD_REPO_DIR}"/
    rmdir "${TEMP_CLONE_DIR}"
fi

cd "${BUILD_REPO_DIR}"
git_sync_branch "${BRANCH}"
EOF

if [ -f "${DEPLOY_ENV_FILE}" ]; then
    echo "📄 同步 ${DEPLOY_ENV_FILE} 到 ${BUILD_HOST}:${BUILD_ENV_FILE}..."
    scp "${DEPLOY_ENV_FILE}" "${BUILD_HOST}:${BUILD_ENV_FILE}" >/dev/null
else
    echo "⚠️ 未找到 ${DEPLOY_ENV_FILE}，跳过同步构建环境文件"
fi

echo "🔐 登录 Harbor on ${BUILD_HOST}..."
printf '%s' "${HARBOR_PASSWORD}" \
    | ssh "${BUILD_HOST}" "mkdir -p '${BUILD_DOCKER_CONFIG}' && DOCKER_CONFIG='${BUILD_DOCKER_CONFIG}' docker login ${HARBOR_REGISTRY} -u '${HARBOR_USERNAME}' --password-stdin"

echo "📦 在 ${BUILD_HOST} 拉取代码、构建镜像并推送..."
if [[ "${BUILD_USE_PROXY:-0}" == "1" ]]; then
    BUILD_RUN_SHELL="zsh -ic 'proxy_on; bash -s'"
else
    BUILD_RUN_SHELL="bash -s"
fi
ssh "${BUILD_HOST}" \
    "BUILD_REPO_DIR='${BUILD_REPO_DIR}' BUILD_DIR='${BUILD_DIR}' BRANCH='${BRANCH}' IMAGE_REF='${IMAGE_REF}' LATEST_REF='${LATEST_REF}' DOCKER_CONFIG='${BUILD_DOCKER_CONFIG}' BUILD_ENV_FILE='${BUILD_ENV_FILE}' ${BUILD_RUN_SHELL}" << 'EOF'
set -euo pipefail
export DOCKER_CONFIG

git_with_proxy() {
    if command -v zsh >/dev/null 2>&1; then
        zsh -ic 'proxyOn >/dev/null 2>&1 || true; cd -- "$1"; shift; git -c http.version=HTTP/1.1 "$@"' \
            git-with-proxy "${BUILD_REPO_DIR}" "$@"
    else
        git -c http.version=HTTP/1.1 "$@"
    fi
}

git_sync_branch() {
    local branch="$1"

    if git_with_proxy fetch origin "${branch}"; then
        git checkout "${branch}"
        git_with_proxy pull --ff-only origin "${branch}"
        return 0
    fi

    echo "⚠️ git 同步首次失败，5 秒后重试一次..."
    sleep 5
    git_with_proxy fetch origin "${branch}"
    git checkout "${branch}"
    git_with_proxy pull --ff-only origin "${branch}"
}

cd "${BUILD_REPO_DIR}"

if [ -f "${BUILD_ENV_FILE}" ]; then
    set -a
    # shellcheck disable=SC1090
    . "${BUILD_ENV_FILE}"
    set +a
fi

if ! git diff --quiet || ! git diff --cached --quiet; then
    echo "⚠️ 构建机工作区有已跟踪文件改动，先 stash 保存"
    git stash push -m "deploy-docker-auto-stash-$(date +%Y%m%d%H%M%S)"
fi

git_sync_branch "${BRANCH}"

cd "${BUILD_DIR}"

docker build --pull -t "${IMAGE_REF}" -t "${LATEST_REF}" .
docker push "${IMAGE_REF}"
docker push "${LATEST_REF}"
EOF

echo "🔐 登录 Harbor on ${DEPLOY_HOST}..."
printf '%s' "${HARBOR_PASSWORD}" \
    | ssh "${DEPLOY_HOST}" "mkdir -p '${DEPLOY_DOCKER_CONFIG}' && DOCKER_CONFIG='${DEPLOY_DOCKER_CONFIG}' docker login ${HARBOR_REGISTRY} -u '${HARBOR_USERNAME}' --password-stdin"

echo "🚀 在 ${DEPLOY_HOST} 拉取镜像并重启容器..."
ssh "${DEPLOY_HOST}" \
    "BRANCH='${BRANCH}' DEPLOY_REPO_URL='${DEPLOY_REPO_URL}' DEPLOY_MONOREPO_DIR='${DEPLOY_MONOREPO_DIR}' DEPLOY_DIR='${DEPLOY_DIR}' DEPLOY_RUNTIME_ENV_FILE='${DEPLOY_RUNTIME_ENV_FILE}' DEPLOY_LOGS_DIR='${DEPLOY_LOGS_DIR}' IMAGE_REF='${IMAGE_REF}' CONTAINER_NAME='${CONTAINER_NAME}' PORT='${PORT}' DOCKER_CONFIG='${DEPLOY_DOCKER_CONFIG}' bash -s" << 'EOF'
set -euo pipefail
export DOCKER_CONFIG

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

if [ ! -d "${DEPLOY_MONOREPO_DIR}/.git" ]; then
    echo "📥 首次初始化 ${DEPLOY_MONOREPO_DIR}..."
    mkdir -p "${DEPLOY_MONOREPO_DIR}"
    TEMP_CLONE_DIR="$(mktemp -d /tmp/football-insight-monorepo-XXXXXX)"
    git_with_proxy clone --branch "${BRANCH}" "${DEPLOY_REPO_URL}" "${TEMP_CLONE_DIR}"
    shopt -s dotglob nullglob
    mv "${TEMP_CLONE_DIR}"/* "${DEPLOY_MONOREPO_DIR}/"
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

mkdir -p "${DEPLOY_DIR}" "${DEPLOY_LOGS_DIR}"

docker pull "${IMAGE_REF}"

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
echo "✅ Docker 容器部署成功"
EOF

echo "🎉 Docker 部署完成: ${IMAGE_REF}"

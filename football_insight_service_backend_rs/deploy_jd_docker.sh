#!/bin/bash

# Build the football insight backend image on out109, push it to Harbor, then
# deploy the image on the JD host.
#
# Required flow:
#   1. Commit and push locally
#   2. out109 pulls the pushed commit
#   3. out109 builds and pushes the Docker image to Harbor
#   4. JD pulls the image and restarts the container
#
# Required secret:
#   Put HARBOR_PASSWORD in .env, or export it before running this script.

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

BRANCH="${DEPLOY_BRANCH:-main}"
BUILD_HOST="${BUILD_HOST:-out109}"
BUILD_DIR="${BUILD_DIR:-/home/wangrui/projects/football_insight/football_insight_service_backend_rs}"
DEPLOY_HOST="${DEPLOY_HOST:-jd}"
DEPLOY_DIR="${DEPLOY_DIR:-/root/projects/football_insight/football_insight_service_backend_rs}"

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

echo "🚀 Docker 镜像部署到 JD"
echo "image: ${IMAGE_REF}"

echo "🔎 检查本地提交是否已 push..."
git fetch origin "${BRANCH}"
LOCAL_HEAD="$(git rev-parse HEAD)"
REMOTE_HEAD="$(git rev-parse "origin/${BRANCH}")"

if [ "${LOCAL_HEAD}" != "${REMOTE_HEAD}" ]; then
    echo "❌ 当前 HEAD 尚未推送到 origin/${BRANCH}"
    echo "local : ${LOCAL_HEAD}"
    echo "remote: ${REMOTE_HEAD}"
    exit 1
fi

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
ssh "${BUILD_HOST}" \
    "BUILD_DIR='${BUILD_DIR}' BRANCH='${BRANCH}' IMAGE_REF='${IMAGE_REF}' LATEST_REF='${LATEST_REF}' DOCKER_CONFIG='${BUILD_DOCKER_CONFIG}' BUILD_ENV_FILE='${BUILD_ENV_FILE}' zsh -ic 'bash -s'" << 'EOF'
set -euo pipefail
export DOCKER_CONFIG

if command -v proxy_on >/dev/null 2>&1; then
    proxy_on
fi

cd "${BUILD_DIR}"

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

git fetch origin "${BRANCH}"
git checkout "${BRANCH}"
git pull --ff-only origin "${BRANCH}"

docker build --pull -t "${IMAGE_REF}" -t "${LATEST_REF}" .
docker push "${IMAGE_REF}"
docker push "${LATEST_REF}"
EOF

echo "🔐 登录 Harbor on ${DEPLOY_HOST}..."
printf '%s' "${HARBOR_PASSWORD}" \
    | ssh "${DEPLOY_HOST}" "mkdir -p '${DEPLOY_DOCKER_CONFIG}' && DOCKER_CONFIG='${DEPLOY_DOCKER_CONFIG}' docker login ${HARBOR_REGISTRY} -u '${HARBOR_USERNAME}' --password-stdin"

echo "🚀 在 ${DEPLOY_HOST} 拉取镜像并重启容器..."
ssh "${DEPLOY_HOST}" \
    "DEPLOY_DIR='${DEPLOY_DIR}' IMAGE_REF='${IMAGE_REF}' CONTAINER_NAME='${CONTAINER_NAME}' PORT='${PORT}' DOCKER_CONFIG='${DEPLOY_DOCKER_CONFIG}' bash -s" << 'EOF'
set -euo pipefail
export DOCKER_CONFIG

cd "${DEPLOY_DIR}"

docker pull "${IMAGE_REF}"

if systemctl list-unit-files football-insight.service >/dev/null 2>&1; then
    systemctl stop football-insight.service || true
    systemctl disable football-insight.service || true
fi

if command -v lsof >/dev/null 2>&1; then
    lsof -ti:"${PORT}" | xargs -r kill -9 || true
fi

docker rm -f "${CONTAINER_NAME}" >/dev/null 2>&1 || true
mkdir -p logs
chown -R 10001:10001 logs

docker run -d \
    --name "${CONTAINER_NAME}" \
    --restart unless-stopped \
    --network host \
    --env-file "${DEPLOY_DIR}/.env" \
    -e PORT="${PORT}" \
    -v "${DEPLOY_DIR}/logs:/app/logs" \
    "${IMAGE_REF}"

sleep 8
docker ps --filter "name=${CONTAINER_NAME}"
curl -fsS "http://127.0.0.1:${PORT}/api/health" >/dev/null
echo "✅ Docker 容器部署成功"
EOF

echo "🎉 Docker 部署完成: ${IMAGE_REF}"

#!/bin/bash

# Replace the match.oryjk.cn TLS certificate on jd (Docker nginx) and
# gracefully reload nginx.
#
# Usage:
#   bash deploy/renew-jd-nginx-ssl.sh [证书目录]
#
# 证书目录需包含 match.oryjk.cn.pem（含中间证书的 fullchain）和
# match.oryjk.cn.key。不传参数时自动选取 ~/Downloads 下最新的
# *_match.oryjk.cn_nginx 目录。
#
# 流程：本地校验（文件存在 / 域名匹配 / 有效期 / 证书私钥配对）
# -> 远端备份旧证书 -> 覆盖上传 -> nginx -t（失败自动回滚）
# -> nginx -s reload -> 回读线上证书确认生效。

set -euo pipefail

HOST="${DEPLOY_HOST:-jd}"
CONTAINER="${NGINX_CONTAINER:-nginx-server}"
SSL_DIR="${SSL_DIR:-/root/docker_data/nginx/ssl}"
CERT_NAME="${CERT_NAME:-match.oryjk.cn}"
DOWNLOADS_DIR="${DOWNLOADS_DIR:-$HOME/Downloads}"

PEM_FILE="${CERT_NAME}.pem"
KEY_FILE="${CERT_NAME}.key"

# "Aug 29 00:00:00 2026 GMT" -> epoch seconds (GNU date first, BSD/macOS fallback)
cert_date_to_epoch() {
    if date -u -d "$1" +%s 2>/dev/null; then
        return 0
    fi
    date -u -j -f "%b %e %H:%M:%S %Y %Z" "$1" +%s
}

echo "🔐 更换 ${HOST} 上 nginx 容器 ${CONTAINER} 的 ${CERT_NAME} 证书"

# ---------- 1. 定位证书目录 ----------

if [ $# -ge 1 ]; then
    CERT_DIR="$1"
else
    CERT_DIR="$(ls -dt "${DOWNLOADS_DIR}"/*_"${CERT_NAME}"_nginx 2>/dev/null | head -1 || true)"
    if [ -z "${CERT_DIR}" ]; then
        echo "❌ 未指定证书目录，且 ${DOWNLOADS_DIR} 下找不到 *_${CERT_NAME}_nginx 目录"
        echo "用法: $0 <包含 ${PEM_FILE} 和 ${KEY_FILE} 的目录>"
        exit 1
    fi
    echo "🧭 未指定目录，自动选择最新下载: ${CERT_DIR}"
fi

PEM_PATH="${CERT_DIR}/${PEM_FILE}"
KEY_PATH="${CERT_DIR}/${KEY_FILE}"

if [ ! -f "${PEM_PATH}" ] || [ ! -f "${KEY_PATH}" ]; then
    echo "❌ ${CERT_DIR} 下缺少 ${PEM_FILE} 或 ${KEY_FILE}"
    exit 1
fi

# ---------- 2. 本地校验 ----------

echo "🔎 校验新证书..."
openssl x509 -in "${PEM_PATH}" -noout -subject -issuer -dates

CERT_TEXT="$(openssl x509 -in "${PEM_PATH}" -noout -text)"
if ! grep -q "${CERT_NAME}" <<<"$(openssl x509 -in "${PEM_PATH}" -noout -subject)" \
    && ! grep -q "${CERT_NAME}" <<<"$(grep -A1 'Subject Alternative Name' <<<"${CERT_TEXT}")"; then
    echo "❌ 证书域名不包含 ${CERT_NAME}，拒绝上传"
    exit 1
fi

if ! openssl x509 -in "${PEM_PATH}" -noout -checkend 0 >/dev/null; then
    echo "❌ 证书已过期，拒绝上传"
    exit 1
fi

NOT_BEFORE="$(openssl x509 -in "${PEM_PATH}" -noout -startdate | cut -d= -f2)"
NOW_EPOCH="$(date -u +%s)"
if [ "$(cert_date_to_epoch "${NOT_BEFORE}")" -gt "${NOW_EPOCH}" ]; then
    echo "❌ 证书尚未生效（notBefore: ${NOT_BEFORE}），拒绝上传"
    exit 1
fi

CERT_MOD="$(openssl x509 -in "${PEM_PATH}" -noout -modulus | openssl md5)"
KEY_MOD="$(openssl rsa -in "${KEY_PATH}" -noout -modulus 2>/dev/null | openssl md5)"
if [ "${CERT_MOD}" != "${KEY_MOD}" ]; then
    echo "❌ 证书与私钥不配对（modulus 不一致），拒绝上传"
    exit 1
fi

CHAIN_COUNT="$(grep -c 'BEGIN CERTIFICATE' "${PEM_PATH}")"
if [ "${CHAIN_COUNT}" -lt 2 ]; then
    echo "⚠️  pem 里只有 ${CHAIN_COUNT} 张证书，可能缺少中间证书（fullchain 应 ≥2）"
fi

EXPECTED_END="$(openssl x509 -in "${PEM_PATH}" -noout -enddate | cut -d= -f2)"
echo "✅ 本地校验通过（证书链 ${CHAIN_COUNT} 张，到期 ${EXPECTED_END}）"

# ---------- 3. 备份远端旧证书 ----------

BACKUP_DIR="${SSL_DIR}.bak.$(date +%Y%m%d%H%M%S)"
echo "📦 备份远端旧证书到 ${BACKUP_DIR}"
ssh "${HOST}" "cp -a '${SSL_DIR}' '${BACKUP_DIR}'"

# ---------- 4. 上传并设置权限 ----------

echo "⬆️  上传新证书到 ${HOST}:${SSL_DIR}/"
scp -q "${PEM_PATH}" "${KEY_PATH}" "${HOST}:${SSL_DIR}/"
ssh "${HOST}" "chmod 644 '${SSL_DIR}/${PEM_FILE}' && chmod 600 '${SSL_DIR}/${KEY_FILE}'"

# ---------- 5. nginx -t 校验（失败自动回滚） ----------

if ! ssh "${HOST}" "docker exec ${CONTAINER} nginx -t"; then
    echo "❌ nginx -t 失败，回滚旧证书"
    ssh "${HOST}" "cp -a '${BACKUP_DIR}/${PEM_FILE}' '${BACKUP_DIR}/${KEY_FILE}' '${SSL_DIR}/'"
    exit 1
fi

# ---------- 6. 平滑重载并验证 ----------

echo "🔄 reload nginx..."
ssh "${HOST}" "docker exec ${CONTAINER} nginx -s reload"

sleep 2
echo "🔍 回读线上证书验证..."
SERVED_END="$(ssh "${HOST}" "echo | openssl s_client -connect 127.0.0.1:443 -servername ${CERT_NAME} 2>/dev/null | openssl x509 -noout -enddate" | cut -d= -f2)"
if [ "${SERVED_END}" != "${EXPECTED_END}" ]; then
    echo "❌ 线上证书到期时间（${SERVED_END}）与预期（${EXPECTED_END}）不一致，请人工检查"
    exit 1
fi

echo "✅ ${CERT_NAME} 证书已生效，到期时间 ${SERVED_END}"
echo "ℹ️  旧证书备份在 ${HOST}:${BACKUP_DIR}，确认无误后可删除"

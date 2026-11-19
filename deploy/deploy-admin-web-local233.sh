#!/usr/bin/env bash
set -euo pipefail

repo_root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
public_root=${GATEWAY_PUBLIC_ROOT:-"$HOME/.local/share/betalpha-admin-downloads"}
release_stamp=$(TZ=Asia/Shanghai date +%Y%m%dT%H%M%S)
release_name="${release_stamp}-$$"
release_root="$public_root/football-admin-releases"
release_dir="$release_root/$release_name"
temporary_link="$public_root/.football-admin-link-$$"

if [[ -e "$public_root/football-admin" && ! -L "$public_root/football-admin" ]]; then
    echo "football-admin exists and is not a managed release link; inspect it before publishing." >&2
    exit 1
fi

cd "$repo_root/football_insight_admin_web"
bun install --frozen-lockfile
bun run test
ADMIN_WEB_BASE_PATH=/football-admin/ bun run build

mkdir -p "$release_dir"
cp -a dist/. "$release_dir/"

if [[ "${SKIP_H5_PREVIEW_BUILD:-0}" != "1" ]]; then
    preview_api_base=${PREVIEW_API_BASE_URL:-http://172.16.60.233:18092/api/v1}
    cd "$repo_root/football_insight_mini"
    bun install --frozen-lockfile
    MINI_H5_BASE_PATH=/football-admin/preview/ \
    VITE_API_BASE_URL="$preview_api_base" \
        bun run build:h5
    mkdir -p "$release_dir/preview"
    cp -a "$repo_root/football_insight_mini/dist/build/h5/." "$release_dir/preview/"
fi

ln -s "football-admin-releases/$release_name" "$temporary_link"
mv -Tf "$temporary_link" "$public_root/football-admin"

echo "Published static release: $release_dir"
echo "Gateway URL: http://172.16.60.233/football-admin/"
echo "Co-hosted H5 Preview URL: http://172.16.60.233/football-admin/preview/"
echo "Requires gateway routes from deploy/nginx/local233-admin-web.conf and backend18092."

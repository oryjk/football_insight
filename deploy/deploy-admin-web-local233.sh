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
ln -s "football-admin-releases/$release_name" "$temporary_link"
mv -Tf "$temporary_link" "$public_root/football-admin"

echo "Published static release: $release_dir"
echo "Gateway URL: http://172.16.60.233/football-admin/"
echo "Requires gateway routes from deploy/nginx/local233-admin-web.conf and backend18092."

#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
ANDROID_DIR=$(cd -- "$SCRIPT_DIR/.." && pwd)
REPO_ROOT=$(git -C "$ANDROID_DIR" rev-parse --show-toplevel)
PUBLISH_HOST=${FOOTBALL_ADMIN_PUBLISH_HOST:-jd}
PUBLIC_DIR=${FOOTBALL_ADMIN_PUBLIC_DIR:-/root/docker_data/nginx/html/football/admin-android}
PUBLIC_URL=${FOOTBALL_ADMIN_PUBLIC_URL:-https://match.oryjk.cn/football/admin-android/}
RETAIN_RELEASES=${RETAIN_RELEASES:-10}
SKIP_BUILD=false
NOTES=()

usage() {
    cat <<'EOF'
Usage: publish-apk.sh [--skip-build] --note TEXT [--note TEXT ...]

Environment:
  FOOTBALL_ADMIN_PUBLISH_HOST  SSH host serving the download page. Defaults to jd.
  FOOTBALL_ADMIN_PUBLIC_DIR    Remote static directory.
  FOOTBALL_ADMIN_PUBLIC_URL    Public HTTPS download URL.
  ANDROID_HOME                 Android SDK path.
  RETAIN_RELEASES              Versioned APK files to retain. Defaults to 10.
EOF
}

while (($#)); do
    case "$1" in
        --skip-build) SKIP_BUILD=true; shift ;;
        --note) NOTES+=("${2:?Missing release note}"); shift 2 ;;
        -h|--help) usage; exit 0 ;;
        *) echo "Unknown argument: $1" >&2; usage >&2; exit 2 ;;
    esac
done

if ((${#NOTES[@]} == 0)); then
    echo "At least one release note is required." >&2
    exit 2
fi
if [[ -n "$(git -C "$REPO_ROOT" status --porcelain --untracked-files=normal)" ]]; then
    echo "Release blocked: commit all changes before publishing." >&2
    exit 1
fi

BRANCH=$(git -C "$REPO_ROOT" symbolic-ref --quiet --short HEAD || true)
UPSTREAM=$(git -C "$REPO_ROOT" rev-parse --abbrev-ref --symbolic-full-name '@{upstream}' 2>/dev/null || true)
if [[ -z "$BRANCH" || -z "$UPSTREAM" ]]; then
    echo "Release blocked: branch and upstream are required." >&2
    exit 1
fi
REMOTE=${UPSTREAM%%/*}
REMOTE_BRANCH=${UPSTREAM#*/}
git -C "$REPO_ROOT" fetch --quiet "$REMOTE" "$REMOTE_BRANCH"
GIT_COMMIT=$(git -C "$REPO_ROOT" rev-parse HEAD)
REMOTE_COMMIT=$(git -C "$REPO_ROOT" rev-parse "$REMOTE/$REMOTE_BRANCH")
if [[ "$GIT_COMMIT" != "$REMOTE_COMMIT" ]]; then
    echo "Release blocked: local HEAD is not synchronized with $UPSTREAM." >&2
    exit 1
fi

if [[ ! "$RETAIN_RELEASES" =~ ^[0-9]+$ ]] || ((RETAIN_RELEASES < 1)); then
    echo "RETAIN_RELEASES must be a positive integer." >&2
    exit 2
fi

if [[ "$SKIP_BUILD" == false ]]; then
    (cd "$ANDROID_DIR" && ./gradlew testDebugUnitTest lintDebug assembleDebug)
fi

APK_PATH="$ANDROID_DIR/app/build/outputs/apk/debug/app-debug.apk"
[[ -f "$APK_PATH" ]] || { echo "APK not found: $APK_PATH" >&2; exit 1; }

SDK_ROOT=${ANDROID_HOME:-$HOME/Android/Sdk}
APK_ANALYZER=${APK_ANALYZER:-$SDK_ROOT/cmdline-tools/latest/bin/apkanalyzer}
APKSIGNER=${APKSIGNER:-$(find "$SDK_ROOT/build-tools" -mindepth 2 -maxdepth 2 -type f -name apksigner -perm -u+x | sort -V | tail -n 1)}
[[ -x "$APK_ANALYZER" ]] || { echo "apkanalyzer not found: $APK_ANALYZER" >&2; exit 1; }
[[ -x "$APKSIGNER" ]] || { echo "apksigner not found" >&2; exit 1; }
"$APKSIGNER" verify "$APK_PATH"

VERSION_NAME=$($APK_ANALYZER manifest version-name "$APK_PATH")
VERSION_CODE=$($APK_ANALYZER manifest version-code "$APK_PATH")
SAFE_VERSION=${VERSION_NAME//[^a-zA-Z0-9._-]/-}
PUBLISHED_AT=$(TZ=Asia/Shanghai date --iso-8601=seconds)
TIMESTAMP=$(TZ=Asia/Shanghai date +%Y%m%d-%H%M%S)
RELEASE_FILE="football-insight-admin-${SAFE_VERSION}-${TIMESTAMP}.apk"
FILE_SIZE=$(stat -c '%s' "$APK_PATH")
SHA256=$(sha256sum "$APK_PATH" | awk '{print $1}')

STAGING_DIR=$(mktemp -d)
trap 'rm -rf -- "$STAGING_DIR"' EXIT
NOTES_JSON=$(jq -n --args '$ARGS.positional' "${NOTES[@]}")
jq -n \
    --arg versionName "$VERSION_NAME" \
    --arg versionCode "$VERSION_CODE" \
    --arg publishedAt "$PUBLISHED_AT" \
    --arg sha256 "$SHA256" \
    --arg gitCommit "$GIT_COMMIT" \
    --arg releaseFile "releases/$RELEASE_FILE" \
    --argjson releaseNotes "$NOTES_JSON" \
    --argjson fileSizeBytes "$FILE_SIZE" \
    '{versionName:$versionName,versionCode:$versionCode,buildType:"debug",publishedAt:$publishedAt,fileSizeBytes:$fileSizeBytes,sha256:$sha256,gitCommit:$gitCommit,releaseFile:$releaseFile,releaseNotes:$releaseNotes}' \
    > "$STAGING_DIR/metadata.json"

ssh "$PUBLISH_HOST" "mkdir -p '$PUBLIC_DIR/releases'"
if ssh "$PUBLISH_HOST" "test -f '$PUBLIC_DIR/releases.json'"; then
    ssh "$PUBLISH_HOST" "cat '$PUBLIC_DIR/releases.json'" > "$STAGING_DIR/existing.json"
else
    echo '[]' > "$STAGING_DIR/existing.json"
fi
jq -n \
    --slurpfile current "$STAGING_DIR/metadata.json" \
    --slurpfile existing "$STAGING_DIR/existing.json" \
    --argjson keep "$RETAIN_RELEASES" \
    '[$current[0]] + [$existing[0][] | select(.sha256 != $current[0].sha256)] | .[:$keep]' \
    > "$STAGING_DIR/releases.json"

rsync -a "$APK_PATH" "$PUBLISH_HOST:$PUBLIC_DIR/releases/$RELEASE_FILE"
rsync -a "$APK_PATH" "$PUBLISH_HOST:$PUBLIC_DIR/.latest.apk.tmp"
rsync -a "$ANDROID_DIR/distribution/index.html" "$PUBLISH_HOST:$PUBLIC_DIR/.index.html.tmp"
rsync -a "$STAGING_DIR/metadata.json" "$PUBLISH_HOST:$PUBLIC_DIR/.metadata.json.tmp"
rsync -a "$STAGING_DIR/releases.json" "$PUBLISH_HOST:$PUBLIC_DIR/.releases.json.tmp"
ssh "$PUBLISH_HOST" "set -eu; mv '$PUBLIC_DIR/.latest.apk.tmp' '$PUBLIC_DIR/latest.apk'; mv '$PUBLIC_DIR/.index.html.tmp' '$PUBLIC_DIR/index.html'; mv '$PUBLIC_DIR/.metadata.json.tmp' '$PUBLIC_DIR/metadata.json'; mv '$PUBLIC_DIR/.releases.json.tmp' '$PUBLIC_DIR/releases.json'"

REMOTE_SHA=$(ssh "$PUBLISH_HOST" "sha256sum '$PUBLIC_DIR/latest.apk' | cut -d ' ' -f 1")
[[ "$REMOTE_SHA" == "$SHA256" ]] || { echo "Remote SHA256 mismatch" >&2; exit 1; }
curl --fail --silent --show-error --max-time 15 "$PUBLIC_URL/metadata.json" | jq -e --arg sha "$SHA256" '.sha256 == $sha' >/dev/null

echo "Published: $PUBLIC_URL"
echo "Version:   $VERSION_NAME ($VERSION_CODE)"
echo "SHA256:    $SHA256"

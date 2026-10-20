# 球员头像小尺寸版本 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 抓取器为每张球员头像生成 96px 与 240px 的正方形 JPEG 小图（与原图同目录、固定命名、长缓存），并提供补生成命令；小程序新增 `FiAvatar` 组件按规则优先加载小图、失败回退原图再回退首字，替换转会与战术板相关的球员头像，海报使用 240px 小图。

**Architecture:** 抓取器新增 `avatar_thumbs.py`（命名、裁剪缩放、上传、补生成），`upload_player_avatar` 上传原图后顺带生成小图；`AssetTarget.upload_bytes` 增加可选 `cache_control`；新 CLI `sync-avatar-thumbs`。不改数据库与后端。小程序新增 `utils/avatarThumb.ts` 与 `components/FiAvatar.vue`，替换 5 处头像，两个海报流程优先取 240 小图。

**Tech Stack:** Python 3.13 / uv / Pillow / boto3 / Typer / pytest；uni-app (Vue 3 + TS) / Bun test。

**Spec:** `docs/superpowers/specs/2026-10-20-avatar-thumbnails-design.md`

## Global Constraints

- **不写生产**：测试一律用假的下载器与存储（`FakeFetcher`/`FakeTarget` 模式）；**不要执行** `sync-avatar-thumbs` 的真实上传、不要运行 `sync-squads --upload-avatars`、不要连接生产 MinIO 或生产数据库。补生成由负责人在 review 后执行。
- 不改数据库表、不改后端代码；不删除原图；不处理球队队徽与用户头像。
- 小程序：组件化（`FiAvatar` 只收属性、不调接口，尺寸外观由使用方外层 view 控制；小程序组件标签上的 class 会落在宿主节点，布局类放在自己模板里的包裹 view 上）；token 化（全局 `style-token-audit`、`lineup-style-audit`、`transfer-style-audit` 自动覆盖）；只做微信小程序，不做 H5。
- 主包距 2MB 约 380KB：`FiAvatar` 与 `avatarThumb.ts` 在主包，保持精简。
- 不部署、不上传小程序、不改 `manifest.json` 版本号（构建后被改动就 `git checkout --` 还原 `manifest.json` 与 `src/config/generatedMiniProgramVersion.ts`）；不读不改 `football_insight_admin_android`；不打印、不提交 `.env` 与密钥。
- 提交信息：英文 subject，`feat(scraper): …` / `feat(mini): …` / `docs: …`，结尾 Co-Authored-By 行。

## Review Focus

1. 透明 PNG 头像——小图背景为白色而不是黑色。（Task 1 测试 `transparent_avatars_are_flattened_on_white`）
2. 小图生成失败（损坏的图片、存储报错）——原图上传与名单同步照常完成。（Task 2 测试 `thumbnail_failures_never_block_the_original_upload`）
3. 补生成中途某张下载失败——继续处理其余，最后以非 0 退出并给出失败数。（Task 3 测试 `backfill_continues_after_failures_and_reports_them`）
4. 小图还没生成（或 404）——小程序自动改用原图，原图也失败才显示首字。（Task 4 测试 `avatar_candidates_fall_back_to_the_original`）
5. 非 MinIO 球员目录的地址（用户头像、队徽、外部 URL）——原样使用，不拼小图地址。（Task 4 测试 `only_minio_player_avatars_get_thumbnails`）

---

### Task 1: 抓取器——小图命名与生成

**Files:**
- Create: `sina_csl_scraper/src/sina_csl_scraper/avatar_thumbs.py`
- Create: `sina_csl_scraper/tests/test_avatar_thumbs.py`

**Interfaces:**
- Produces: `THUMB_SIZES = (96, 240)`、`THUMB_CACHE_CONTROL = "public, max-age=31536000, immutable"`、`thumb_object_name(object_name: str, size: int) -> str`（不在 `/players/` 下或文件名为空时抛 `ValueError`）、`make_thumbnail(content: bytes, size: int) -> bytes`（JPEG）。

- [ ] **Step 1: 写失败测试**

`tests/test_avatar_thumbs.py`：

```python
from __future__ import annotations

import io

import pytest
from PIL import Image

from sina_csl_scraper.avatar_thumbs import THUMB_SIZES, make_thumbnail, thumb_object_name


def image_bytes(size: tuple[int, int], color, mode: str = "RGB", fmt: str = "PNG") -> bytes:
    out = io.BytesIO()
    Image.new(mode, size, color).save(out, fmt)
    return out.getvalue()


def open_jpeg(content: bytes) -> Image.Image:
    image = Image.open(io.BytesIO(content))
    assert image.format == "JPEG"
    return image


def test_thumb_names_keep_the_original_filename_and_add_jpg() -> None:
    assert THUMB_SIZES == (96, 240)
    assert thumb_object_name("summary/players/475933.png", 96) == "summary/players/thumbs/96/475933.png.jpg"
    assert thumb_object_name("summary/players/113121-team143-20260304.jpg", 240) == (
        "summary/players/thumbs/240/113121-team143-20260304.jpg.jpg"
    )
    for bad in ["summary/teams/1.png", "summary/players/", "players"]:
        with pytest.raises(ValueError):
            thumb_object_name(bad, 96)


def test_portraits_are_cropped_from_the_top_square() -> None:
    # 600x900 竖图：上方 600x600 为红色、下方为蓝色；取上方正方形后应为红色
    portrait = Image.new("RGB", (600, 900), (0, 0, 255))
    portrait.paste((255, 0, 0), (0, 0, 600, 600))
    out = io.BytesIO()
    portrait.save(out, "JPEG")
    thumb = open_jpeg(make_thumbnail(out.getvalue(), 96))
    assert thumb.size == (96, 96)
    r, g, b = thumb.convert("RGB").getpixel((48, 90))
    assert r > 200 and b < 60


def test_landscape_images_are_cropped_from_the_center() -> None:
    wide = Image.new("RGB", (300, 100), (0, 0, 255))
    wide.paste((0, 255, 0), (100, 0, 200, 100))
    out = io.BytesIO()
    wide.save(out, "PNG")
    thumb = open_jpeg(make_thumbnail(out.getvalue(), 96))
    assert thumb.size == (96, 96)
    r, g, b = thumb.convert("RGB").getpixel((5, 48))
    assert g > 200


def test_small_images_are_not_upscaled() -> None:
    thumb = open_jpeg(make_thumbnail(image_bytes((85, 85), (10, 20, 30)), 240))
    assert thumb.size == (85, 85)


def test_transparent_avatars_are_flattened_on_white() -> None:
    thumb = open_jpeg(make_thumbnail(image_bytes((120, 120), (0, 0, 0, 0), mode="RGBA"), 96))
    r, g, b = thumb.convert("RGB").getpixel((10, 10))
    assert min(r, g, b) > 240


def test_thumbnails_are_much_smaller_than_portraits() -> None:
    portrait = Image.effect_noise((600, 900), 64).convert("RGB")
    out = io.BytesIO()
    portrait.save(out, "JPEG", quality=95)
    assert len(make_thumbnail(out.getvalue(), 96)) * 10 < len(out.getvalue())
```

- [ ] **Step 2: 运行确认失败**

Run: `cd sina_csl_scraper && uv run pytest tests/test_avatar_thumbs.py -q 2>&1 | tail -3`
Expected: 失败，`ModuleNotFoundError: sina_csl_scraper.avatar_thumbs`。

- [ ] **Step 3: 实现**

`src/sina_csl_scraper/avatar_thumbs.py`：

```python
"""Square JPEG thumbnails for player avatars, stored next to the original in MinIO.

summary/players/<file> -> summary/players/thumbs/<size>/<file>.jpg
"""
from __future__ import annotations

import io

from PIL import Image, ImageOps

THUMB_SIZES = (96, 240)
THUMB_CACHE_CONTROL = "public, max-age=31536000, immutable"
_JPEG_QUALITY = 82


def thumb_object_name(object_name: str, size: int) -> str:
    head, separator, filename = object_name.rpartition("/players/")
    if not separator or not filename or "/" in filename:
        raise ValueError(f"not a player avatar object: {object_name}")
    return f"{head}/players/thumbs/{size}/{filename}.jpg"


def make_thumbnail(content: bytes, size: int) -> bytes:
    with Image.open(io.BytesIO(content)) as source:
        image = ImageOps.exif_transpose(source)
        width, height = image.size
        side = min(width, height)
        left = (width - side) // 2
        # 竖图（标准照）脸部在上方，取上方正方形；其他取居中正方形
        top = 0 if height > width else (height - side) // 2
        square = image.crop((left, top, left + side, top + side))
        if square.mode in ("RGBA", "LA") or (square.mode == "P" and "transparency" in square.info):
            rgba = square.convert("RGBA")
            flattened = Image.new("RGB", rgba.size, (255, 255, 255))
            flattened.paste(rgba, mask=rgba.getchannel("A"))
            square = flattened
        else:
            square = square.convert("RGB")
        if side > size:
            square = square.resize((size, size), Image.LANCZOS)
        out = io.BytesIO()
        square.save(out, "JPEG", quality=_JPEG_QUALITY, optimize=True)
        return out.getvalue()
```

- [ ] **Step 4: 运行确认通过**

Run: `uv run pytest tests/test_avatar_thumbs.py -q 2>&1 | tail -2`
Expected: `6 passed`

- [ ] **Step 5: 提交**

```bash
git add src/sina_csl_scraper/avatar_thumbs.py tests/test_avatar_thumbs.py
git commit -m "feat(scraper): generate square avatar thumbnails"
```

---

### Task 2: 抓取器——上传头像时顺带生成小图

**Files:**
- Modify: `sina_csl_scraper/src/sina_csl_scraper/assets.py`
- Modify: `sina_csl_scraper/src/sina_csl_scraper/avatar_thumbs.py`
- Modify: 所有实现 `upload_bytes` 的测试假对象（先 `grep -rn "def upload_bytes" sina_csl_scraper/tests sina_csl_scraper/src` 找全）
- Test: `sina_csl_scraper/tests/test_avatar_thumbs.py`

**Interfaces:**
- Consumes: Task 1。
- Produces: `AssetTarget.upload_bytes(self, object_name, content, content_type, cache_control: str | None = None) -> str`；`MinioAssetTarget.upload_bytes` 有 `cache_control` 时传 `CacheControl`；`upload_thumbnails(target, object_name: str, content: bytes) -> list[str]`；`upload_player_avatar(...)` 在原图上传成功后调用 `upload_thumbnails`，异常只记日志。

- [ ] **Step 1: 写失败测试**（追加到 `tests/test_avatar_thumbs.py`）

```python
from sina_csl_scraper.assets import MinioAssetTarget, upload_player_avatar
from sina_csl_scraper.avatar_thumbs import THUMB_CACHE_CONTROL, upload_thumbnails


class RecordingTarget:
    def __init__(self, fail_on: str | None = None) -> None:
        self.calls: list[tuple[str, str, str | None]] = []
        self.fail_on = fail_on

    def upload_bytes(self, object_name: str, content: bytes, content_type: str, cache_control: str | None = None) -> str:
        if self.fail_on and self.fail_on in object_name:
            raise RuntimeError("storage down")
        self.calls.append((object_name, content_type, cache_control))
        return f"https://cdn.example.com/{object_name}"


class StaticFetcher:
    def __init__(self, content: bytes, content_type: str = "image/png") -> None:
        self.content, self.content_type = content, content_type

    def fetch(self, url: str) -> tuple[bytes, str]:
        return self.content, self.content_type


def test_upload_thumbnails_writes_both_sizes_with_long_cache() -> None:
    target = RecordingTarget()
    urls = upload_thumbnails(target, "summary/players/7.png", image_bytes((150, 150), (1, 2, 3)))
    assert target.calls == [
        ("summary/players/thumbs/96/7.png.jpg", "image/jpeg", THUMB_CACHE_CONTROL),
        ("summary/players/thumbs/240/7.png.jpg", "image/jpeg", THUMB_CACHE_CONTROL),
    ]
    assert urls[0].endswith("/thumbs/96/7.png.jpg")


def test_player_avatar_upload_also_writes_thumbnails() -> None:
    target = RecordingTarget()
    result = upload_player_avatar(StaticFetcher(image_bytes((150, 150), (1, 2, 3))), target, "summary", 7, 999999, "https://src/7.png")
    assert result is not None
    names = [call[0] for call in target.calls]
    assert names[0] == "summary/players/7.png"
    assert names[1:] == ["summary/players/thumbs/96/7.png.jpg", "summary/players/thumbs/240/7.png.jpg"]


def test_thumbnail_failures_never_block_the_original_upload() -> None:
    broken = upload_player_avatar(StaticFetcher(b"not an image"), RecordingTarget(), "summary", 8, 999999, "https://src/8.png")
    assert broken is not None and broken[1] == "summary/players/8.png"
    target = RecordingTarget(fail_on="/thumbs/")
    stored = upload_player_avatar(StaticFetcher(image_bytes((150, 150), (1, 2, 3))), target, "summary", 9, 999999, "https://src/9.png")
    assert stored is not None
    assert [call[0] for call in target.calls] == ["summary/players/9.png"]


def test_minio_target_sends_cache_control_only_when_given() -> None:
    class Client:
        def __init__(self) -> None:
            self.kwargs: list[dict] = []

        def put_object(self, **kwargs) -> None:
            self.kwargs.append(kwargs)

    client = Client()
    target = MinioAssetTarget(client, "bucket", "https://cdn.example.com/bucket")
    target.upload_bytes("a.png", b"x", "image/png")
    target.upload_bytes("b.jpg", b"y", "image/jpeg", cache_control=THUMB_CACHE_CONTROL)
    assert "CacheControl" not in client.kwargs[0]
    assert client.kwargs[1]["CacheControl"] == THUMB_CACHE_CONTROL
```

（`999999` 是一个不会命中 `PLAYER_AVATAR_OVERRIDES` 的球队 id；如果 `upload_player_avatar` 的签名与计划不同，以仓库现有签名为准并记 Ruling。）

- [ ] **Step 2: 运行确认失败**

Run: `uv run pytest tests/test_avatar_thumbs.py -q 2>&1 | tail -3`
Expected: 新增 4 个测试失败（`upload_thumbnails` 不存在、`upload_bytes` 不接受 `cache_control`）。

- [ ] **Step 3: 实现**

`avatar_thumbs.py` 追加：

```python
import logging

logger = logging.getLogger(__name__)


def upload_thumbnails(target, object_name: str, content: bytes) -> list[str]:
    """Upload every thumbnail size for one original; raises on the first failure."""
    return [
        target.upload_bytes(
            thumb_object_name(object_name, size),
            make_thumbnail(content, size),
            "image/jpeg",
            cache_control=THUMB_CACHE_CONTROL,
        )
        for size in THUMB_SIZES
    ]
```

`assets.py`：
1. `AssetTarget` 协议改为 `def upload_bytes(self, object_name: str, content: bytes, content_type: str, cache_control: str | None = None) -> str: ...`。
2. `MinioAssetTarget.upload_bytes` 增加同名参数；构造 `put_object` 参数字典，`cache_control` 非空时加 `CacheControl=cache_control`。
3. `upload_player_avatar` 在 `target.upload_bytes(...)` 之后：

```python
    storage_url = target.upload_bytes(object_name, content, content_type)
    try:
        upload_thumbnails(target, object_name, content)
    except Exception:  # noqa: BLE001 - 小图是增强，失败不影响原图与名单同步
        logger.warning("avatar thumbnails failed for %s", object_name, exc_info=True)
    return source_url, object_name, storage_url
```

（`from .avatar_thumbs import upload_thumbnails`；在 `assets.py` 顶部定义 `logger = logging.getLogger(__name__)`。注意循环引用：`avatar_thumbs.py` 不要 import `assets.py`。）

4. 把仓库里所有 `upload_bytes` 假实现补上 `cache_control: str | None = None` 参数（只加参数，不改行为）。若某些现有测试因原图上传后多出两次小图上传而断言 `calls` 长度失败，按"原图之后多两条小图记录"更新断言，并在进度文件记 Ruling（列出改了哪些断言）。

- [ ] **Step 4: 运行抓取器全部测试**

Run: `uv run pytest -q 2>&1 | tail -3`
Expected: 全部通过（跳过数与改动前一致）。

- [ ] **Step 5: 提交**

```bash
git add src/sina_csl_scraper/assets.py src/sina_csl_scraper/avatar_thumbs.py tests
git commit -m "feat(scraper): upload avatar thumbnails alongside originals"
```

---

### Task 3: 抓取器——补生成命令

**Files:**
- Modify: `sina_csl_scraper/src/sina_csl_scraper/avatar_thumbs.py`、`assets.py`（`MinioAssetTarget.exists`）、`cli.py`
- Test: `sina_csl_scraper/tests/test_avatar_thumbs.py`

**Interfaces:**
- Produces: `MinioAssetTarget.exists(object_name) -> bool`（`head_object`；404/NoSuchKey 为 False，其他错误向上抛）；`sync_avatar_thumbs(urls: list[str], public_base_url: str, prefix: str, fetcher, target, dry_run: bool = False, limit: int | None = None) -> dict[str, int]`（键：`candidates`、`skipped`、`generated`、`failed`、`ignored`）；CLI `sync-avatar-thumbs`（`--dry-run`、`--limit`，参数与 `sync-squads` 的 MinIO/数据库参数相同），有失败时 `raise typer.Exit(1)`。

- [ ] **Step 1: 写失败测试**（追加）

```python
from sina_csl_scraper.avatar_thumbs import sync_avatar_thumbs


class ExistingTarget(RecordingTarget):
    def __init__(self, existing: set[str]) -> None:
        super().__init__()
        self.existing = existing

    def exists(self, object_name: str) -> bool:
        return object_name in self.existing


class MapFetcher:
    def __init__(self, contents: dict[str, bytes]) -> None:
        self.contents = contents
        self.calls: list[str] = []

    def fetch(self, url: str) -> tuple[bytes, str]:
        self.calls.append(url)
        if url not in self.contents:
            raise RuntimeError("download failed")
        return self.contents[url], "image/png"


BASE = "https://oryjk.cn:82/minio/football-insight"


def test_backfill_skips_existing_and_ignores_foreign_urls() -> None:
    png = image_bytes((150, 150), (1, 2, 3))
    urls = [f"{BASE}/summary/players/1.png", f"{BASE}/summary/players/2.png", f"{BASE}/summary/teams/1.png", "https://other.example/p.png"]
    target = ExistingTarget({"summary/players/thumbs/96/1.png.jpg", "summary/players/thumbs/240/1.png.jpg"})
    fetcher = MapFetcher({urls[1]: png})
    summary = sync_avatar_thumbs(urls, BASE, "summary", fetcher, target)
    assert summary == {"candidates": 2, "skipped": 1, "generated": 1, "failed": 0, "ignored": 2}
    assert fetcher.calls == [urls[1]]
    assert [call[0] for call in target.calls] == ["summary/players/thumbs/96/2.png.jpg", "summary/players/thumbs/240/2.png.jpg"]


def test_backfill_dry_run_never_downloads_or_writes() -> None:
    urls = [f"{BASE}/summary/players/1.png"]
    target, fetcher = ExistingTarget(set()), MapFetcher({})
    summary = sync_avatar_thumbs(urls, BASE, "summary", fetcher, target, dry_run=True)
    assert summary["candidates"] == 1 and summary["generated"] == 0
    assert fetcher.calls == [] and target.calls == []


def test_backfill_continues_after_failures_and_reports_them() -> None:
    png = image_bytes((150, 150), (1, 2, 3))
    urls = [f"{BASE}/summary/players/{i}.png" for i in (1, 2, 3)]
    summary = sync_avatar_thumbs(urls, BASE, "summary", MapFetcher({urls[0]: png, urls[2]: png}), ExistingTarget(set()))
    assert (summary["generated"], summary["failed"]) == (2, 1)


def test_backfill_limit_and_duplicate_urls() -> None:
    png = image_bytes((150, 150), (1, 2, 3))
    urls = [f"{BASE}/summary/players/{i}.png" for i in (1, 1, 2, 3)]
    fetcher = MapFetcher({u: png for u in urls})
    summary = sync_avatar_thumbs(urls, BASE, "summary", fetcher, ExistingTarget(set()), limit=2)
    assert summary["candidates"] == 2 and summary["generated"] == 2
```

- [ ] **Step 2: 运行确认失败**

Run: `uv run pytest tests/test_avatar_thumbs.py -q 2>&1 | tail -3`
Expected: 新增 4 个测试失败（`sync_avatar_thumbs` 不存在）。

- [ ] **Step 3: 实现**

`avatar_thumbs.py` 追加：

```python
def sync_avatar_thumbs(urls, public_base_url: str, prefix: str, fetcher, target, dry_run: bool = False, limit: int | None = None) -> dict[str, int]:
    """Generate missing thumbnails for existing player avatars (idempotent)."""
    base = f"{public_base_url.rstrip('/')}/"
    player_prefix = f"{prefix.strip('/')}/players/"
    summary = {"candidates": 0, "skipped": 0, "generated": 0, "failed": 0, "ignored": 0}
    seen: set[str] = set()
    for url in urls:
        if url in seen:
            continue
        seen.add(url)
        object_name = url[len(base):] if url.startswith(base) else ""
        if not object_name.startswith(player_prefix) or "/" in object_name[len(player_prefix):]:
            summary["ignored"] += 1
            continue
        if limit is not None and summary["candidates"] >= limit:
            break
        summary["candidates"] += 1
        if dry_run:
            continue
        try:
            if all(target.exists(thumb_object_name(object_name, size)) for size in THUMB_SIZES):
                summary["skipped"] += 1
                continue
            content, _ = fetcher.fetch(url)
            upload_thumbnails(target, object_name, content)
            summary["generated"] += 1
        except Exception:  # noqa: BLE001 - 单张失败继续，最后汇总
            logger.warning("thumbnail backfill failed for %s", url, exc_info=True)
            summary["failed"] += 1
    return summary
```

`assets.py` 的 `MinioAssetTarget` 增加：

```python
    def exists(self, object_name: str) -> bool:
        if self.client is None:
            return False
        try:
            self.client.head_object(Bucket=self.bucket, Key=object_name)
            return True
        except Exception as error:  # botocore ClientError
            code = getattr(error, "response", {}).get("Error", {}).get("Code")
            if code in ("404", "NoSuchKey", "NotFound"):
                return False
            raise
```

`cli.py` 新增命令（参数照抄 `sync-squads` 的数据库与 MinIO 选项，`minio_public_base_url` 为空时用 `default_public_base_url(minio_endpoint, minio_bucket)`）：

```python
@app.command("sync-avatar-thumbs")
def sync_avatar_thumbs_command(
    dry_run: bool = typer.Option(False, "--dry-run", help="Only count candidates; no download, no upload."),
    limit: int | None = typer.Option(None, help="Process at most N avatars."),
    database_url: str | None = typer.Option(None, envvar=["FI_DATABASE_URL", "DATABASE_URL"]),
    # …与 sync-squads 相同的 minio_* 选项…
) -> None:
    """Backfill 96/240px thumbnails for existing player avatars (idempotent)."""
    if not database_url:
        raise typer.BadParameter("PostgreSQL database_url is required")
    if not dry_run and (not minio_access_key or not minio_secret_key):
        raise typer.BadParameter("MinIO credentials are required unless --dry-run")
    target = MinioAssetTarget.from_credentials(...)  # 与 sync-squads 相同
    with psycopg.connect(database_url, autocommit=True) as connection:
        urls = [row[0] for row in connection.execute(
            "SELECT DISTINCT avatar_storage_url FROM f_i_squad_players WHERE avatar_storage_url IS NOT NULL ORDER BY 1"
        ).fetchall()]
    public_base = minio_public_base_url or default_public_base_url(minio_endpoint, minio_bucket)
    summary = sync_avatar_thumbs(urls, public_base, minio_prefix, HttpAssetFetcher(), target, dry_run=dry_run, limit=limit)
    typer.echo(json.dumps(summary, ensure_ascii=False))
    if summary["failed"]:
        raise typer.Exit(1)
```

（只读 `SELECT`；**不要**真实运行这个命令。）

- [ ] **Step 4: 运行抓取器全部测试与命令帮助**

Run: `uv run pytest -q 2>&1 | tail -2 && uv run sina-csl-scraper sync-avatar-thumbs --help | head -5`
Expected: 全部通过；帮助信息正常输出（只看帮助，不执行）。

- [ ] **Step 5: 提交**

```bash
git add src/sina_csl_scraper/avatar_thumbs.py src/sina_csl_scraper/assets.py src/sina_csl_scraper/cli.py tests/test_avatar_thumbs.py
git commit -m "feat(scraper): add an idempotent avatar thumbnail backfill command"
```

---

### Task 4: 小程序——`FiAvatar` 与各处替换

**Files:**
- Create: `football_insight_mini/src/utils/avatarThumb.ts`、`src/utils/avatarThumb.test.ts`、`src/components/FiAvatar.vue`
- Modify: `src/pages/transfer/components/TransferNominateSheet.vue`、`src/components/FiTransferCard.vue`、`src/pages/lineup/components/LineupPlayerSheet.vue`、`src/pages/lineup/components/LineupPlayerDot.vue`、`src/pages/insights/components/InsightsConsensusCard.vue`、`src/pages/transfer/useTransferShare.ts`、`src/pages/lineup/useLineupShare.ts`、`scripts/lazy-avatars.test.ts`

**Interfaces:**
- Produces: `type AvatarThumbSize = 96 | 240`；`avatarThumbUrl(url, size): string`；`avatarCandidates(url, size): string[]`（去重后的 `[小图, 原图]`，非球员目录地址只有 `[原图]`，空地址为 `[]`）；`FiAvatar` 属性 `{ src: string | null; size: AvatarThumbSize; name: string; lazy?: boolean }`。

- [ ] **Step 1: 先重新读取上面列出的所有 Modify 文件**（这些文件近期被改过：`lazy-load`、表态文案、共识卡片、海报）。

- [ ] **Step 2: 写失败测试**

`src/utils/avatarThumb.test.ts`：

```ts
import { describe, expect, test } from 'bun:test'
import { avatarCandidates, avatarThumbUrl } from './avatarThumb'

const BASE = 'https://oryjk.cn:82/minio/football-insight/summary/players'

describe('avatar thumbnails', () => {
  test('only_minio_player_avatars_get_thumbnails', () => {
    expect(avatarThumbUrl(`${BASE}/475933.png`, 96)).toBe(`${BASE}/thumbs/96/475933.png.jpg`)
    expect(avatarThumbUrl(`${BASE}/113121-team143-20260304.jpg`, 240)).toBe(`${BASE}/thumbs/240/113121-team143-20260304.jpg.jpg`)
    for (const url of ['https://oryjk.cn:82/minio/football-insight/summary/teams/1.png', 'https://thirdwx.qlogo.cn/abc/132', 'https://example.com/players/1.png']) {
      expect(avatarThumbUrl(url, 96)).toBe(url)
    }
    expect(avatarThumbUrl(null, 96)).toBe('')
    expect(avatarThumbUrl('', 96)).toBe('')
  })

  test('avatar_candidates_fall_back_to_the_original', () => {
    expect(avatarCandidates(`${BASE}/1.png`, 96)).toEqual([`${BASE}/thumbs/96/1.png.jpg`, `${BASE}/1.png`])
    expect(avatarCandidates('https://thirdwx.qlogo.cn/abc/132', 96)).toEqual(['https://thirdwx.qlogo.cn/abc/132'])
    expect(avatarCandidates(null, 96)).toEqual([])
  })
})
```

`scripts/lazy-avatars.test.ts`：把检查从"`<image class=…>` 带 `lazy-load`"改为"文件里的 `<FiAvatar` 标签带 `lazy`"（替换后这 4 个长列表都用 `FiAvatar`）：

```ts
test('avatars in long lists load lazily', () => {
  const missing = LONG_LISTS.filter(([file]) => {
    const tags = [...readFileSync(join(SRC, file), 'utf8').matchAll(/<FiAvatar\b[^>]*>/g)].map(m => m[0])
    return tags.length === 0 || tags.some(tag => !/\blazy\b/.test(tag))
  }).map(([file]) => file)
  expect(missing).toEqual([])
})
```

（`LONG_LISTS` 只保留 `TransferNominateSheet`、`LineupPlayerSheet`、`FiTransferCard` 三项；`FiLineupCommunityCard` 是作者的微信头像，不在球员目录，不换 `FiAvatar`，从列表移除并在注释说明它仍用 `<image lazy-load>`。）

- [ ] **Step 3: 运行确认失败**

Run: `cd football_insight_mini && bun test src/utils/avatarThumb.test.ts scripts/lazy-avatars.test.ts 2>&1 | grep -E "^\(fail|^ *[0-9]+ (pass|fail)"`
Expected: 失败（模块不存在；长列表还没用 `FiAvatar`）。

- [ ] **Step 4: 实现工具与组件**

`src/utils/avatarThumb.ts`：

```ts
export type AvatarThumbSize = 96 | 240

/** MinIO 球员目录的头像 → 同目录 thumbs/<size>/<文件名>.jpg；其他地址原样返回。 */
export function avatarThumbUrl(url: string | null | undefined, size: AvatarThumbSize): string {
  if (!url) return ''
  const match = /^(https?:\/\/[^?#]*\/minio\/[^?#]*\/players\/)([^/?#]+)$/.exec(url)
  return match ? `${match[1]}thumbs/${size}/${match[2]}.jpg` : url
}

/** 依次尝试的地址：小图 → 原图（去重）；都失败时由组件显示首字。 */
export function avatarCandidates(url: string | null | undefined, size: AvatarThumbSize): string[] {
  if (!url) return []
  return [...new Set([avatarThumbUrl(url, size), url])]
}
```

`src/components/FiAvatar.vue`：

```vue
<template>
  <!-- 球员头像：先小图，加载失败改用原图，再失败显示名字首字。尺寸与圆角由使用方的外层 view 决定。 -->
  <image v-if="current" class="fi-avatar" :src="current" mode="aspectFill" :lazy-load="lazy" @error="next" />
  <view v-else class="fi-avatar fi-avatar--fallback"><text>{{ initial }}</text></view>
</template>
<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { avatarCandidates, type AvatarThumbSize } from '../utils/avatarThumb'
const props = withDefaults(defineProps<{ src: string | null; size: AvatarThumbSize; name: string; lazy?: boolean }>(), { lazy: false })
const index = ref(0)
const candidates = computed(() => avatarCandidates(props.src, props.size))
const current = computed(() => candidates.value[index.value] ?? '')
const initial = computed(() => Array.from(props.name.trim())[0] ?? '')
watch(() => props.src, () => { index.value = 0 })
function next(): void { index.value += 1 }
</script>
<style scoped>
.fi-avatar { display: block; width: 100%; height: 100%; border-radius: inherit; }
.fi-avatar--fallback { display: flex; align-items: center; justify-content: center; color: var(--fi-color-text-secondary); font-size: inherit; font-weight: var(--fi-weight-bold); }
</style>
```

- [ ] **Step 5: 替换各处头像**

原则：原来的 `<image … class="X__avatar">` 与它的首字占位 `<view class="X__avatar X__avatar--fallback">` 一起替换为 `<view class="X__avatar"><FiAvatar … /></view>`；`X__avatar` 原有的尺寸、圆角、背景保留，并补 `overflow: hidden`；原 `--fallback` 样式里的字号移到 `X__avatar` 上（`FiAvatar` 的首字继承 `font-size`）。

- `TransferNominateSheet.vue`：`<view class="nominate__avatar"><FiAvatar :src="player.avatar_url" :size="96" :name="player.name" lazy /></view>`
- `FiTransferCard.vue`：`<view class="transfer-card__avatar"><FiAvatar :src="card.player.avatar_url" :size="96" :name="card.player.name" lazy /></view>`
- `LineupPlayerSheet.vue`：`<view class="row__avatar"><FiAvatar :src="item.avatar_url" :size="96" :name="item.name" lazy /></view>`（无头像时原逻辑显示号码：保留 `v-if="item.avatar_url"` 分支用 `FiAvatar`，`v-else` 分支的号码占位不变）
- `LineupPlayerDot.vue`：头像分支改为 `<view class="dot__avatar"><FiAvatar :src="player.player.avatar_url" :size="96" :name="player.player.name" /></view>`（号码角标、"前"、"独家"标记不变）
- `InsightsConsensusCard.vue`：`<view class="consensus__avatar"><FiAvatar :src="entry.player.avatar_url" :size="96" :name="entry.player.name" /></view>`

各文件 `import FiAvatar from '…/components/FiAvatar.vue'`（相对路径按所在目录）。

- [ ] **Step 6: 海报优先用 240 小图**

`useTransferShare.ts` 与 `useLineupShare.ts` 的 `render` 中，把"`imageInfo(url)` 加载每个地址"改为依次尝试候选地址，结果仍以**原地址**为键存入 `images`（绘制函数按原地址取图，不用改）：

```ts
import { avatarCandidates } from '../../utils/avatarThumb'

async function loadImage(url: string): Promise<string> {
  for (const candidate of avatarCandidates(url, 240)) {
    try { return await imageInfo(candidate) } catch { /* 试下一个 */ }
  }
  throw new Error('image unavailable')
}
// …原来的 images.set(url, await imageInfo(url)) 改为 images.set(url, await loadImage(url))
```

- [ ] **Step 7: 检查与构建**

Run:

```bash
bun test > /tmp/thumbs-mini.log 2>&1; grep -E "^\(fail|^ *[0-9]+ (pass|fail)" /tmp/thumbs-mini.log
bun run type-check 2>&1 | tail -1 && bun run check:boundaries | tail -1
MINI_REVIEW_SKIP=1 bun run build:mp-weixin > /tmp/thumbs-build.log 2>&1; echo build=$?
bun run verify:mp-components | tail -1
```

Expected: 0 fail；类型、边界通过；`build=0`；`FiAvatar` 已注册。构建后还原被改动的版本号文件。（新工作目录若缺 `.env.production` 导致构建门禁拦截，按 `scripts/mini-release-guards.mjs` 的要求在本地创建，只放公开配置，不提交。）

- [ ] **Step 8: 提交**

```bash
git add src/utils/avatarThumb.ts src/utils/avatarThumb.test.ts src/components/FiAvatar.vue src/pages/transfer src/components/FiTransferCard.vue src/pages/lineup src/pages/insights/components/InsightsConsensusCard.vue scripts/lazy-avatars.test.ts
git commit -m "feat(mini): load player avatar thumbnails with fallback to originals"
```

---

### Task 5: 文档

- [ ] **Step 1**：`CHANGELOG.md` 顶部新增"2026-10-20 · 球员头像小尺寸版本（代码完成，待补生成与上传）"：问题与排查数字（spec 第 1 节）、方案、验证（实际测试数字）、发布状态（**小图尚未补生成**：负责人确认后执行 `uv run sina-csl-scraper sync-avatar-thumbs --dry-run` 再正式执行；补生成前小程序会自动回退原图，不会显示坏图；小程序未上传）。`DEPLOYMENT.md` 抓取器相关小节补补生成命令与先 `--dry-run` 的步骤；`FEATURES.md` 补一行。
- [ ] **Step 2**：

```bash
git add CHANGELOG.md DEPLOYMENT.md FEATURES.md
git commit -m "docs: record avatar thumbnails"
```

---

## Self-Review 记录

- **Spec 覆盖**：命名规则、尺寸、裁剪（竖图取上、其余居中、不放大、透明铺白）、JPEG 82、长缓存 → Task 1–2；上传时顺带生成且失败不阻断 → Task 2；补生成（跳过已存在、dry-run、limit、单张失败继续、非 0 退出、只处理球员目录）→ Task 3；小程序换算、`FiAvatar` 回退、5 处替换、海报 240、长列表 lazy → Task 4。
- **已裁定**：作者（微信用户）头像不在球员目录，`FiLineupCommunityCard` 不换组件，仍用 `<image lazy-load>`；`FiAvatar` 回退逻辑由纯函数 `avatarCandidates` 覆盖，组件本身只按下标切换。

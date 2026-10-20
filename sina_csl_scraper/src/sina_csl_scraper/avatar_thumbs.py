"""Square JPEG thumbnails for player avatars, stored next to the original in MinIO.

summary/players/<file> -> summary/players/thumbs/<size>/<file>.jpg
"""
from __future__ import annotations

import io
import logging

from PIL import Image, ImageOps

logger = logging.getLogger(__name__)

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


def sync_avatar_thumbs(
    urls, public_base_url: str, prefix: str, fetcher, target, dry_run: bool = False, limit: int | None = None
) -> dict[str, int]:
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

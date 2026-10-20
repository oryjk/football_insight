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

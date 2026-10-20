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

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

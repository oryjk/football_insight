from __future__ import annotations

import pytest

from sina_csl_scraper.assets import (
    AssetUploader,
    MinioAssetTarget,
    default_public_base_url,
)
from sina_csl_scraper.models import PlayerProfile, TeamProfile
from sina_csl_scraper.models import SquadPlayer
from sina_csl_scraper.squad_sync import make_avatar_uploader


class FakeFetcher:
    def __init__(self) -> None:
        self.calls: list[str] = []

    def fetch(self, url: str) -> tuple[bytes, str]:
        self.calls.append(url)
        return (b"png-data", "image/png")


class FakeTarget:
    def __init__(self) -> None:
        self.calls: list[tuple[str, bytes, str]] = []

    def upload_bytes(
        self, object_name: str, content: bytes, content_type: str, cache_control: str | None = None
    ) -> str:
        self.calls.append((object_name, content, content_type))
        return f"https://cdn.example.com/{object_name}"


def test_asset_uploader_uploads_team_and_player_avatars() -> None:
    fetcher = FakeFetcher()
    target = FakeTarget()
    uploader = AssetUploader(fetcher=fetcher, target=target)

    teams = [
        TeamProfile(
            team_id=77680,
            team_name="成都蓉城",
            avatar_source_url="https://cdn.example.com/teams/77680.png",
        )
    ]
    players = [
        PlayerProfile(
            player_id=999001,
            player_name="未覆盖球员",
            team_id=77680,
            team_name="成都蓉城",
            avatar_source_url="https://cdn.example.com/players/999001.png",
        )
    ]

    synced_teams = uploader.upload_team_avatars(teams, prefix="summary")
    synced_players = uploader.upload_player_avatars(players, prefix="summary")

    assert synced_teams[0].avatar_object_name == "summary/teams/77680.png"
    assert synced_teams[0].avatar_storage_url == "https://cdn.example.com/summary/teams/77680.png"
    assert synced_players[0].avatar_object_name == "summary/players/999001.png"
    assert synced_players[0].avatar_storage_url == "https://cdn.example.com/summary/players/999001.png"
    assert len(target.calls) == 2


def test_minio_asset_target_builds_public_url() -> None:
    target = MinioAssetTarget(
        client=None,
        bucket="football-insight",
        public_base_url="https://oryjk.cn:82/minio/football-insight",
    )

    url = target.build_public_url("summary/teams/77680.png")

    assert url == "https://oryjk.cn:82/minio/football-insight/summary/teams/77680.png"


def test_default_public_base_url_supports_path_prefixed_endpoint() -> None:
    url = default_public_base_url(
        endpoint="https://oryjk.cn:82/minio",
        bucket="football-insight",
    )

    assert url == "https://oryjk.cn:82/minio/football-insight"


def test_default_public_base_url_supports_root_endpoint() -> None:
    url = default_public_base_url(
        endpoint="http://127.0.0.1:9000",
        bucket="football-insight",
    )

    assert url == "http://127.0.0.1:9000/football-insight"


@pytest.mark.parametrize("player_id,image_id", [
    (184911, "800b9c6aa26f4ca2a293ecacff9c81f5"),
    (520149, "c6180d34d2be4c95b331b191df8f6df3"),
    (204211, "2b4dc1e566724cebaa3cc8948b0a87a1"),
    (446237, "ab7df141c2a1451e8e9c192d57246f98"),
    (446339, "b194b7a9c8f042929fe0586bde39b218"),
    (7727346, "b758ecd6a4794738bcc89a4e587ce8dd"),
    (580161, "745ec0929e2e47bdb34bd02d91847791"),
    (8224558, "ff8c2515e7e14ffbac12d17030320210"),
])
def test_verified_chengdu_portraits_replace_old_provider_images_in_both_sync_paths(player_id, image_id):
    old = f"http://n.sinaimg.cn/ty/opta/player150/{player_id}.png"
    fetcher, target = FakeFetcher(), FakeTarget()
    profile = PlayerProfile(player_id, "球员", 77680, "成都蓉城", old)
    updated = AssetUploader(fetcher, target).upload_player_avatars([profile], "summary")[0]
    expected_object = f"summary/players/{player_id}-chengdu-20260228.jpg"
    assert image_id in fetcher.calls[0]
    assert updated.avatar_source_url == fetcher.calls[0]
    assert updated.avatar_object_name == expected_object
    squad_upload = make_avatar_uploader(fetcher, target, "summary")
    squad = SquadPlayer(77680, player_id, "球员", 7, "FW", "中国", old)
    assert squad_upload(squad) == updated.avatar_storage_url
    assert fetcher.calls[1] == fetcher.calls[0]
    assert [call[0] for call in target.calls] == [expected_object, expected_object]


def test_chengdu_portrait_override_does_not_apply_to_another_team():
    fetcher, target = FakeFetcher(), FakeTarget()
    source = "https://provider/another-team.png"
    profile = PlayerProfile(184911, "韦世豪", 999, "其他球队", source)
    updated = AssetUploader(fetcher, target).upload_player_avatars([profile], "summary")[0]
    assert fetcher.calls == [source]
    assert updated.avatar_object_name == "summary/players/184911.png"


@pytest.mark.parametrize("team_id,player_id", [(136,617722),(178,508435),(41300,7712350),(500,536482),(501,113139),(123380,6936990)])
def test_other_teams_use_reviewed_portraits_in_both_sync_paths(team_id, player_id):
    fetcher, target = FakeFetcher(), FakeTarget()
    source = f"https://provider/players/{player_id}.png"
    profile = PlayerProfile(player_id, "球员", team_id, "球队", source)
    updated = AssetUploader(fetcher, target).upload_player_avatars([profile], "summary")[0]
    assert ".itc.cn/" in fetcher.calls[0]
    assert updated.avatar_source_url == fetcher.calls[0]
    assert f"{player_id}-team{team_id}-2026" in updated.avatar_object_name
    squad = SquadPlayer(team_id, player_id, "球员", 7, "MF", "中国", source)
    assert make_avatar_uploader(fetcher, target, "summary")(squad) == updated.avatar_storage_url
    assert fetcher.calls[1] == fetcher.calls[0]


def test_unconfirmed_standard_portrait_keeps_existing_source():
    fetcher, target = FakeFetcher(), FakeTarget()
    source = "https://provider/players/86364.png"
    profile = PlayerProfile(86364, "李可", 144, "上海申花", source)
    updated = AssetUploader(fetcher, target).upload_player_avatars([profile], "summary")[0]
    assert fetcher.calls == [source]
    assert updated.avatar_object_name == "summary/players/86364.png"

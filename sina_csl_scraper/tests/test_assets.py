from __future__ import annotations

from sina_csl_scraper.assets import (
    AssetUploader,
    MinioAssetTarget,
    default_public_base_url,
)
from sina_csl_scraper.models import PlayerProfile, TeamProfile


class FakeFetcher:
    def __init__(self) -> None:
        self.calls: list[str] = []

    def fetch(self, url: str) -> tuple[bytes, str]:
        self.calls.append(url)
        return (b"png-data", "image/png")


class FakeTarget:
    def __init__(self) -> None:
        self.calls: list[tuple[str, bytes, str]] = []

    def upload_bytes(self, object_name: str, content: bytes, content_type: str) -> str:
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
            player_id=204211,
            player_name="费利佩",
            team_id=77680,
            team_name="成都蓉城",
            avatar_source_url="https://cdn.example.com/players/204211.png",
        )
    ]

    synced_teams = uploader.upload_team_avatars(teams, prefix="summary")
    synced_players = uploader.upload_player_avatars(players, prefix="summary")

    assert synced_teams[0].avatar_object_name == "summary/teams/77680.png"
    assert synced_teams[0].avatar_storage_url == "https://cdn.example.com/summary/teams/77680.png"
    assert synced_players[0].avatar_object_name == "summary/players/204211.png"
    assert synced_players[0].avatar_storage_url == "https://cdn.example.com/summary/players/204211.png"
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

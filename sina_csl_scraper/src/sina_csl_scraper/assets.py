from __future__ import annotations

from dataclasses import replace
from typing import Protocol
from urllib.parse import urlparse, urlunparse

import boto3
from botocore.config import Config
import requests

from .models import PlayerProfile, TeamProfile

# Manually verified 2026 Chengdu headshots; Sina still serves the previous clubs' photos.
# Name captions and date: https://www.sohu.com/picture/991014558 (2026-02-28).
# Team-scoped, versioned keys preserve the old objects and invalidate client image caches.
PLAYER_AVATAR_OVERRIDES: dict[tuple[int, int], tuple[str, str]] = {
    (77680, 184911): (
        "https://q3.itc.cn/q_85,c_zoom,w_600/images03/20260228/800b9c6aa26f4ca2a293ecacff9c81f5.jpeg",
        "184911-chengdu-20260228.jpg",
    ),
    (77680, 520149): (
        "https://q2.itc.cn/q_85,c_zoom,w_600/images03/20260228/c6180d34d2be4c95b331b191df8f6df3.jpeg",
        "520149-chengdu-20260228.jpg",
    ),
}


class AssetFetchError(RuntimeError):
    """Raised when a remote asset cannot be downloaded."""


class AssetFetcher(Protocol):
    def fetch(self, url: str) -> tuple[bytes, str]: ...


class AssetTarget(Protocol):
    def upload_bytes(self, object_name: str, content: bytes, content_type: str) -> str: ...


def upload_player_avatar(
    fetcher: AssetFetcher, target: AssetTarget, prefix: str,
    player_id: int, team_id: int, source_url: str | None,
) -> tuple[str, str, str] | None:
    override = PLAYER_AVATAR_OVERRIDES.get((team_id, player_id))
    if override:
        source_url = override[0]
    if not source_url:
        return None
    content, content_type = fetcher.fetch(source_url)
    filename = override[1] if override else f"{player_id}{_extension_for(source_url, content_type)}"
    object_name = f"{prefix.strip('/')}/players/{filename}"
    return source_url, object_name, target.upload_bytes(object_name, content, content_type)


class HttpAssetFetcher:
    def __init__(self, session: requests.Session | None = None, timeout: int = 20) -> None:
        self.session = session or requests.Session()
        if session is None:
            # Image CDNs may reject the default python-requests user agent.
            self.session.headers["User-Agent"] = "Mozilla/5.0 (compatible; FootballInsight/1.0)"
        self.timeout = timeout

    def fetch(self, url: str) -> tuple[bytes, str]:
        response = self.session.get(url, timeout=self.timeout)
        response.raise_for_status()
        content_type = response.headers.get("Content-Type", "application/octet-stream")
        return response.content, content_type


def _extension_for(url: str, content_type: str) -> str:
    lowered_url = url.lower()
    if lowered_url.endswith(".png") or "png" in content_type.lower():
        return ".png"
    if lowered_url.endswith(".jpg") or lowered_url.endswith(".jpeg") or "jpeg" in content_type.lower():
        return ".jpg"
    if lowered_url.endswith(".webp") or "webp" in content_type.lower():
        return ".webp"
    return ".png"


def _normalize_endpoint_url(endpoint: str, secure: bool = True) -> str:
    parsed = urlparse(endpoint)
    if parsed.scheme:
        return endpoint.rstrip("/")

    scheme = "https" if secure else "http"
    return f"{scheme}://{endpoint.rstrip('/')}"


def default_public_base_url(endpoint: str, bucket: str, secure: bool = True) -> str:
    endpoint_url = _normalize_endpoint_url(endpoint, secure=secure)
    parsed = urlparse(endpoint_url)
    endpoint_path = parsed.path.rstrip("/")
    public_path = f"{endpoint_path}/{bucket}" if endpoint_path else f"/{bucket}"

    return urlunparse((parsed.scheme, parsed.netloc, public_path, "", "", ""))


class MinioAssetTarget:
    def __init__(self, client: object | None, bucket: str, public_base_url: str) -> None:
        self.client = client
        self.bucket = bucket
        self.public_base_url = public_base_url.rstrip("/")

    @classmethod
    def from_credentials(
        cls,
        endpoint: str,
        access_key: str,
        secret_key: str,
        bucket: str,
        public_base_url: str | None = None,
        region: str = "us-east-1",
        secure: bool = True,
    ) -> "MinioAssetTarget":
        endpoint_url = _normalize_endpoint_url(endpoint, secure=secure)
        session = boto3.session.Session()
        client = session.client(
            "s3",
            endpoint_url=endpoint_url,
            aws_access_key_id=access_key,
            aws_secret_access_key=secret_key,
            region_name=region,
            verify=secure,
            config=Config(
                signature_version="s3v4",
                s3={"addressing_style": "path"},
            ),
        )
        return cls(
            client=client,
            bucket=bucket,
            public_base_url=public_base_url or default_public_base_url(endpoint_url, bucket),
        )

    def build_public_url(self, object_name: str) -> str:
        return f"{self.public_base_url}/{object_name.lstrip('/')}"

    def upload_bytes(self, object_name: str, content: bytes, content_type: str) -> str:
        if self.client is None:
            return self.build_public_url(object_name)

        self.client.put_object(
            Bucket=self.bucket,
            Key=object_name,
            Body=content,
            ContentType=content_type,
        )
        return self.build_public_url(object_name)


class AssetUploader:
    def __init__(self, fetcher: AssetFetcher, target: AssetTarget) -> None:
        self.fetcher = fetcher
        self.target = target

    def upload_team_avatars(self, teams: list[TeamProfile], prefix: str) -> list[TeamProfile]:
        updated: list[TeamProfile] = []
        for team in teams:
            if not team.avatar_source_url:
                updated.append(team)
                continue

            content, content_type = self.fetcher.fetch(team.avatar_source_url)
            object_name = f"{prefix.strip('/')}/teams/{team.team_id}{_extension_for(team.avatar_source_url, content_type)}"
            storage_url = self.target.upload_bytes(object_name, content, content_type)
            updated.append(
                replace(
                    team,
                    avatar_object_name=object_name,
                    avatar_storage_url=storage_url,
                )
            )
        return updated

    def upload_player_avatars(self, players: list[PlayerProfile], prefix: str) -> list[PlayerProfile]:
        updated: list[PlayerProfile] = []
        for player in players:
            uploaded = upload_player_avatar(
                self.fetcher, self.target, prefix, player.player_id, player.team_id, player.avatar_source_url,
            )
            if uploaded is None:
                updated.append(player)
                continue

            source_url, object_name, storage_url = uploaded
            updated.append(
                replace(
                    player,
                    avatar_source_url=source_url,
                    avatar_object_name=object_name,
                    avatar_storage_url=storage_url,
                )
            )
        return updated

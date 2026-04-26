from __future__ import annotations

from dataclasses import replace
from typing import Protocol
from urllib.parse import urlparse, urlunparse

import boto3
from botocore.config import Config
import requests

from .models import PlayerProfile, TeamProfile


class AssetFetchError(RuntimeError):
    """Raised when a remote asset cannot be downloaded."""


class AssetFetcher(Protocol):
    def fetch(self, url: str) -> tuple[bytes, str]: ...


class AssetTarget(Protocol):
    def upload_bytes(self, object_name: str, content: bytes, content_type: str) -> str: ...


class HttpAssetFetcher:
    def __init__(self, session: requests.Session | None = None, timeout: int = 20) -> None:
        self.session = session or requests.Session()
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
            if not player.avatar_source_url:
                updated.append(player)
                continue

            content, content_type = self.fetcher.fetch(player.avatar_source_url)
            object_name = f"{prefix.strip('/')}/players/{player.player_id}{_extension_for(player.avatar_source_url, content_type)}"
            storage_url = self.target.upload_bytes(object_name, content, content_type)
            updated.append(
                replace(
                    player,
                    avatar_object_name=object_name,
                    avatar_storage_url=storage_url,
                )
            )
        return updated

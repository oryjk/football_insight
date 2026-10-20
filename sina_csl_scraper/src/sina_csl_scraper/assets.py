from __future__ import annotations

import logging
from dataclasses import replace
from typing import Protocol
from urllib.parse import urlparse, urlunparse

import boto3
from botocore.config import Config
import requests

from .models import PlayerProfile, TeamProfile

from .avatar_thumbs import upload_thumbnails
from .player_portraits import PLAYER_AVATAR_OVERRIDES

logger = logging.getLogger(__name__)


class AssetFetchError(RuntimeError):
    """Raised when a remote asset cannot be downloaded."""


class AssetFetcher(Protocol):
    def fetch(self, url: str) -> tuple[bytes, str]: ...


class AssetTarget(Protocol):
    def upload_bytes(
        self, object_name: str, content: bytes, content_type: str, cache_control: str | None = None
    ) -> str: ...


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
    storage_url = target.upload_bytes(object_name, content, content_type)
    try:
        upload_thumbnails(target, object_name, content)
    except Exception:  # noqa: BLE001 - 小图是增强，失败不影响原图与名单同步
        logger.warning("avatar thumbnails failed for %s", object_name, exc_info=True)
    return source_url, object_name, storage_url


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


def _sign_without_proxy_prefix(client, prefix: str) -> None:
    """反向代理（如 oryjk.cn:82/minio）转发前会去掉路径前缀，MinIO 按去掉前缀的路径验签。
    签名时临时去掉前缀，发送前恢复原地址，请求仍经代理转发。"""
    if not prefix:
        return
    original: dict[int, str] = {}

    def strip_prefix(request, **_kwargs) -> None:
        parsed = urlparse(request.url)
        if parsed.path.startswith(prefix + "/"):
            original[id(request)] = request.url
            request.url = urlunparse(parsed._replace(path=parsed.path[len(prefix):]))

    def restore_prefix(request, **_kwargs) -> None:
        url = original.pop(id(request), None)
        if url is not None:
            request.url = url

    client.meta.events.register_first("before-sign.s3", strip_prefix)
    # 签名器挂在 request-created 上；排在它之后恢复，后续准备与发送都用带前缀的地址。
    client.meta.events.register_last("request-created.s3", restore_prefix)


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
        _sign_without_proxy_prefix(client, urlparse(endpoint_url).path.rstrip("/"))
        return cls(
            client=client,
            bucket=bucket,
            public_base_url=public_base_url or default_public_base_url(endpoint_url, bucket),
        )

    def build_public_url(self, object_name: str) -> str:
        return f"{self.public_base_url}/{object_name.lstrip('/')}"

    def upload_bytes(
        self, object_name: str, content: bytes, content_type: str, cache_control: str | None = None
    ) -> str:
        if self.client is None:
            return self.build_public_url(object_name)

        kwargs = {
            "Bucket": self.bucket,
            "Key": object_name,
            "Body": content,
            "ContentType": content_type,
        }
        if cache_control:
            kwargs["CacheControl"] = cache_control
        self.client.put_object(**kwargs)
        return self.build_public_url(object_name)

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

"""Content-addressed original news and timezone-safe publication metadata."""
from __future__ import annotations

from dataclasses import dataclass
from datetime import datetime
from email.utils import parsedate_to_datetime
import hashlib
import json
import re
from urllib.parse import urlsplit, urlunsplit
from zoneinfo import ZoneInfo

SHANGHAI = ZoneInfo("Asia/Shanghai")


def canonical_url(value: str) -> str:
    if not isinstance(value, str):
        raise ValueError("Source URL must be HTTP(S)")
    parts = urlsplit(value.strip())
    if parts.scheme.lower() not in {"http", "https"} or not parts.hostname or parts.username or parts.password:
        raise ValueError("Source URL must be HTTP(S) without credentials")
    return urlunsplit((parts.scheme.lower(), parts.netloc.lower(), parts.path or "/", parts.query, ""))


def parse_timestamp(value: str | datetime, now: datetime | None = None) -> datetime:
    """Accept aware ISO8601/RFC2822 only; never infer a missing timezone."""
    try:
        if isinstance(value, datetime):
            result = value
        elif isinstance(value, str) and value.strip():
            try:
                result = datetime.fromisoformat(value.strip().replace("Z", "+00:00"))
            except ValueError:
                result = parsedate_to_datetime(value.strip())
        else:
            raise ValueError
        if result.tzinfo is None or result.utcoffset() is None:
            raise ValueError
        result = result.astimezone(SHANGHAI)
        reference = now or datetime.now(SHANGHAI)
        if reference.tzinfo is None or reference.utcoffset() is None or result > reference:
            raise ValueError
        return result
    except (ValueError, TypeError, OverflowError, AttributeError):
        raise ValueError("Publication time must be known, timezone-aware, and not in the future") from None


@dataclass(frozen=True)
class SourceArticle:
    title: str
    url: str
    publisher: str
    published_at: datetime
    body: str

    def __post_init__(self) -> None:
        for value in (self.title, self.publisher, self.body):
            if not isinstance(value, str) or not value.strip() or re.search(r"<[^>]+>|[\x00-\x08\x0b\x0c\x0e-\x1f]", value):
                raise ValueError("Source title, publisher and original body are required")
        if not isinstance(self.published_at, datetime) or self.published_at.tzinfo is None or self.published_at.utcoffset() is None:
            raise ValueError("Source publication time must be timezone-aware")
        object.__setattr__(self, "url", canonical_url(self.url))
        object.__setattr__(self, "published_at", self.published_at.astimezone(SHANGHAI))

    @property
    def source_id(self) -> str:
        return hashlib.sha256((self.url + "\n" + self.body).encode("utf-8")).hexdigest()

    def as_source(self) -> dict:
        return {"title": self.title, "url": self.url, "publisher": self.publisher, "published_at": self.published_at.isoformat()}


def source_fingerprint(sources: list[SourceArticle]) -> str:
    material = sorted({json.dumps({**source.as_source(), "source_id": source.source_id}, ensure_ascii=False, sort_keys=True) for source in sources})
    return hashlib.sha256(json.dumps(material, ensure_ascii=False).encode("utf-8")).hexdigest()


fingerprint_sources = source_fingerprint

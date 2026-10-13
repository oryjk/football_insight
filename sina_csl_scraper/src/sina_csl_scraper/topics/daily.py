"""Daily publication keeps the previous edition until new text and audio are ready."""
from __future__ import annotations

from datetime import datetime, timedelta
import json
import math
from pathlib import Path
from urllib.parse import urlsplit

from .models import SourceArticle, parse_timestamp
from .pipeline import NoTopicsError, _atomic_json, generate_edition


class DailyFinalizationError(RuntimeError):
    """The database commit succeeded but the local checkpoint could not finish."""


def _complete_audio(edition: dict) -> bool:
    items = edition['payload']['items']
    if not items:
        return False
    for item in items:
        audio = item['audio']
        parts = urlsplit(audio.get('url') or '')
        duration = audio.get('duration_seconds')
        if (audio.get('status') != 'ready' or parts.scheme != 'https' or not parts.hostname
                or parts.username or parts.password or isinstance(duration, bool)
                or not isinstance(duration, (int, float)) or not math.isfinite(duration) or duration <= 0):
            return False
    return True


def run_daily_update(sources: list[SourceArticle], *, repository, generator_factory,
                     tts_factory, target_factory, cache_dir: Path, now: datetime,
                     team_id: int = 77680, limit: int = 5, max_age_hours: int = 72) -> dict:
    if now.tzinfo is None or max_age_hours <= 0 or not 1 <= limit <= 5 or team_id <= 0:
        raise ValueError('invalid daily topic parameters')
    path = cache_dir / 'daily-state.json'
    state = json.loads(path.read_text()) if path.exists() else {'version': 1, 'team_id': team_id, 'pending': None}
    if state.get('version') != 1 or state.get('team_id') != team_id:
        raise ValueError('daily state is incompatible with this team')
    known_urls, watermark = repository.published_sources(team_id)
    cutoff = now - timedelta(hours=max_age_hours)
    pending = state['pending']
    if pending:
        originals = [SourceArticle(item['title'], item['url'], item['publisher'],
                                   parse_timestamp(item['published_at'], now), item['body'], tuple(item.get('images') or []))
                     for item in pending['sources']]
        if not originals or any(item.published_at < cutoff for item in originals) or all(item.url in known_urls for item in originals):
            state['pending'] = pending = None
    if not pending:
        fresh = sorted((item for item in sources if item.url not in known_urls
                        and cutoff <= item.published_at <= now
                        and (watermark is None or item.published_at > watermark)),
                       key=lambda item: (item.published_at, item.url), reverse=True)[:30]
        if not fresh:
            _atomic_json(path, state)
            return {'status': 'no_new', 'collected': len(sources), 'published': 0}
        pending = {'now': now.isoformat(), 'limit': limit,
                   'sources': [item.as_material() for item in fresh]}
        state['pending'] = pending
        _atomic_json(path, state)
    else:
        fresh = originals
    try:
        edition = generate_edition(fresh, generator=generator_factory(), tts=tts_factory(),
            target=target_factory(), cache_dir=cache_dir, now=parse_timestamp(pending['now'], now),
            team_id=team_id, limit=pending['limit'])
    except NoTopicsError:
        state['pending'] = None
        _atomic_json(path, state)
        return {'status': 'no_topics', 'collected': len(sources), 'published': 0}
    if not _complete_audio(edition):
        raise ValueError('new edition audio is incomplete; previous published edition retained')
    repository.save(edition)
    try:
        state['pending'] = None
        _atomic_json(path, state)
        _atomic_json(cache_dir / 'latest.json', edition['payload'])
    except OSError as error:
        raise DailyFinalizationError('database published; local finalization failed') from error
    return {'status': 'published', 'collected': len(sources),
            'published': len(edition['payload']['items']), 'edition_id': edition['id']}

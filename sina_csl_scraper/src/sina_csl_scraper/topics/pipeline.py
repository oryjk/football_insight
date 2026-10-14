"""Publish an atomic article edition; failed audio never discards readable text."""
from __future__ import annotations

import hashlib
import copy
import json
import logging
import re
from datetime import date
from datetime import datetime
from pathlib import Path
from urllib.parse import urlparse

from .generation import GENERATOR_VERSION
from .models import SHANGHAI, SourceArticle, source_fingerprint
from .images import TopicImagePublisher

logger = logging.getLogger(__name__)


class NoTopicsError(ValueError):
    """No publishable material; keep the last successful database edition."""


def _atomic_json(path: Path, value: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_suffix('.tmp')
    temporary.write_text(json.dumps(value, ensure_ascii=False, indent=2), encoding='utf-8')
    temporary.replace(path)


def generate_edition(sources: list[SourceArticle], *, generator, tts, target,
                     cache_dir: Path, now: datetime, team_id: int = 77680,
                     limit: int = 5, tts_unavailable: bool = False) -> dict:
    if not sources:
        raise NoTopicsError('no recent verified article bodies; previous edition retained')
    if not 1 <= limit <= 5 or team_id <= 0 or now.tzinfo is None:
        raise ValueError('invalid team, limit or generation timestamp')
    fingerprint = source_fingerprint(sources)
    date = now.astimezone(SHANGHAI).date().isoformat()
    identity = [team_id, date, fingerprint, GENERATOR_VERSION, generator.model,
                getattr(generator, 'base_url', ''), limit, getattr(generator, 'cache_scope', [])]
    edition_id = hashlib.sha256(json.dumps(identity, ensure_ascii=False).encode()).hexdigest()
    path = cache_dir / 'editions' / f'{edition_id}.json'
    if path.exists():
        edition = json.loads(path.read_text(encoding='utf-8'))
    else:
        items = generator.generate(sources, team_id=team_id, now=now, limit=limit)
        if not items:
            raise NoTopicsError('LLM found no publishable topics; previous edition retained')
        if len(items) > limit:
            raise ValueError('LLM exceeded topic limit')
        edition = {'id': edition_id, 'source_fingerprint': fingerprint,
                   'generator_version': GENERATOR_VERSION + ':' + generator.model,
                   'payload': {'team_id': team_id, 'edition_date': date,
                               'updated_at': now.isoformat(), 'items': items}}
    # Persist the validated text before long-running TTS so interruption can resume.
    selected = getattr(generator, 'selected_team_ids', set())
    if selected and any(not any(tag.get('team_id') in selected for tag in item.get('team_tags', [])) for item in edition['payload']['items']):
        raise ValueError('cached topic does not match the current news scope')
    _atomic_json(path, edition)
    if target is not None and any(source.images for source in sources):
        try:
            TopicImagePublisher(target=target, cache_dir=cache_dir).enrich(edition['payload']['items'], sources)
            _atomic_json(path, edition)
        except Exception as error:
            logger.warning('Photo publication unavailable (%s); text and audio retained', type(error).__name__)
    for item in edition['payload']['items']:
        audio = item['audio']
        if audio['status'] == 'ready':
            continue
        if tts_unavailable:
            item['audio'] = {'status': 'failed', 'url': None, 'dialect': 'sichuan', 'duration_seconds': None}
            continue
        if tts is None:
            continue
        try:
            narration = item['title'].rstrip('。') + '。\n' + '\n\n'.join(item['paragraphs'])
            generated = tts.synthesize(narration)
            if target is None:
                # Verified files remain in the local TTS cache until upload is configured.
                item['audio'] = {'status': 'pending', 'url': None, 'dialect': 'sichuan', 'duration_seconds': None}
                continue
            object_name = f'summary/audio/topics/{team_id}/{date}/{item["id"]}/{generated.sha256}.m4a'
            url = target.upload_bytes(object_name, generated.path.read_bytes(), 'audio/mp4')
            parsed = urlparse(url)
            if parsed.scheme != 'https' or not parsed.netloc or generated.duration_seconds <= 0:
                raise ValueError('audio publication requires a valid HTTPS URL and duration')
            item['audio'] = {'status': 'ready', 'url': url, 'dialect': 'sichuan',
                             'duration_seconds': round(generated.duration_seconds, 3)}
        except Exception as error:
            # Never log provider responses/commands that might contain credentials.
            logger.warning('topic audio failed (%s); text retained', type(error).__name__)
            item['audio'] = {'status': 'failed', 'url': None, 'dialect': 'sichuan', 'duration_seconds': None}
        finally:
            _atomic_json(path, edition)
    _atomic_json(path, edition)
    return edition


def refresh_edition_audio(edition: dict, *, tts, target) -> dict:
    """Reuse verified article text and replace audio only after every item succeeds.

    Never mutate the caller's already-published edition on partial native/upload
    failure. The caller can then perform one atomic repository upsert.
    """
    updated = copy.deepcopy(edition)
    payload = updated['payload']
    team_id, edition_date, items = payload['team_id'], payload['edition_date'], payload['items']
    if isinstance(team_id, bool) or not isinstance(team_id, int) or team_id <= 0 or not 1 <= len(items) <= 5:
        raise ValueError('audio refresh requires a nonempty valid topic edition')
    date.fromisoformat(edition_date)
    if not re.fullmatch(r'[0-9a-f]{64}', updated['id']):
        raise ValueError('invalid edition identity')
    for item in items:
        if item['team_id'] != team_id or not re.fullmatch(r'[0-9a-f]{64}', item['id']):
            raise ValueError('invalid article identity or team')
        if not isinstance(item['title'], str) or not item['paragraphs'] or not all(isinstance(p, str) and p.strip() for p in item['paragraphs']):
            raise ValueError('audio refresh requires existing article text')
    for item in items:
        narration = item['title'].rstrip('。') + '。\n' + '\n\n'.join(item['paragraphs'])
        generated = tts.synthesize(narration)
        object_name = f'summary/audio/topics/{team_id}/{edition_date}/{item["id"]}/{generated.sha256}.m4a'
        url = target.upload_bytes(object_name, generated.path.read_bytes(), 'audio/mp4')
        parsed = urlparse(url)
        if parsed.scheme != 'https' or not parsed.netloc or parsed.username or parsed.password or generated.duration_seconds <= 0:
            raise ValueError('audio publication requires a public HTTPS URL and duration')
        item['audio'] = {'status': 'ready', 'url': url, 'dialect': 'sichuan',
                         'duration_seconds': round(generated.duration_seconds, 3)}
    return updated

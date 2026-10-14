"""Only writes the table provisioned by the Rust migration, never creates tables."""
from __future__ import annotations

from datetime import datetime

import psycopg
from psycopg.types.json import Jsonb
from .models import canonical_url


class TopicRepository:
    def __init__(self, database_url: str):
        self.database_url = database_url

    def check_schema(self) -> None:
        with psycopg.connect(self.database_url) as connection:
            with connection.cursor() as cursor:
                cursor.execute("SELECT to_regclass('public.f_i_daily_topic_editions')")
                if cursor.fetchone()[0] is None:
                    raise ValueError('apply the Rust f_i_daily_topic_editions migration before --write-db')

    def save(self, edition: dict) -> None:
        payload = edition['payload']
        if not payload['items'] or len(payload['items']) > 5:
            raise ValueError('only nonempty editions with at most five topics can be published')
        with psycopg.connect(self.database_url) as connection:
            with connection.cursor() as cursor:
                cursor.execute('''
                    INSERT INTO f_i_daily_topic_editions
                      (id, team_id, edition_date, published_at, payload, source_fingerprint, generator_version)
                    VALUES (%s, %s, %s, %s, %s, %s, %s)
                    ON CONFLICT (id) DO UPDATE SET payload = EXCLUDED.payload
                ''', (edition['id'], payload['team_id'], payload['edition_date'], payload['updated_at'],
                      Jsonb(payload), edition['source_fingerprint'], edition['generator_version']))

    def published_sources(self, team_id: int) -> tuple[set[str], datetime | None]:
        """Use publication history as the durable deduplication baseline."""
        with psycopg.connect(self.database_url) as connection:
            with connection.cursor() as cursor:
                cursor.execute('''
                    SELECT DISTINCT source->>'url', source->>'published_at'
                    FROM f_i_daily_topic_editions
                    CROSS JOIN LATERAL jsonb_array_elements(payload->'items') AS item
                    CROSS JOIN LATERAL jsonb_array_elements(item->'sources') AS source
                    WHERE team_id = %s
                ''', (team_id,))
                rows = cursor.fetchall()
        urls = {canonical_url(url) for url, _ in rows}
        # Keep the tuple contract for older callers, but never derive a news
        # watermark from publication-history clocks (which may be query snapshots).
        return urls, None

    def published_items(self, team_id: int) -> list[dict]:
        with psycopg.connect(self.database_url) as connection:
            rows = connection.execute('''SELECT payload FROM f_i_daily_topic_editions
                WHERE team_id = %s ORDER BY published_at DESC, id DESC''', (team_id,)).fetchall()
        items = {}
        for (payload,) in rows:
            for item in payload['items']:
                items.setdefault(item['id'], item)
        return list(items.values())

    def patch_images(self, team_id: int, updates: list[dict]) -> int:
        """Lock current rows and change only images, including historical copies."""
        by_id = {item['id']: item for item in updates if item.get('images')}
        changed = 0
        with psycopg.connect(self.database_url) as connection:
            rows = connection.execute('''SELECT id, payload FROM f_i_daily_topic_editions
                WHERE team_id = %s ORDER BY id FOR UPDATE''', (team_id,)).fetchall()
            for edition_id, payload in rows:
                dirty = False
                for item in payload['items']:
                    update = by_id.get(item['id'])
                    if update and item['sources'] == update['sources'] and item.get('images') != update['images']:
                        item['images'] = update['images']
                        changed += 1
                        dirty = True
                if dirty:
                    connection.execute('UPDATE f_i_daily_topic_editions SET payload = %s WHERE id = %s',
                                       (Jsonb(payload), edition_id))
        return changed

    def patch_tags(self, team_id: int, updates: list[dict]) -> int:
        """Patch only tags under row locks; retain concurrent audio/images and metrics."""
        by_id = {item['id']: item for item in updates}
        changed = 0
        with psycopg.connect(self.database_url) as connection:
            rows = connection.execute('SELECT id, payload FROM f_i_daily_topic_editions WHERE team_id=%s ORDER BY id FOR UPDATE', (team_id,)).fetchall()
            for edition_id, payload in rows:
                dirty = False
                for item in payload['items']:
                    update = by_id.get(item['id'])
                    if update and all(item.get(key) == update.get(key) for key in ('title', 'summary', 'paragraphs')) and item.get('team_tags') != update['team_tags']:
                        item['team_tags'] = update['team_tags']
                        changed += 1
                        dirty = True
                if dirty:
                    connection.execute('UPDATE f_i_daily_topic_editions SET payload=%s WHERE id=%s', (Jsonb(payload), edition_id))
        return changed

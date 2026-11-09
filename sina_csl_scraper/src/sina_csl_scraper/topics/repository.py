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

    def roundup_for_date(self, team_id: int, edition_date: str) -> dict | None:
        """当天这一期每日电报（一天一行，ID 由日期决定），没有则 None。"""
        from .roundup import roundup_edition_id
        edition_id = roundup_edition_id(team_id, edition_date)
        with psycopg.connect(self.database_url) as connection:
            row = connection.execute('''SELECT id, source_fingerprint, generator_version, payload
                FROM f_i_daily_topic_editions WHERE id = %s AND team_id = %s''', (edition_id, team_id)).fetchone()
        if row is None:
            return None
        return {'id': row[0], 'source_fingerprint': row[1], 'generator_version': row[2], 'payload': row[3]}

    def save_roundup(self, edition: dict) -> None:
        """原地更新当天这一行；不删除任何其他批次，保留历史和阅读统计。"""
        payload = edition['payload']
        items = payload['items']
        if len(items) != 1 or not items[0].get('entries'):
            raise ValueError('a daily roundup edition holds exactly one item with entries')
        with psycopg.connect(self.database_url) as connection:
            connection.execute('''
                INSERT INTO f_i_daily_topic_editions
                  (id, team_id, edition_date, published_at, payload, source_fingerprint, generator_version)
                VALUES (%s, %s, %s, %s, %s, %s, %s)
                ON CONFLICT (id) DO UPDATE SET
                  published_at = EXCLUDED.published_at,
                  payload = EXCLUDED.payload,
                  source_fingerprint = EXCLUDED.source_fingerprint,
                  generator_version = EXCLUDED.generator_version
            ''', (edition['id'], payload['team_id'], payload['edition_date'], payload['updated_at'],
                  Jsonb(payload), edition['source_fingerprint'], edition['generator_version']))

    def roundup_sources(self, team_id: int) -> set[str]:
        """每日电报用过的原文。旧版多篇话题不参与：它们不再生成，也不该挡住当天快讯。"""
        with psycopg.connect(self.database_url) as connection:
            rows = connection.execute('''
                SELECT DISTINCT source->>'url'
                FROM f_i_daily_topic_editions
                CROSS JOIN LATERAL jsonb_array_elements(payload->'items') AS item
                CROSS JOIN LATERAL jsonb_array_elements(item->'sources') AS source
                WHERE team_id = %s AND generator_version LIKE 'daily-roundup-%%'
            ''', (team_id,)).fetchall()
        return {canonical_url(url) for (url,) in rows if url}

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

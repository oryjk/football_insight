"""Only writes the table provisioned by the Rust migration, never creates tables."""
from __future__ import annotations

from datetime import datetime

import psycopg
from psycopg.types.json import Jsonb
from .models import canonical_url, parse_timestamp


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
        dates = [parse_timestamp(timestamp) for _, timestamp in rows if timestamp]
        return urls, max(dates) if dates else None

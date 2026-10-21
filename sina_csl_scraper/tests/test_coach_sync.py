import io
import os
import uuid
from pathlib import Path
from urllib.parse import urlsplit

import psycopg
import pytest
from PIL import Image

from sina_csl_scraper.client import parse_team_coach
from sina_csl_scraper.coach_portraits import COACH_AVATAR_OVERRIDES, CoachPortrait
from sina_csl_scraper.coach_sync import (
    PostgresCoachStore,
    coach_avatar_object_name,
    crop_portrait,
    make_coach_avatar_uploader,
    sync_coaches,
)
from sina_csl_scraper.models import TeamCoach

MIGRATIONS = Path(__file__).parents[2] / 'football_insight_service_backend_rs/migrations'


def test_parses_active_manager_before_season_start_manager():
    data = {'manager': 'Old Coach', 'manager_cn': '旧帅', 'active_manager': 'John Aloisi', 'active_manager_cn': '约翰·阿洛伊西'}
    assert parse_team_coach(77680, data) == TeamCoach(77680, 'John Aloisi', '约翰·阿洛伊西')
    assert parse_team_coach(1, {'manager': 'Only Old', 'manager_cn': '', 'active_manager': ' ', 'active_manager_cn': ''}) == \
        TeamCoach(1, 'Only Old', 'Only Old')
    assert parse_team_coach(1, {'active_manager_cn': '只有中文'}) == TeamCoach(1, '只有中文', '只有中文')
    assert parse_team_coach(1, {'manager': '', 'assistant': 'X'}) is None


def test_reviewed_portraits_are_versioned_jpegs_with_valid_crops():
    names = [p.filename for p in COACH_AVATAR_OVERRIDES.values()]
    assert len(names) == len(set(names))
    for portrait in COACH_AVATAR_OVERRIDES.values():
        assert portrait.filename.startswith('coach-') and portrait.filename.endswith('.jpg')
        assert portrait.source_url.startswith('https://')
        if portrait.crop:
            left, top, right, bottom = portrait.crop
            assert right - left == bottom - top > 0, portrait


class FakeStore:
    def __init__(self, stored=None):
        self.applied, self.avatars, self.stored = [], [], stored or {}

    def team_ids(self):
        return [1, 2, 3, 4]

    def apply(self, coach):
        self.applied.append(coach)
        return 9_000_000_000 + coach.team_id, self.stored.get(coach.coach_key)

    def set_avatar(self, coach_id, source_url, object_name, storage_url):
        self.avatars.append((coach_id, object_name))


def test_sync_isolates_team_failures_and_uploads_only_new_portraits():
    portraits = {'A': CoachPortrait('https://x/a.jpg', 'coach-a.jpg'), 'B': CoachPortrait('https://x/b.jpg', 'coach-b.jpg')}

    def fetch(team_id):
        if team_id == 3:
            raise RuntimeError('boom')
        return {1: TeamCoach(1, 'A', '甲'), 2: TeamCoach(2, 'B', '乙'), 4: None}[team_id]

    uploads = []

    def upload(portrait):
        uploads.append(portrait.filename)
        return portrait.source_url, coach_avatar_object_name('summary', portrait), 'https://cdn/' + portrait.filename

    # B 已是同一版本的头像：不重复上传。
    store = FakeStore(stored={'B': 'summary/players/coach-b.jpg'})
    summary = sync_coaches(fetch=fetch, store=store, prefix='summary', avatars=upload, portraits=portraits)
    assert [c.coach_key for c in store.applied] == ['A', 'B']
    assert uploads == ['coach-a.jpg']
    assert store.avatars == [(9_000_000_001, 'summary/players/coach-a.jpg')]
    assert summary == {'coaches': 2, 'teams_without_coach': [4], 'teams_failed': [3], 'coach_avatars_uploaded': 1}

    # 不上传头像时只写教练。
    store = FakeStore()
    assert sync_coaches(fetch=fetch, store=store, prefix='summary', portraits=portraits)['coach_avatars_uploaded'] == 0
    assert store.avatars == []


def test_avatar_failure_keeps_coach():
    store = FakeStore()

    def upload(_):
        raise OSError('cdn down')

    summary = sync_coaches(fetch=lambda t: TeamCoach(t, f'C{t}', f'教{t}'), store=store, prefix='p', avatars=upload,
                           portraits={'C1': CoachPortrait('https://x', 'coach-c1.jpg')})
    assert summary['coaches'] == 4 and summary['coach_avatars_uploaded'] == 0 and store.avatars == []


def jpeg(width, height):
    out = io.BytesIO()
    Image.new('RGB', (width, height), (200, 30, 30)).save(out, 'JPEG')
    return out.getvalue()


def test_uploader_crops_posters_and_writes_thumbnails():
    class Fetcher:
        def fetch(self, url):
            return jpeg(400, 600), 'image/jpeg'

    class Target:
        def __init__(self):
            self.objects = {}

        def upload_bytes(self, name, content, content_type, cache_control=None):
            self.objects[name] = content
            return f'https://cdn/{name}'

    target = Target()
    upload = make_coach_avatar_uploader(Fetcher(), target, '/summary/')
    source, name, url = upload(CoachPortrait('https://x/p.jpeg', 'coach-x.jpg', (50, 60, 250, 260)))
    assert (source, name, url) == ('https://x/p.jpeg', 'summary/players/coach-x.jpg', 'https://cdn/summary/players/coach-x.jpg')
    assert Image.open(io.BytesIO(target.objects[name])).size == (200, 200)
    assert sorted(target.objects) == ['summary/players/coach-x.jpg', 'summary/players/thumbs/240/coach-x.jpg.jpg',
                                      'summary/players/thumbs/96/coach-x.jpg.jpg']
    assert Image.open(io.BytesIO(crop_portrait(jpeg(10, 10), (0, 0, 4, 4)))).size == (4, 4)


@pytest.fixture
def pg_store():
    url = os.getenv('SQUAD_TEST_DATABASE_URL')
    if not url:
        pytest.skip('explicit disposable localhost _test database required')
    parsed = urlsplit(url)
    assert parsed.hostname in {'localhost', '127.0.0.1'} and parsed.path.endswith('_test')
    schema = f'coach_{uuid.uuid4().hex}'
    with psycopg.connect(url, autocommit=True) as c:
        c.execute(f'CREATE SCHEMA {schema}')
        try:
            c.execute(f'SET search_path TO {schema}')
            c.execute('CREATE TABLE f_i_teams (team_id BIGINT PRIMARY KEY)')
            c.execute('INSERT INTO f_i_teams VALUES (1), (2), (3)')
            c.execute((MIGRATIONS / '20261019120000_add_squad_players.sql').read_text())
            c.execute((MIGRATIONS / '20261021120000_add_team_coaches.sql').read_text())
            c.execute("""INSERT INTO f_i_squad_players (team_id, player_id, player_name, last_season, active)
                         VALUES (1, 1, 'a', 2026, true), (2, 2, 'b', 2026, true), (3, 3, 'c', 2025, false)""")
            yield PostgresCoachStore(c)
        finally:
            c.execute(f'DROP SCHEMA {schema} CASCADE')


def coaches(store):
    return store.connection.execute(
        'SELECT coach_id, coach_key, team_id, active FROM f_i_team_coaches ORDER BY coach_id').fetchall()


def test_postgres_store_keeps_one_row_per_coach_across_changes(pg_store):
    assert pg_store.team_ids() == [1, 2]
    first, _ = pg_store.apply(TeamCoach(1, 'A', '甲'))
    assert first == 9_000_000_001
    assert pg_store.apply(TeamCoach(1, 'A', '甲')) == (first, None)
    pg_store.set_avatar(first, 'https://src', 'summary/players/coach-a.jpg', 'https://cdn/a')
    assert pg_store.apply(TeamCoach(1, 'A', '甲')) == (first, 'summary/players/coach-a.jpg')
    # 1 队换帅：A 下课，B 上任。
    second, _ = pg_store.apply(TeamCoach(1, 'B', '乙'))
    assert coaches(pg_store) == [(first, 'A', 1, False), (second, 'B', 1, True)]
    # A 去 2 队执教：同一行改队并恢复在任，编号不变。
    assert pg_store.apply(TeamCoach(2, 'A', '甲'))[0] == first
    assert coaches(pg_store) == [(first, 'A', 2, True), (second, 'B', 1, True)]

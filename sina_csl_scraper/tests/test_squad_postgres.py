import os
from pathlib import Path
from urllib.parse import urlsplit

import psycopg
import pytest

from sina_csl_scraper.models import SquadPlayer
from sina_csl_scraper.squad_sync import PostgresSquadStore, build_squad_rows


@pytest.fixture
def store():
    url = os.getenv('SQUAD_TEST_DATABASE_URL')
    if not url:
        pytest.skip('explicit disposable localhost _test database required')
    parsed = urlsplit(url)
    assert parsed.hostname in {'localhost', '127.0.0.1'} and parsed.path.endswith('_test')
    with psycopg.connect(url, autocommit=True) as c:
        c.execute('SET search_path TO pg_temp')
        # 旧的头像同步写入的球员表（只用到这两列）
        c.execute('CREATE TEMP TABLE f_i_players(player_id bigint, avatar_storage_url text)')
        migration = Path(__file__).parents[2] / 'football_insight_service_backend_rs/migrations/20261019120000_add_squad_players.sql'
        c.execute(migration.read_text())
        yield PostgresSquadStore(c)


def p(pid, team=1, number=None, position='MF', name=None):
    return SquadPlayer(team, pid, name or f'P{pid}', number, position, None, None)


def rows(store, team=1):
    return store.connection.execute(
        'SELECT player_id, shirt_number, last_season, active FROM f_i_squad_players WHERE team_id=%s ORDER BY player_id',
        (team,)).fetchall()


def test_current_marks_missing_players_as_former(store):
    store.apply(1, 2026, build_squad_rows([p(1), p(2)], 2026, {}), current=True)
    store.apply(1, 2026, build_squad_rows([p(1)], 2026, {}), current=True)
    assert rows(store) == [(1, None, 2026, True), (2, None, 2026, False)]


def test_backfill_adds_former_players_without_touching_current(store):
    store.apply(1, 2026, build_squad_rows([p(1, number=6)], 2026, {}), current=True)
    store.apply(1, 2024, build_squad_rows([p(1, number=30), p(9, number=7)], 2024, {}), current=False)
    assert rows(store) == [(1, 6, 2026, True), (9, 7, 2024, False)]


def test_backfill_moves_last_season_forward_only(store):
    store.apply(1, 2024, build_squad_rows([p(9, number=7)], 2024, {}), current=False)
    store.apply(1, 2025, build_squad_rows([p(9, number=10)], 2025, {}), current=False)
    store.apply(1, 2023, build_squad_rows([p(9, number=1)], 2023, {}), current=False)
    assert rows(store) == [(9, 10, 2025, False)]


def test_same_player_in_two_teams_keeps_two_rows(store):
    store.apply(1, 2026, build_squad_rows([p(5, team=1)], 2026, {}), current=True)
    store.apply(2, 2026, build_squad_rows([p(5, team=2)], 2026, {}), current=True)
    assert rows(store, 1) == [(5, None, 2026, True)]
    assert rows(store, 2) == [(5, None, 2026, True)]


def test_existing_avatar_is_kept_when_new_row_has_none(store):
    store.apply(1, 2026, build_squad_rows([p(1)], 2026, {1: 'https://minio/1.png'}), current=True)
    store.apply(1, 2026, build_squad_rows([p(1)], 2026, {}), current=True)
    assert store.avatar_urls(1) == {1: 'https://minio/1.png'}


def test_empty_apply_is_refused(store):
    with pytest.raises(ValueError):
        store.apply(1, 2026, [], current=True)


def test_avatar_urls_reuse_existing_player_and_other_team_avatars(store):
    # 旧头像同步已把 pid 3 传到 MinIO；pid 5 在另一支队的名单里已有头像（转会球员）。
    store.connection.execute("INSERT INTO f_i_players VALUES (3, 'https://minio/3.png')")
    store.apply(2, 2026, build_squad_rows([p(5, team=2)], 2026, {5: 'https://minio/5.png'}), current=True)
    urls = store.avatar_urls(1)
    assert urls[3] == 'https://minio/3.png'
    assert urls[5] == 'https://minio/5.png'


def test_active_count_counts_only_current_players_of_that_team(store):
    store.apply(1, 2026, build_squad_rows([p(1), p(2), p(3)], 2026, {}), current=True)
    store.apply(1, 2025, build_squad_rows([p(9)], 2025, {}), current=False)
    store.apply(2, 2026, build_squad_rows([p(7, team=2)], 2026, {}), current=True)
    assert store.active_count(1) == 3
    assert store.active_count(2) == 1
    assert store.active_count(99) == 0

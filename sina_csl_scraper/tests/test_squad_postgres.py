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

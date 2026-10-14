import os
from pathlib import Path
from urllib.parse import urlsplit

import psycopg
import pytest

from sina_csl_scraper.player_goal_sync import replace_player_logs
from sina_csl_scraper.player_match_goals import PlayerMatchGoals


@pytest.fixture
def connection():
    url = os.getenv('PLAYER_GOALS_TEST_DATABASE_URL')
    if not url:
        pytest.skip('explicit disposable localhost _test database required')
    parsed = urlsplit(url)
    assert parsed.hostname in {'localhost', '127.0.0.1'} and parsed.path.endswith('_test')
    with psycopg.connect(url, autocommit=True) as c:
        c.execute('SET search_path TO pg_temp')
        c.execute('CREATE TEMP TABLE f_i_matches(match_id bigint PRIMARY KEY)')
        migration = Path(__file__).parents[2] / 'football_insight_service_backend_rs/migrations/20261014163000_add_player_match_stats.sql'
        c.execute(migration.read_text())
        assert c.execute("SELECT relnamespace=pg_my_temp_schema() FROM pg_class WHERE oid='f_i_player_match_stats'::regclass").fetchone()[0]
        c.execute('INSERT INTO f_i_matches VALUES (1),(2)')
        yield c


def record(goals=0):
    return PlayerMatchGoals(10, 1, 2026, goals, 90, 'player', 'match')


def test_migration_and_idempotent_replacement_preserve_actual_zero_and_hat_trick(connection):
    replace_player_logs(connection, season=2026, player_id=10, records=[record()])
    replace_player_logs(connection, season=2026, player_id=10, records=[record(3)])
    assert connection.execute('SELECT match_id,goals FROM f_i_player_match_stats').fetchall() == [(1, 3)]


def test_failed_replacement_rolls_back_delete_and_other_players_remain_untouched(connection):
    replace_player_logs(connection, season=2026, player_id=10, records=[record(3)])
    other = PlayerMatchGoals(20, 2, 2026, 0, 10, 'other', 'other-match')
    replace_player_logs(connection, season=2026, player_id=20, records=[other])
    with pytest.raises(psycopg.errors.CheckViolation):
        replace_player_logs(connection, season=2026, player_id=10, records=[record(-1)])
    assert connection.execute('SELECT player_id,goals FROM f_i_player_match_stats ORDER BY player_id').fetchall() == [(10, 3), (20, 0)]

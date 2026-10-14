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


def test_refresh_includes_scorers_beyond_home_top_five(connection, monkeypatch):
    from contextlib import nullcontext
    from sina_csl_scraper import player_goal_sync

    connection.execute("""
        ALTER TABLE f_i_matches
            ADD season int DEFAULT 2026, ADD round_number int DEFAULT 1,
            ADD round_name text DEFAULT '第1轮', ADD match_date date DEFAULT '2026-03-07',
            ADD match_time text DEFAULT '19:35', ADD status text DEFAULT '3',
            ADD home_team_id bigint DEFAULT 500, ADD home_team_name text DEFAULT '云南玉昆',
            ADD home_score text DEFAULT '3', ADD away_team_id bigint DEFAULT 143,
            ADD away_team_name text DEFAULT '青岛海牛', ADD away_score text DEFAULT '1',
            ADD source_active bool DEFAULT TRUE;
        UPDATE f_i_matches SET match_date='2026-03-08' WHERE match_id=2;
        CREATE TEMP TABLE f_i_ranking_categories(id int,scope text,slug text);
        CREATE TEMP TABLE f_i_player_ranking_snapshots(id int,category_id int,season int,snapshot_kind text,snapshot_at timestamptz);
        CREATE TEMP TABLE f_i_player_ranking_entries(snapshot_id int,player_id bigint,player_name text,team_id bigint,score_value text,rank_no int);
        INSERT INTO f_i_ranking_categories VALUES (1,'player','goals');
        INSERT INTO f_i_player_ranking_snapshots VALUES (1,1,2026,'live',NOW());
        INSERT INTO f_i_player_ranking_entries
            SELECT 1,n,'球员' || n,500,CASE WHEN n<=5 THEN '3' ELSE '1' END,n
            FROM generate_series(1,20) n;
    """)

    class LogClient:
        def fetch_player_logs(self, *, name, team_id, season):
            n = int(name.removeprefix('球员'))
            identity = f'player-{n}'
            return identity, [{
                'match_id': 'official-match', 'player_id': identity,
                'competition_code': 'CSL', 'tournament_calendar_name': '2026',
                'local_date': '2026-03-07', 'local_time': '19:35:00',
                'home_contestant_id': '3n4jrcw6w965v1qvp0azx5kwk',
                'away_contestant_id': 'a3rgv4o76uowd2gxzx9pgf02d',
                'minutes_played': 90, 'goals': 3 if n<=5 else 1,
            }]

    monkeypatch.setattr(player_goal_sync.psycopg, 'connect', lambda *args, **kwargs: nullcontext(connection))
    result = player_goal_sync.refresh_scorer_goals('isolated-fixture', 2026, client=LogClient())
    assert result == {'players_updated': 20, 'appearances': 20, 'failed_players': []}
    assert connection.execute('SELECT COUNT(*) FROM f_i_player_match_stats').fetchone()[0] == 20

import json

from typer.testing import CliRunner

from sina_csl_scraper import cli


def test_sync_squads_requires_database_url(monkeypatch):
    monkeypatch.delenv('FI_DATABASE_URL', raising=False)
    monkeypatch.delenv('DATABASE_URL', raising=False)
    result = CliRunner().invoke(cli.app, ['sync-squads'])
    assert result.exit_code != 0


def test_sync_squads_uses_league_season_and_prints_summary(monkeypatch):
    captured = {}

    class FakeClient:
        def fetch_league_info(self):
            return type('L', (), {'current_season': 2026})()

        def fetch_team_squad(self, team_id, season=None):
            return []

        def fetch_team_coach(self, team_id):
            return None

    class FakeConn:
        def __enter__(self):
            return self

        def __exit__(self, *a):
            return False

    def fake_sync(**kwargs):
        captured.update(kwargs)
        return {'teams_ok': 16, 'teams_failed': [], 'players_current': 500, 'avatars_uploaded': 0}

    monkeypatch.setattr(cli, 'SinaCslClient', lambda: FakeClient())
    monkeypatch.setattr(cli.psycopg, 'connect', lambda *a, **k: FakeConn())
    monkeypatch.setattr(cli, 'sync_squads', fake_sync)
    coach_calls = {}

    def fake_coaches(**kwargs):
        coach_calls.update(kwargs)
        return {'coaches': 16, 'teams_without_coach': [], 'teams_failed': [], 'coach_avatars_uploaded': 0}

    monkeypatch.setattr(cli, 'sync_coaches', fake_coaches)
    result = CliRunner().invoke(cli.app, ['sync-squads', '--database-url', 'postgresql://x/y', '--backfill-seasons', '3'])
    assert result.exit_code == 0, result.output
    assert captured['season'] == 2026 and captured['backfill_seasons'] == 3 and captured['avatars'] is None
    assert coach_calls['avatars'] is None and coach_calls['prefix'] == 'summary'
    output = json.loads(result.output.strip().splitlines()[-1])
    assert output['teams_ok'] == 16 and output['coaches'] == 16


def test_coach_failure_does_not_fail_squad_sync(monkeypatch):
    class FakeClient:
        def fetch_league_info(self):
            return type('L', (), {'current_season': 2026})()

        def fetch_team_coach(self, team_id):
            return None

    class FakeConn:
        def __enter__(self):
            return self

        def __exit__(self, *a):
            return False

    def broken_coaches(**kwargs):
        raise RuntimeError('relation "f_i_team_coaches" does not exist')

    monkeypatch.setattr(cli, 'SinaCslClient', lambda: FakeClient())
    monkeypatch.setattr(cli.psycopg, 'connect', lambda *a, **k: FakeConn())
    monkeypatch.setattr(cli, 'sync_squads', lambda **k: {'teams_ok': 16})
    monkeypatch.setattr(cli, 'sync_coaches', broken_coaches)
    result = CliRunner().invoke(cli.app, ['sync-squads', '--database-url', 'postgresql://x/y'])
    assert result.exit_code == 0, result.output
    assert json.loads(result.stdout.strip().splitlines()[-1]) == {'season': 2026, 'teams_ok': 16, 'coaches_error': 'RuntimeError'}

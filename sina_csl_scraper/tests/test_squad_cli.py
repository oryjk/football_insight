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
    result = CliRunner().invoke(cli.app, ['sync-squads', '--database-url', 'postgresql://x/y', '--backfill-seasons', '3'])
    assert result.exit_code == 0, result.output
    assert captured['season'] == 2026 and captured['backfill_seasons'] == 3 and captured['avatars'] is None
    assert json.loads(result.output.strip().splitlines()[-1])['teams_ok'] == 16

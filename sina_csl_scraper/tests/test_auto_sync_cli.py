from __future__ import annotations

from datetime import datetime
from pathlib import Path

from typer.testing import CliRunner

from sina_csl_scraper.auto_sync import AutoSyncDecision, AutoSyncState
from sina_csl_scraper.cli import app
from sina_csl_scraper.models import LeagueInfo


class FakeSinaClient:
    def __init__(self, league_id: int = 213) -> None:
        self.league_id = league_id

    def fetch_league_info(self) -> LeagueInfo:
        return LeagueInfo(
            lid=self.league_id,
            name="中超",
            current_season=2026,
            current_round=7,
            max_round=30,
        )

    def fetch_all_matches(self, season: int, max_round: int | None = None) -> list[object]:
        return []


def test_auto_sync_due_logs_completed_due_window_with_explicit_name(monkeypatch, tmp_path: Path) -> None:
    monkeypatch.setattr("sina_csl_scraper.cli.SinaCslClient", FakeSinaClient)
    monkeypatch.setattr(
        "sina_csl_scraper.cli.load_auto_sync_state",
        lambda path: AutoSyncState(
            last_processed_due_at=datetime.fromisoformat("2026-04-18T21:45:00+08:00"),
        ),
    )
    monkeypatch.setattr(
        "sina_csl_scraper.cli.build_auto_sync_decision",
        lambda *args, **kwargs: AutoSyncDecision(
            should_run=True,
            latest_due_at=datetime.fromisoformat("2026-04-18T22:10:00+08:00"),
            newly_due_match_ids=(288619,),
            active_match_ids=(),
        ),
    )

    runner = CliRunner()
    result = runner.invoke(
        app,
        [
            "auto-sync-due",
            "--state-file",
            str(tmp_path / ".auto_sync_state.json"),
            "--dry-run",
        ],
    )

    assert result.exit_code == 0
    assert "completed_due_match_ids=288619" in result.stdout
    assert "latest_completed_due_at=2026-04-18T22:10:00+08:00" in result.stdout
    assert "latest_due_at=" not in result.stdout


def test_auto_sync_due_logs_active_refresh_without_ambiguous_due_name(monkeypatch, tmp_path: Path) -> None:
    monkeypatch.setattr("sina_csl_scraper.cli.SinaCslClient", FakeSinaClient)
    monkeypatch.setattr(
        "sina_csl_scraper.cli.load_auto_sync_state",
        lambda path: AutoSyncState(
            last_processed_due_at=datetime.fromisoformat("2026-04-18T22:10:00+08:00"),
            last_run_at=datetime.fromisoformat("2026-04-21T20:23:00+08:00"),
        ),
    )
    monkeypatch.setattr(
        "sina_csl_scraper.cli.build_auto_sync_decision",
        lambda *args, **kwargs: AutoSyncDecision(
            should_run=True,
            latest_due_at=datetime.fromisoformat("2026-04-18T22:10:00+08:00"),
            newly_due_match_ids=(),
            active_match_ids=(288620, 288621),
        ),
    )

    runner = CliRunner()
    result = runner.invoke(
        app,
        [
            "auto-sync-due",
            "--state-file",
            str(tmp_path / ".auto_sync_state.json"),
            "--dry-run",
        ],
    )

    assert result.exit_code == 0
    assert "active_match_ids=288620,288621" in result.stdout
    assert "latest_completed_due_at=2026-04-18T22:10:00+08:00" in result.stdout
    assert "latest_due_at=" not in result.stdout

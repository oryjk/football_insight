from __future__ import annotations

from datetime import datetime
from pathlib import Path

from typer.testing import CliRunner

from sina_csl_scraper.auto_sync import AutoSyncDecision, AutoSyncState
from sina_csl_scraper.cli import app
from sina_csl_scraper.models import LeagueInfo, MatchResult


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

    def fetch_round_matches(self, season: int, round_number: int) -> list[MatchResult]:
        return [
            MatchResult(
                match_id=round_number * 100 + item,
                season=season,
                round_number=round_number,
                round_name=f"第{round_number}轮",
                date="2026-03-01",
                time="19:35",
                status="1",
                home_team_id=round_number * 1000 + item * 2,
                home_team_name=f"主队{round_number}-{item}",
                home_score="",
                away_team_id=round_number * 1000 + item * 2 + 1,
                away_team_name=f"客队{round_number}-{item}",
                away_score="",
                home_logo="",
                away_logo="",
            )
            for item in range(1, 9)
        ]


def test_auto_sync_due_uses_reconciled_schedule(monkeypatch, tmp_path: Path) -> None:
    captured: dict[str, object] = {}

    class DirectFetchMustNotBeUsed(FakeSinaClient):
        def fetch_all_matches(self, season: int, max_round: int | None = None) -> list[object]:
            raise AssertionError("auto sync must use the reconciled schedule")

    def fake_fetch_reconciled_schedule(**kwargs: object) -> list[object]:
        captured.update(kwargs)
        return []

    monkeypatch.setattr("sina_csl_scraper.cli.SinaCslClient", DirectFetchMustNotBeUsed)
    monkeypatch.setattr(
        "sina_csl_scraper.cli.fetch_reconciled_schedule",
        fake_fetch_reconciled_schedule,
        raising=False,
    )
    monkeypatch.setattr(
        "sina_csl_scraper.cli.build_auto_sync_decision",
        lambda *args, **kwargs: AutoSyncDecision(
            should_run=False,
            latest_due_at=None,
            newly_due_match_ids=(),
        ),
    )

    result = CliRunner().invoke(
        app,
        ["auto-sync-due", "--state-file", str(tmp_path / ".auto_sync_state.json"), "--dry-run"],
    )

    assert result.exit_code == 0
    assert captured["season"] == 2026
    assert captured["max_round"] == 30


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


def test_auto_sync_due_limits_corner_enrichment_to_triggered_match_ids(monkeypatch, tmp_path: Path) -> None:
    captured: dict[str, object] = {}

    monkeypatch.setattr("sina_csl_scraper.cli.SinaCslClient", FakeSinaClient)
    monkeypatch.setattr("sina_csl_scraper.cli.save_auto_sync_state", lambda *args, **kwargs: None)
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
            active_match_ids=(288620,),
        ),
    )

    def fake_run_scrape(**kwargs: object) -> dict[str, object]:
        captured.update(kwargs)
        return {"season": 2026, "run_id": None, "matches": 0}

    monkeypatch.setattr("sina_csl_scraper.cli.run_scrape", fake_run_scrape)

    runner = CliRunner()
    result = runner.invoke(
        app,
        [
            "auto-sync-due",
            "--state-file",
            str(tmp_path / ".auto_sync_state.json"),
            "--enrich-corners",
        ],
    )

    assert result.exit_code == 0
    assert captured["enrich_corners"] is True
    assert captured["enrich_match_ids"] == {288619, 288620}


def test_scrape_command_keeps_manual_corner_enrichment_unfiltered(monkeypatch, tmp_path: Path) -> None:
    captured: dict[str, object] = {}

    def fake_run_scrape(**kwargs: object) -> dict[str, object]:
        captured.update(kwargs)
        return {"season": 2026, "run_id": None, "matches": 0}

    monkeypatch.setattr("sina_csl_scraper.cli.run_scrape", fake_run_scrape)

    runner = CliRunner()
    result = runner.invoke(
        app,
        [
            "scrape",
            "--season",
            "2026",
            "--output-dir",
            str(tmp_path),
            "--enrich-corners",
        ],
    )

    assert result.exit_code == 0
    assert captured["enrich_corners"] is True
    assert "enrich_match_ids" not in captured

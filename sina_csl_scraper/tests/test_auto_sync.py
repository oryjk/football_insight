from __future__ import annotations

from datetime import datetime

from sina_csl_scraper.auto_sync import (
    AutoSyncState,
    build_auto_sync_decision,
    resolve_match_due_at,
)
from sina_csl_scraper.models import MatchResult


def build_match(
    *,
    match_id: int = 1,
    date: str = "2026-04-12",
    time: str = "19:35",
) -> MatchResult:
    return MatchResult(
        match_id=match_id,
        season=2026,
        round_number=6,
        round_name="第6轮",
        date=date,
        time=time,
        status="1",
        home_team_id=1,
        home_team_name="成都蓉城",
        home_score="",
        away_team_id=2,
        away_team_name="北京国安",
        away_score="",
        home_logo="home.png",
        away_logo="away.png",
    )


def test_resolve_match_due_at_uses_kickoff_plus_130_minutes() -> None:
    due_at = resolve_match_due_at(build_match(date="2026-04-12", time="19:35"))

    assert due_at is not None
    assert due_at.isoformat() == "2026-04-12T21:45:00+08:00"


def test_build_auto_sync_decision_runs_once_when_new_due_match_appears() -> None:
    decision = build_auto_sync_decision(
        [build_match(match_id=101, date="2026-04-12", time="19:35")],
        now=datetime.fromisoformat("2026-04-12T21:46:00+08:00"),
        state=AutoSyncState(),
    )

    assert decision.should_run is True
    assert decision.newly_due_match_ids == (101,)
    assert decision.latest_due_at.isoformat() == "2026-04-12T21:45:00+08:00"


def test_build_auto_sync_decision_skips_before_kickoff() -> None:
    decision = build_auto_sync_decision(
        [build_match(match_id=101, date="2026-04-12", time="19:35")],
        now=datetime.fromisoformat("2026-04-12T19:34:00+08:00"),
        state=AutoSyncState(),
    )

    assert decision.should_run is False
    assert decision.newly_due_match_ids == ()
    assert decision.latest_due_at is None


def test_build_auto_sync_decision_does_not_repeat_same_due_window() -> None:
    decision = build_auto_sync_decision(
        [build_match(match_id=101, date="2026-04-12", time="19:35")],
        now=datetime.fromisoformat("2026-04-12T21:55:00+08:00"),
        state=AutoSyncState(
            last_processed_due_at=datetime.fromisoformat("2026-04-12T21:45:00+08:00"),
        ),
    )

    assert decision.should_run is False
    assert decision.newly_due_match_ids == ()
    assert decision.latest_due_at.isoformat() == "2026-04-12T21:45:00+08:00"


def test_build_auto_sync_decision_batches_multiple_new_due_matches() -> None:
    decision = build_auto_sync_decision(
        [
            build_match(match_id=101, date="2026-04-12", time="18:00"),
            build_match(match_id=102, date="2026-04-12", time="19:35"),
        ],
        now=datetime.fromisoformat("2026-04-12T21:46:00+08:00"),
        state=AutoSyncState(
            last_processed_due_at=datetime.fromisoformat("2026-04-12T20:00:00+08:00"),
        ),
    )

    assert decision.should_run is True
    assert decision.newly_due_match_ids == (101, 102)
    assert decision.latest_due_at.isoformat() == "2026-04-12T21:45:00+08:00"


def test_build_auto_sync_decision_runs_during_active_match_window() -> None:
    decision = build_auto_sync_decision(
        [build_match(match_id=201, date="2026-04-21", time="19:35")],
        now=datetime.fromisoformat("2026-04-21T20:05:00+08:00"),
        state=AutoSyncState(),
    )

    assert decision.should_run is True
    assert decision.newly_due_match_ids == ()
    assert decision.active_match_ids == (201,)
    assert decision.latest_due_at is None


def test_build_auto_sync_decision_throttles_active_match_window_by_last_run_at() -> None:
    decision = build_auto_sync_decision(
        [build_match(match_id=201, date="2026-04-21", time="19:35")],
        now=datetime.fromisoformat("2026-04-21T20:05:00+08:00"),
        state=AutoSyncState(
            last_run_at=datetime.fromisoformat("2026-04-21T20:04:30+08:00"),
        ),
    )

    assert decision.should_run is False
    assert decision.newly_due_match_ids == ()
    assert decision.active_match_ids == (201,)
    assert decision.latest_due_at is None

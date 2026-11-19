from __future__ import annotations

from datetime import date

import pytest

from sina_csl_scraper.director_sina_age_sync import (
    CandidateAgeTarget,
    approximate_birth_date_from_age,
    extract_sina_player_id_from_candidate_code,
    parse_player_age,
    sync_director_sina_ages,
)


class FakeAgeStore:
    def __init__(
        self,
        squad_ids: list[int] | None = None,
        candidate_targets: list[CandidateAgeTarget] | None = None,
    ) -> None:
        self._squad_ids = list(squad_ids or [])
        self._candidate_targets = list(candidate_targets or [])
        self.squad_updates: list[tuple[int, date]] = []
        self.candidate_updates: list[tuple[int, date]] = []

    def load_squad_missing_birth_ids(self) -> list[int]:
        return list(self._squad_ids)

    def load_candidate_missing_birth_targets(self) -> list[CandidateAgeTarget]:
        return list(self._candidate_targets)

    def update_squad_birth_date_if_null(self, player_id: int, birth_date: date) -> int:
        self.squad_updates.append((player_id, birth_date))
        return 1

    def update_candidate_birth_date_if_null(self, candidate_id: int, birth_date: date) -> int:
        self.candidate_updates.append((candidate_id, birth_date))
        return 1


def test_extract_sina_player_id_from_candidate_code():
    assert extract_sina_player_id_from_candidate_code("sina-k1-156047") == 156047
    assert extract_sina_player_id_from_candidate_code("sina-acl-999") == 999
    assert extract_sina_player_id_from_candidate_code("sina-cl1-42") == 42
    assert extract_sina_player_id_from_candidate_code("k1-cesinha") is None
    assert extract_sina_player_id_from_candidate_code("sina-k1-0") is None
    assert extract_sina_player_id_from_candidate_code("") is None
    assert extract_sina_player_id_from_candidate_code(None) is None


def test_parse_player_age_validates_bounds():
    assert parse_player_age({"age": "38"}) == 38
    assert parse_player_age({"age": 24}) == 24
    assert parse_player_age({"age": " 17 "}) == 17
    assert parse_player_age({"age": ""}) is None
    assert parse_player_age({"age": "0"}) is None
    assert parse_player_age({"age": "10"}) is None
    assert parse_player_age({"age": "60"}) is None
    assert parse_player_age({"age": "N/A"}) is None
    assert parse_player_age(None) is None


def test_approximate_birth_date_from_age():
    assert approximate_birth_date_from_age(38, today=date(2026, 11, 19)) == date(1988, 1, 1)
    assert approximate_birth_date_from_age(20, today=date(2026, 5, 1)) == date(2006, 1, 1)
    with pytest.raises(ValueError):
        approximate_birth_date_from_age(10, today=date(2026, 11, 19))


def test_sync_director_sina_ages_deduplicates_and_updates_both_tables():
    store = FakeAgeStore(
        squad_ids=[156047, 198252],
        candidate_targets=[
            CandidateAgeTarget(id=10, code="sina-csl-156047", sina_player_id=156047),
            CandidateAgeTarget(id=11, code="sina-k1-900001", sina_player_id=900001),
            CandidateAgeTarget(id=12, code="sina-j1-900002", sina_player_id=900002),
        ],
    )
    fetched: list[int] = []

    def fake_fetcher(pid: int):
        fetched.append(pid)
        if pid == 900002:
            raise RuntimeError("timeout")
        return {
            156047: {"nam_cn": "冯卓毅", "age": "38"},
            198252: {"nam_cn": "罗慕洛", "age": ""},
            900001: {"nam_cn": "塞西尼亚", "age": "36"},
        }[pid]

    summary = sync_director_sina_ages(
        store=store,
        profile_fetcher=fake_fetcher,
        dry_run=False,
        concurrency=1,
        today=date(2026, 11, 19),
    )

    # pid 156047 appears in both squad and candidates, must only be fetched once.
    assert fetched == [156047, 198252, 900001, 900002]
    assert summary["unique_pids_to_fetch"] == 4
    assert summary["fetched_ok"] == 3
    assert summary["fetched_failed"] == 1
    assert summary["ages_found"] == 2
    assert summary["squad_matched"] == 1
    assert summary["squad_updated"] == 1
    assert summary["candidate_matched"] == 2
    assert summary["candidate_updated"] == 2
    assert store.squad_updates == [(156047, date(1988, 1, 1))]
    assert store.candidate_updates == [
        (10, date(1988, 1, 1)),
        (11, date(1990, 1, 1)),
    ]


def test_sync_director_sina_ages_dry_run_and_limit():
    store = FakeAgeStore(
        squad_ids=[1, 2, 3],
        candidate_targets=[CandidateAgeTarget(id=20, code="sina-k1-4", sina_player_id=4)],
    )
    fetched: list[int] = []

    def fake_fetcher(pid: int):
        fetched.append(pid)
        return {"age": "25"}

    summary = sync_director_sina_ages(
        store=store,
        profile_fetcher=fake_fetcher,
        dry_run=True,
        limit=2,
        concurrency=2,
        today=date(2026, 11, 19),
    )

    assert sorted(fetched) == [1, 2]
    assert summary["dry_run"] is True
    assert summary["unique_pids_to_fetch"] == 2
    assert summary["squad_matched"] == 2
    assert summary["squad_updated"] == 0
    assert store.squad_updates == []
    assert store.candidate_updates == []

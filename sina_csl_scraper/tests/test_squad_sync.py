from __future__ import annotations

from sina_csl_scraper.models import SquadPlayer
from sina_csl_scraper.squad_sync import build_squad_rows, sync_squads


def player(pid, team=1, number=None, position="MF", logo=None):
    return SquadPlayer(team, pid, f"P{pid}", number, position, None, logo)


class MemoryStore:
    def __init__(self, teams, avatar_urls=None, active_counts=None):
        self.teams = teams
        self.urls = avatar_urls or {}
        self.active = active_counts or {}
        self.applied = []

    def active_count(self, team_id):
        return self.active.get(team_id, 0)

    def team_ids(self):
        return list(self.teams)

    def avatar_urls(self, team_id):
        return dict(self.urls.get(team_id, {}))

    def apply(self, team_id, season, rows, current):
        self.applied.append((team_id, season, [r["player_id"] for r in rows], current))


def full_squad(base=100, team=1, **kwargs):
    """11 名球员的合理名单（满足当前赛季名单的最小人数）。"""
    return [player(base + i, team=team, **kwargs) for i in range(11)]


def test_build_rows_keeps_existing_storage_url():
    rows = build_squad_rows([player(1), player(2)], 2026, {1: "https://minio/p1.png"})
    assert rows[0]["avatar_storage_url"] == "https://minio/p1.png"
    assert rows[1]["avatar_storage_url"] is None
    assert rows[0]["season"] == 2026


def test_history_seasons_apply_before_current_and_current_is_marked():
    calls = []

    def fetch(team_id, season):
        calls.append(season)
        return full_squad(base=season * 100)

    store = MemoryStore([7])
    summary = sync_squads(fetch=fetch, store=store, season=2026, backfill_seasons=2)
    assert calls == [2024, 2025, 2026]
    assert [(s, current) for _, s, _, current in store.applied] == [(2024, False), (2025, False), (2026, True)]
    assert summary["teams_ok"] == 1 and summary["teams_failed"] == []


def test_empty_current_squad_is_a_failure_and_changes_nothing():
    store = MemoryStore([7])
    summary = sync_squads(fetch=lambda t, s: [], store=store, season=2026, backfill_seasons=0)
    assert store.applied == []
    assert summary["teams_failed"] == [7]


def test_one_team_failing_does_not_stop_others():
    def fetch(team_id, season):
        if team_id == 1:
            raise RuntimeError("boom")
        return full_squad(team=team_id)

    store = MemoryStore([1, 2])
    summary = sync_squads(fetch=fetch, store=store, season=2026, backfill_seasons=0)
    assert summary["teams_failed"] == [1]
    assert [t for t, *_ in store.applied] == [2]


def test_backfill_failure_still_applies_current_season():
    def fetch(team_id, season):
        if season == 2025:
            raise RuntimeError("history down")
        return full_squad()

    store = MemoryStore([3])
    summary = sync_squads(fetch=fetch, store=store, season=2026, backfill_seasons=1)
    assert [(s, c) for _, s, _, c in store.applied] == [(2026, True)]
    assert summary["teams_ok"] == 1


def test_backfill_never_overrides_newer_season_or_active():
    # 回填只以 current=False 写入；"不覆盖更新赛季"由 SQL 保证（见 postgres 测试）
    store = MemoryStore([3])
    sync_squads(fetch=lambda t, s: full_squad(), store=store, season=2026, backfill_seasons=1)
    assert [c for *_, c in store.applied] == [False, True]


def test_avatars_upload_only_when_missing_and_failures_are_tolerated():
    uploaded = []

    def avatars(p):
        uploaded.append(p.player_id)
        if p.player_id == 3:
            raise RuntimeError("cdn down")
        return f"https://minio/{p.player_id}.png"

    store = MemoryStore([1], avatar_urls={1: {1: "https://minio/1.png"}})
    fetched = [player(1, logo="http://a/1.png"), player(2, logo="http://a/2.png"), player(3, logo="http://a/3.png"), player(4)] + [player(10 + i) for i in range(7)]
    summary = sync_squads(fetch=lambda t, s: fetched, store=store, season=2026, backfill_seasons=0, avatars=avatars)
    assert uploaded == [2, 3]
    assert summary["avatars_uploaded"] == 1
    assert len(store.applied[0][2]) == 11


def squad(n, team=1):
    return [player(i, team=team) for i in range(1, n + 1)]


def test_tiny_current_squad_is_refused_as_suspicious():
    # 只解析出少量球员（字段改名或列表被截断）时，不能据此把其余球员标成离队。
    store = MemoryStore([7])
    summary = sync_squads(fetch=lambda t, s: squad(3, team=t), store=store, season=2026, backfill_seasons=0)
    assert store.applied == []
    assert summary["teams_failed"] == [7]


def test_current_squad_far_smaller_than_known_active_is_refused():
    store = MemoryStore([7], active_counts={7: 30})
    summary = sync_squads(fetch=lambda t, s: squad(14, team=t), store=store, season=2026, backfill_seasons=0)
    assert store.applied == []
    assert summary["teams_failed"] == [7]


def test_reasonable_shrink_is_accepted():
    # 30 → 16（约 53%）属于正常的名单收缩，应照常写入
    store = MemoryStore([7], active_counts={7: 30})
    summary = sync_squads(fetch=lambda t, s: squad(16, team=t), store=store, season=2026, backfill_seasons=0)
    assert [c for *_, c in store.applied] == [True]
    assert summary["teams_ok"] == 1


def test_first_sync_of_a_new_team_only_needs_the_minimum_size():
    store = MemoryStore([7], active_counts={7: 0})
    summary = sync_squads(fetch=lambda t, s: squad(11, team=t), store=store, season=2026, backfill_seasons=0)
    assert summary["teams_ok"] == 1

from dataclasses import replace

import pytest

from sina_csl_scraper.models import MatchResult
from sina_csl_scraper.player_match_goals import CflPlayerLogClient, map_player_logs, resolve_official_player
from sina_csl_scraper.player_goal_sync import validate_complete_player_logs


def match(**changes):
    base = MatchResult(10, 2026, 1, "第1轮", "2026-03-07", "19:35", "3",
                       500, "云南玉昆", "3", 143, "青岛海牛", "1", "", "")
    return replace(base, **changes)


def log(**changes):
    return {"match_id": "official-match", "player_id": "oscar-id",
            "tournament_calendar_name": "2026", "competition_code": "CSL",
            "local_date": "2026-03-07", "local_time": "19:35:00",
            "home_contestant_id": "3n4jrcw6w965v1qvp0azx5kwk",
            "away_contestant_id": "a3rgv4o76uowd2gxzx9pgf02d",
            "minutes_played": 90, "goals": 0, **changes}


def parse(rows, matches=None):
    return map_player_logs(player_id=456290, official_player_id="oscar-id", season=2026,
                           rows=rows, matches=matches or [match()])


def test_identity_requires_exact_name_and_team_and_rejects_ambiguity():
    correct = {"id": "oscar-id", "name": "奥斯卡", "contestant_id": "3n4jrcw6w965v1qvp0azx5kwk"}
    wrong_team = {**correct, "id": "other", "contestant_id": "cvx4yvx5oby9dz7vm70mw04um"}
    assert resolve_official_player([wrong_team, correct], name="奥斯卡", team_id=500, season=2026) == "oscar-id"
    with pytest.raises(ValueError):
        resolve_official_player([correct, {**correct, "id": "ambiguous"}], name="奥斯卡", team_id=500, season=2026)


def test_preserves_zero_and_hat_trick_and_ignores_zero_minutes():
    assert parse([log()])[0].goals == 0
    assert parse([log(goals=3)])[0].goals == 3
    assert parse([log(minutes_played=0)]) == []


@pytest.mark.parametrize("value", [None, "", -1, "1.5", 1.5, True])
def test_invalid_goals_never_become_zero(value):
    with pytest.raises(ValueError):
        parse([log(goals=value)])


@pytest.mark.parametrize("changes", [{"player_id": "other"}, {"competition_code": "CFA"},
                                    {"tournament_calendar_name": "2025"},
                                    {"local_date": "2026-03-08"}])
def test_refuses_wrong_identity_season_competition_or_fixture(changes):
    with pytest.raises(ValueError):
        parse([log(**changes)])


def test_only_finished_matches_and_consistent_duplicates_are_accepted():
    assert parse([log()], [match(status="2")]) == []
    assert len(parse([log(), log()])) == 1
    with pytest.raises(ValueError):
        parse([log(), log(goals=1)])


class Response:
    def __init__(self, data):
        self.data = data

    def raise_for_status(self):
        pass

    def json(self):
        return self.data


class Session:
    headers = {}

    def __init__(self, *, incomplete=False, ambiguous=False, duplicate=False, transfer=False, sparse_search=False):
        self.incomplete = incomplete
        self.ambiguous = ambiguous
        self.duplicate = duplicate
        self.transfer = transfer
        self.sparse_search = sparse_search

    def get(self, url, *, params, timeout):
        if url.endswith('/players/find') and params.get('name_like') == '奥斯卡':
            if self.sparse_search and params['curPage'] > 1:
                return Response({'status': 200, 'data': {'count': 3, 'dataList': []}})
            candidates = [{"id": "oscar-id", "name": "奥斯卡", "contestant_id": "3n4jrcw6w965v1qvp0azx5kwk"}]
            if self.ambiguous:
                candidates.append({**candidates[0], "id": "other"})
            if self.transfer:
                candidates.append({**candidates[0], "contestant_id": "cvx4yvx5oby9dz7vm70mw04um"})
            return Response({"status": 200, "data": {"count": 3 if self.sparse_search else len(candidates), "dataList": candidates}})
        if (url.endswith('/players/page') and self.sparse_search
                and params.get('contestant_id') == '3n4jrcw6w965v1qvp0azx5kwk'):
            return Response({'status': 200, 'data': {'count': 1, 'dataList': [{
                'id': 'roster-row-uuid', 'player_id': 'oscar-id', 'player_name': '奥斯卡',
                'contestant_id': '3n4jrcw6w965v1qvp0azx5kwk',
            }]}})
        if self.transfer and params.get('contestant_id') == 'cvx4yvx5oby9dz7vm70mw04um':
            return Response({"status": 200, "data": {"count": 1, "dataList": [log(match_id='previous-club', goals=1)]}})
        if (url.endswith('/players/player/played') and params.get('player_id') == 'oscar-id'
                and params.get('contestant_id') == '3n4jrcw6w965v1qvp0azx5kwk'
                and params.get('tournament_calendar_id') == 'e6818x4pwankpph8awr91m1hw'):
            rows = [log(goals=3)] if params['curPage'] == 1 else ([] if self.incomplete else [log(match_id='second-match')])
            if self.duplicate:
                rows = [log(goals=3)]
            return Response({"status": 200, "data": {"count": 2, "dataList": rows}})
        return Response({"status": 400})


def test_client_uses_verified_identity_required_club_and_all_pages():
    identity, rows = CflPlayerLogClient(session=Session()).fetch_player_logs(name='奥斯卡', team_id=500, season=2026)
    assert identity == 'oscar-id'
    assert [row['goals'] for row in rows] == [3, 0]


def test_same_player_identity_at_two_clubs_includes_both_club_logs():
    identity, rows = CflPlayerLogClient(session=Session(transfer=True)).fetch_player_logs(name='奥斯卡', team_id=500, season=2026)
    assert identity == 'oscar-id'
    assert [row['goals'] for row in rows] == [3, 0, 1]


def test_sparse_name_search_uses_complete_season_club_roster_for_identity():
    identity, rows = CflPlayerLogClient(session=Session(sparse_search=True)).fetch_player_logs(name='奥斯卡', team_id=500, season=2026)
    assert identity == 'oscar-id'
    assert [row['goals'] for row in rows] == [3, 0]


def test_sparse_search_preserves_previous_clubs_after_independent_roster_verification():
    identity, rows = CflPlayerLogClient(session=Session(sparse_search=True, transfer=True)).fetch_player_logs(name='奥斯卡', team_id=500, season=2026)
    assert identity == 'oscar-id'
    assert [row['goals'] for row in rows] == [3, 0, 1]


def test_client_rejects_partial_pages_and_ambiguous_player_instead_of_partial_history():
    for session in [Session(incomplete=True), Session(ambiguous=True), Session(duplicate=True)]:
        with pytest.raises(ValueError):
            CflPlayerLogClient(session=session).fetch_player_logs(name='奥斯卡', team_id=500, season=2026)


def test_incomplete_season_cannot_replace_verified_zero_goal_appearances():
    rows = parse([log(goals=3)])
    validate_complete_player_logs(rows, expected_goals='3', existing_match_ids={10})
    with pytest.raises(ValueError):
        validate_complete_player_logs(rows, expected_goals='4', existing_match_ids=set())
    with pytest.raises(ValueError):
        validate_complete_player_logs(rows, expected_goals='3', existing_match_ids={10, 11})

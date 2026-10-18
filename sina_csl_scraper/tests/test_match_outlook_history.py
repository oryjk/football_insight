from dataclasses import replace
from datetime import UTC, datetime
import pytest
from sina_csl_scraper.match_outlook.model import Game
from sina_csl_scraper.match_outlook.history import (
    validate_history,
    expected_count,
    game_from_dict,
)


def season_rows(year):
    return [
        Game(
            year * 1000 + i,
            year,
            datetime(year, 1, 1, tzinfo=UTC),
            1,
            2,
            0,
            0,
            year in (2020, 2021),
        )
        for i in range(expected_count(year))
    ]


def test_historical_counts_and_neutral_seasons():
    for year, count in [(2019, 240), (2020, 160), (2021, 176), (2022, 306)]:
        assert expected_count(year) == count
        validate_history(season_rows(year), [year])
        with pytest.raises(ValueError):
            validate_history(season_rows(year)[:-1], [year])


def test_duplicate_missing_score_and_invalid_time_rejected_before_writing():
    rows = season_rows(2025)
    for invalid in [
        rows[:-1] + [rows[0]],
        rows[:-1] + [replace(rows[-1], home_score=-1)],
        rows[:-1] + [replace(rows[-1], neutral=True)],
    ]:
        with pytest.raises(ValueError):
            validate_history(invalid, [2025])
    raw = {
        "match_id": 1,
        "season": 2025,
        "date": "2025-01-01",
        "time": "19:35",
        "home_team_id": 1,
        "away_team_id": 2,
        "home_score": "",
        "away_score": "0",
    }
    with pytest.raises(ValueError):
        game_from_dict(raw)
    with pytest.raises(ValueError):
        game_from_dict({**raw, "home_score": "0", "time": "--"})
    assert (
        game_from_dict({**raw, "home_score": "0"}).kickoff.utcoffset().total_seconds()
        == 28800
    )
    with pytest.raises(ValueError):
        game_from_dict({**raw, "home_score": "0", "status": "1"})

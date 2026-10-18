from datetime import datetime, UTC
from dataclasses import replace
import pytest
from sina_csl_scraper.match_outlook.model import Params, Game
from sina_csl_scraper.match_outlook.service import build_predictions
from test_match_outlook_history import season_rows


def test_2027_rollover_reselects_parameters_and_does_not_learn_future_scores(
    monkeypatch,
):
    rows = [game for year in range(2014, 2027) for game in season_rows(year)]
    targets = []

    def choose(history, years):
        targets.extend(years)
        assert max(game.season for game in history) == 2026
        return {year: Params(newstart=1400) for year in years}

    monkeypatch.setattr("sina_csl_scraper.match_outlook.service.tune", choose)
    future = Game(999999, 2027, datetime(2027, 3, 1, tzinfo=UTC), 99, 100, 0, 0, False)
    now = datetime(2027, 1, 1, tzinfo=UTC)
    a = build_predictions(
        rows, [future, replace(future, match_id=999998, home_team_id=101)], now
    )
    b = build_predictions(
        rows,
        [
            replace(future, home_score=99),
            replace(future, match_id=999998, home_team_id=101),
        ],
        now,
    )
    assert targets == [2027, 2027]
    assert a[0][1] == b[0][1] and a[1][1] == b[1][1]
    assert a[0][3]["newstart"] == 1400
    assert (
        build_predictions(
            rows, [replace(future, kickoff=datetime(2026, 1, 1, tzinfo=UTC))], now
        )
        == []
    )


def test_incomplete_prior_season_is_rejected_before_publication():
    future = Game(999999, 2027, datetime(2027, 3, 1, tzinfo=UTC), 99, 100, 0, 0, False)
    with pytest.raises(ValueError, match="incomplete historical season"):
        build_predictions(
            [game for year in range(2014, 2026) for game in season_rows(year)],
            [future],
            datetime(2027, 1, 1, tzinfo=UTC),
        )

from dataclasses import replace
from datetime import datetime, timedelta, UTC
import pytest
from sina_csl_scraper.match_outlook.model import (
    Game,
    Params,
    Replay,
    probabilities,
    rps,
    tune,
    baseline,
)


def games(n=60, season=2026):
    return [
        Game(
            i,
            season,
            datetime(season, 1, 1, tzinfo=UTC) + timedelta(days=i),
            i % 4 + 1,
            (i + 1) % 4 + 1,
            i % 3,
            (i + 1) % 3,
            False,
        )
        for i in range(n)
    ]


def test_probability_simple_baseline_and_score():
    for x in [-1000, -1, 0, 1, 1000]:
        p = probabilities(x, 0.7)
        assert min(p) >= 0 and sum(p) == pytest.approx(1)
    assert probabilities(0, 0.7)[0] == probabilities(0, 0.7)[2]
    assert rps((0.5, 0.25, 0.25), 0) == pytest.approx(0.15625)
    assert baseline([]) == pytest.approx((0.46, 0.26, 0.28))
    assert baseline([games(1)[0]]) == pytest.approx((13.8 / 31, 7.8 / 31, 9.4 / 31))
    assert baseline([replace(games(1)[0], neutral=True)]) == pytest.approx(
        (0.46, 0.26, 0.28)
    )


def test_first_seen_teams_use_newstart_and_season_anchor_is_1500():
    model = Replay(Params(newstart=1400))
    model.predict(games(1)[0])
    assert model.ratings == {1: 1400, 2: 1400}
    model.predict(replace(games(1)[0], season=2027, home_team_id=3))
    assert model.ratings[1] == pytest.approx(1420)
    assert model.ratings[3] == 1400


def test_future_results_do_not_change_earlier_forecasts_and_ids_survive_renames():
    rows = games()
    model = Replay(Params())
    a = model.run(rows)
    changed = [
        g if i < 30 else replace(g, home_score=9, away_score=0)
        for i, g in enumerate(rows)
    ]
    b = Replay(Params()).run(changed)
    assert a[:31] == b[:31]
    assert all(sum(p.probabilities) == pytest.approx(1) for p in a)
    assert all(0.3 <= p.tau <= 1.2 for p in a)
    assert a[0].tau == a[39].tau == 1


def test_neutral_removes_home_advantage_and_adaptive_expectation_drives_update():
    g = games(1)[0]
    p = Params()
    normal = Replay(p).predict(g)
    neutral = Replay(p).predict(replace(g, neutral=True))
    assert neutral.probabilities[0] == pytest.approx(neutral.probabilities[2])
    assert normal.probabilities[0] > neutral.probabilities[0]
    model = Replay(p)
    pred = model.predict(g)
    model.update(g, pred)
    expected = (
        p.K
        * (1 + abs(g.home_score - g.away_score)) ** p.gexp
        * (0 - (pred.probabilities[0] + 0.5 * pred.probabilities[1]))
    )
    assert model.ratings[g.home_team_id] == pytest.approx(p.newstart + expected)


def test_parameter_tuning_ignores_target_and_future_seasons():
    rows = games(12, 2023) + games(12, 2024) + games(12, 2025)
    grid = [Params(), Params(K=45)]
    first = tune(rows, [2026], grid)[2026]
    assert tune(rows + games(10, 2026), [2026], grid)[2026] == first


def test_simultaneous_fixtures_cannot_learn_each_others_results():
    rows = games(60)
    first = replace(rows[-1], match_id=100, home_team_id=10, away_team_id=11)
    second = replace(first, match_id=101, home_team_id=12, away_team_id=13)

    class ObservedReplay(Replay):
        def __init__(self):
            super().__init__(Params())
            self.observed = []

        def predict(self, game):
            self.observed.append(len(self.history))
            return super().predict(game)

    model = ObservedReplay()
    model.run(rows[:-1] + [first, second])
    assert model.observed[-2:] == [59, 59]
    a = Replay(Params()).run(rows[:-1] + [first, second])
    b = Replay(Params()).run(rows[:-1] + [replace(first, home_score=99), second])
    assert a[-1] == b[-1]

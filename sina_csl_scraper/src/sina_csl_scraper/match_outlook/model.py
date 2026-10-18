"""Deterministic score-only Elo V2; no IO and no account data.

The 3888 parameter candidates replay together in NumPy. Selection uses tau=1;
only the selected trajectory runs adaptive shrinkage. Simultaneous fixtures are
predicted together so they cannot learn each other's results.
"""

from __future__ import annotations

from dataclasses import dataclass
from datetime import datetime
from itertools import product, groupby
from math import exp

import numpy as np


@dataclass(frozen=True)
class Game:
    match_id: int
    season: int
    kickoff: datetime
    home_team_id: int
    away_team_id: int
    home_score: int
    away_score: int
    neutral: bool

    @property
    def outcome(self) -> int:
        return (
            0
            if self.home_score > self.away_score
            else 1
            if self.home_score == self.away_score
            else 2
        )


@dataclass(frozen=True)
class Params:
    K: float = 10
    H: float = 60
    s: float = 150
    c: float = 0.7
    gexp: float = 1
    reg: float = 0.8
    newstart: float = 1450


@dataclass(frozen=True)
class Forecast:
    probabilities: tuple[float, float, float]
    tau: float
    x: float


def probabilities(x: float, c: float) -> tuple[float, float, float]:
    sigmoid = lambda v: 1 / (1 + exp(-max(-700, min(700, v))))
    home, away = sigmoid(x - c), sigmoid(-x - c)
    draw = max(1 - home - away, 1e-6)
    total = home + draw + away
    return home / total, draw / total, away / total


def rps(p, outcome: int) -> float:
    return (
        (p[0] - (1 if outcome == 0 else 0)) ** 2
        + (p[0] + p[1] - (1 if outcome <= 1 else 0)) ** 2
    ) / 2


def baseline(games: list[Game]) -> tuple[float, float, float]:
    counts = [13.8, 7.8, 8.4]
    for game in games:
        if not game.neutral:
            counts[game.outcome] += 1
    return tuple(c / sum(counts) for c in counts)


TAUS = np.array([0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0, 1.1, 1.2])


class Replay:
    def __init__(self, params: Params, *, adaptive: bool = True):
        self.params, self.adaptive = params, adaptive
        self.ratings: dict[int, float] = {}
        self.history: list[tuple[float, int]] = []
        self.season: int | None = None

    def _tau(self) -> float:
        if not self.adaptive or len(self.history) < 40:
            return 1.0
        window = self.history[-150:]
        xs = np.array([x for x, _ in window])
        outcomes = np.array([o for _, o in window])
        x = TAUS[:, None] * xs
        ph = 1 / (1 + np.exp(np.clip(-(x - self.params.c), -700, 700)))
        pa = 1 / (1 + np.exp(np.clip(x + self.params.c, -700, 700)))
        pd = np.maximum(1 - ph - pa, 1e-6)
        all_p = np.stack((ph, pd, pa), axis=2)
        all_p /= all_p.sum(axis=2)[:, :, None]
        chosen = all_p[:, np.arange(len(xs)), outcomes]
        loss = -np.log(np.maximum(chosen, 1e-12)).sum(axis=1)
        return float(TAUS[int(np.argmin(loss))])

    def predict(self, game: Game) -> Forecast:
        p = self.params
        if game.season != self.season:
            self.season = game.season
            self.ratings = {
                team: p.reg * rating + (1 - p.reg) * 1500
                for team, rating in self.ratings.items()
            }
        home = self.ratings.setdefault(game.home_team_id, p.newstart)
        away = self.ratings.setdefault(game.away_team_id, p.newstart)
        x = (home + (0 if game.neutral else p.H) - away) / p.s
        tau = self._tau()
        return Forecast(probabilities(x * tau, p.c), tau, x)

    def update(self, game: Game, prediction: Forecast) -> None:
        p = self.params
        expected = prediction.probabilities[0] + 0.5 * prediction.probabilities[1]
        delta = (
            p.K
            * (1 + abs(game.home_score - game.away_score)) ** p.gexp
            * ((1.0, 0.5, 0.0)[game.outcome] - expected)
        )
        self.ratings[game.home_team_id] += delta
        self.ratings[game.away_team_id] -= delta
        self.history.append((prediction.x, game.outcome))

    def run(self, games: list[Game]) -> list[Forecast]:
        predictions = []
        for _, simultaneous in groupby(games, key=lambda game: game.kickoff):
            batch = list(simultaneous)
            forecasts = [self.predict(game) for game in batch]
            predictions.extend(forecasts)
            # Even a simultaneous result is unavailable at prediction time.
            for game, prediction in zip(batch, forecasts):
                self.update(game, prediction)
        return predictions


def parameter_grid() -> list[Params]:
    return [
        Params(*p)
        for p in product(
            (10, 20, 30, 45),
            (30, 60, 90),
            (150, 200, 250),
            (0.3, 0.5, 0.7),
            (0, 0.5, 1.0),
            (0.4, 0.6, 0.8, 1.0),
            (1400, 1450, 1500),
        )
    ]


def tune(
    games: list[Game], targets: list[int], grid: list[Params] | None = None
) -> dict[int, Params]:
    grid = parameter_grid() if grid is None else grid
    if not games or not grid:
        raise ValueError("complete prior history required")
    # The target/future season never enters the tuning replay or scoring window.
    games = [g for g in games if g.season < max(targets)]
    values = np.array([[p.K, p.H, p.s, p.c, p.gexp, p.reg, p.newstart] for p in grid]).T
    K, H, s, c, gexp, reg, ns = values
    teams = {
        team: i
        for i, team in enumerate(
            sorted({t for g in games for t in (g.home_team_id, g.away_team_id)})
        )
    }
    ratings = np.tile(ns[:, None], (1, len(teams)))
    seen: set[int] = set()
    season = None
    totals: dict[int, np.ndarray] = {}
    counts: dict[int, int] = {}
    for game in games:
        if game.season != season:
            season = game.season
            for team in seen:
                ratings[:, team] = reg * ratings[:, team] + (1 - reg) * 1500
        hi, ai = teams[game.home_team_id], teams[game.away_team_id]
        seen.update((hi, ai))
        x = (ratings[:, hi] + (0 if game.neutral else H) - ratings[:, ai]) / s
        ph = 1 / (1 + np.exp(np.clip(-(x - c), -700, 700)))
        pa = 1 / (1 + np.exp(np.clip(x + c, -700, 700)))
        pd = np.maximum(1 - ph - pa, 1e-6)
        z = ph + pd + pa
        ph, pd = ph / z, pd / z
        loss = (
            (ph - (game.outcome == 0)) ** 2 + (ph + pd - (game.outcome <= 1)) ** 2
        ) / 2
        totals[season] = totals.get(season, np.zeros(len(grid))) + loss
        counts[season] = counts.get(season, 0) + 1
        delta = (
            K
            * (1 + abs(game.home_score - game.away_score)) ** gexp
            * ((1.0, 0.5, 0.0)[game.outcome] - (ph + 0.5 * pd))
        )
        ratings[:, hi] += delta
        ratings[:, ai] -= delta
    selected = {}
    for target in targets:
        window = [year for year in range(target - 3, target) if year >= 2016]
        if not window or any(year not in totals for year in window):
            raise ValueError("three prior tuning seasons required")
        objective = np.mean([totals[year] / counts[year] for year in window], axis=0)
        selected[target] = grid[int(np.argmin(objective))]
    return selected

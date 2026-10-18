"""Opt-in acceptance against the same score snapshot as research/fix.py; no IO services."""

import json
import os
from pathlib import Path

import pytest
from sina_csl_scraper.match_outlook.history import game_from_dict, validate_history
from sina_csl_scraper.match_outlook.model import Params, Replay, rps, tune


def test_reference_snapshot_reproduces_v2_rps_and_2026_parameters():
    directory = os.getenv("MATCH_OUTLOOK_HISTORY_DIR")
    if not directory:
        pytest.skip("explicit research score snapshot directory required")
    rows = []
    for year in range(2014, 2027):
        raw = json.loads((Path(directory) / f"{year}.json").read_text())
        rows.extend(
            game_from_dict(row)
            for row in raw
            if row["home_score"] != "" and row["away_score"] != ""
        )
    validate_history(
        [game for game in rows if game.season < 2026], list(range(2014, 2026))
    )
    rows.sort(key=lambda game: (game.kickoff, game.match_id))
    chosen = tune(rows, list(range(2019, 2027)))
    assert chosen[2026] == Params(10, 60, 150, 0.7, 1, 0.8, 1450)
    scores = []
    for year, params in chosen.items():
        history = [game for game in rows if game.season <= year]
        predictions = Replay(params).run(history)
        scores.extend(
            rps(prediction.probabilities, game.outcome)
            for game, prediction in zip(history, predictions)
            if game.season == year and not game.neutral
        )
    assert len(scores) == 1487, (
        "use the frozen October research snapshot, not a later season update"
    )
    actual = sum(scores) / len(scores)
    print(
        f"production V2 RPS={actual:.14f}; sample={len(scores)}; 2026 parameters={chosen[2026]}"
    )
    assert actual == pytest.approx(0.1988, abs=0.0005)

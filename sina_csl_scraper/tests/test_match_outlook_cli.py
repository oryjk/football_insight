from types import SimpleNamespace
import pytest
from sina_csl_scraper.match_outlook.cli import collect_history, backfill


def test_history_download_resumes_only_after_completed_rounds(tmp_path, monkeypatch):
    monkeypatch.setattr("sina_csl_scraper.match_outlook.cli.time.sleep", lambda _: None)
    calls = []

    def fetch(year, rnd):
        calls.append(rnd)
        if rnd == 3 and calls.count(3) == 1:
            raise RuntimeError("interrupted")
        return [
            SimpleNamespace(
                to_dict=lambda i=i: {
                    "match_id": rnd * 100 + i,
                    "season": year,
                    "date": "2025-01-01",
                    "time": "19:35",
                    "home_team_id": 1,
                    "away_team_id": 2,
                    "home_score": "0",
                    "away_score": "0",
                    "status": "3",
                }
            )
            for i in range(8)
        ]

    client = SimpleNamespace(fetch_round_matches=fetch)
    with pytest.raises(RuntimeError):
        collect_history(client, [2025], tmp_path, 0.1)
    assert len(collect_history(client, [2025], tmp_path, 0.1)) == 240
    assert calls[:5] == [1, 2, 3, 3, 4]
    client.fetch_round_matches = lambda *_: pytest.fail(
        "valid cache should not be downloaded again"
    )
    assert len(collect_history(client, [2025], tmp_path, 0.1)) == 240


def test_invalid_history_cannot_connect_to_write_database(tmp_path, monkeypatch):
    def invalid(*_):
        raise ValueError("incomplete historical season")

    monkeypatch.setattr("sina_csl_scraper.match_outlook.cli.collect_history", invalid)
    monkeypatch.setattr(
        "sina_csl_scraper.match_outlook.cli.psycopg.connect",
        lambda *_: pytest.fail("must validate before connecting"),
    )
    with pytest.raises(ValueError):
        backfill(2014, 2025, tmp_path, 0.1, True, "unused")

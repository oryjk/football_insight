from sina_csl_scraper.sync import InsightSyncService
from test_sync import FakeRepository, _sample_payload


def test_outlook_failure_after_core_commit_does_not_roll_back_sync():
    repo = FakeRepository()
    events = []

    def fail(season):
        events.extend(repo.calls)
        raise ValueError("model unavailable")

    result = InsightSyncService(repo, after_sync=fail).sync(_sample_payload())
    assert result.run_id == "run-1"
    assert any(name == "commit_sync" for name, _ in events)
    assert not any(name == "rollback_sync" for name, _ in repo.calls)

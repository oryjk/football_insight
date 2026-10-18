import os
from uuid import uuid4
from psycopg import sql
from datetime import UTC, datetime, timedelta
from pathlib import Path
from urllib.parse import urlsplit
import psycopg
import pytest
from sina_csl_scraper.match_outlook.repository import (
    publish,
    score_finished,
    annotate_reschedules,
)


@pytest.fixture
def connection():
    url = os.getenv("MATCH_OUTLOOK_TEST_DATABASE_URL")
    if not url:
        pytest.skip("explicit isolated localhost match_outlook_test database required")
    parsed = urlsplit(url)
    assert parsed.hostname == "127.0.0.1" and parsed.path == "/match_outlook_test"
    with psycopg.connect(url, autocommit=True) as c:
        schema = "outlook_test_" + uuid4().hex
        c.execute(sql.SQL("CREATE SCHEMA {}").format(sql.Identifier(schema)))
        c.execute(sql.SQL("SET search_path TO {}").format(sql.Identifier(schema)))
        c.execute(
            "CREATE TABLE f_i_matches(match_id bigint primary key, season int, status text, source_active bool, match_date date, match_time text, home_score text, away_score text)"
        )
        migration = (
            Path(__file__).parents[2]
            / "football_insight_service_backend_rs/migrations/20261018140000_add_match_outlooks.sql"
        )
        with c.transaction(force_rollback=True):
            c.execute("SET LOCAL lock_timeout='2s'")
            c.execute(migration.read_text())
        assert (
            c.execute("SELECT to_regclass('f_i_match_outlooks')").fetchone()[0] is None
        )
        c.execute(migration.read_text())
        c.execute(
            "INSERT INTO f_i_matches VALUES(1,2026,'1',true,'2099-01-01','19:35','',''),(2,2026,'3',true,'2026-01-01','19:35','2','0')"
        )
        yield c


def insert_frozen(c):
    c.execute("""INSERT INTO f_i_match_outlooks(match_id,home_win,draw,away_win,baseline_home,baseline_draw,baseline_away,tau,model_version,params,generated_at,kickoff_at)
      VALUES(2,.5,.25,.25,.46,.26,.28,.7,'elo-v2','{}',clock_timestamp()-interval '2h',clock_timestamp()-interval '1h')""")


def test_future_only_and_direct_frozen_updates_are_rejected(connection):
    c = connection
    future = (datetime.now(UTC) + timedelta(hours=1)).replace(microsecond=0)
    c.execute(
        "UPDATE f_i_matches SET match_date=(%s AT TIME ZONE 'Asia/Shanghai')::date,match_time=(%s AT TIME ZONE 'Asia/Shanghai')::time::text WHERE match_id=1",
        (future, future),
    )
    assert publish(c, 1, (0.5, 0.25, 0.25), (0.46, 0.26, 0.28), 0.7, {}, future) == 1
    assert publish(c, 1, (0.4, 0.3, 0.3), (0.46, 0.26, 0.28), 0.7, {}, future) == 1
    insert_frozen(c)
    before = c.execute(
        "SELECT home_win,generated_at,kickoff_at,params FROM f_i_match_outlooks WHERE match_id=2"
    ).fetchone()
    assert publish(c, 2, (0.4, 0.3, 0.3), (0.46, 0.26, 0.28), 0.7, {}, future) == 0
    assert (
        c.execute(
            "UPDATE f_i_match_outlooks SET home_win=.4,draw=.3,away_win=.3 WHERE match_id=2"
        ).rowcount
        == 0
    )
    assert (
        c.execute(
            "SELECT home_win,generated_at,kickoff_at,params FROM f_i_match_outlooks WHERE match_id=2"
        ).fetchone()
        == before
    )
    # No forecast is created by arriving after its scheduled kickoff.
    c.execute(
        "INSERT INTO f_i_matches VALUES(3,2026,'3',true,'2026-01-01','19:35','0','0')"
    )
    assert (
        publish(
            c,
            3,
            (0.5, 0.25, 0.25),
            (0.46, 0.26, 0.28),
            1,
            {},
            datetime.now(UTC) - timedelta(hours=1),
        )
        == 0
    )


def test_scoring_is_once_only_and_uses_original_frozen_probabilities(connection):
    c = connection
    insert_frozen(c)
    assert score_finished(c) == 1
    first = c.execute(
        "SELECT outcome,rps_model,rps_baseline,scored_at FROM f_i_match_outlooks WHERE match_id=2"
    ).fetchone()
    assert first[0] == 0 and first[1] == pytest.approx(0.15625)
    assert first[2] == pytest.approx((0.54**2 + 0.28**2) / 2)
    c.execute("UPDATE f_i_matches SET home_score='0',away_score='4' WHERE match_id=2")
    assert score_finished(c) == 0
    assert (
        c.execute("UPDATE f_i_match_outlooks SET rps_model=0 WHERE match_id=2").rowcount
        == 0
    )
    assert (
        c.execute(
            "SELECT outcome,rps_model,rps_baseline,scored_at FROM f_i_match_outlooks WHERE match_id=2"
        ).fetchone()
        == first
    )


def test_frozen_reschedule_is_marked_without_rewriting_forecast(connection):
    c = connection
    insert_frozen(c)
    c.execute(
        "UPDATE f_i_matches SET match_date='2099-01-01',status='1' WHERE match_id=2"
    )
    before = c.execute(
        "SELECT home_win,generated_at,kickoff_at FROM f_i_match_outlooks WHERE match_id=2"
    ).fetchone()
    assert annotate_reschedules(c) == 1
    assert (
        c.execute(
            "SELECT params->>'schedule_changed' FROM f_i_match_outlooks WHERE match_id=2"
        ).fetchone()[0]
        == "true"
    )
    assert (
        c.execute(
            "SELECT home_win,generated_at,kickoff_at FROM f_i_match_outlooks WHERE match_id=2"
        ).fetchone()
        == before
    )
    c.execute("UPDATE f_i_matches SET match_time='待定' WHERE match_id=2")
    assert annotate_reschedules(c) == 0


def test_publish_rejects_changed_schedule_or_inactive_fixture(connection):
    c = connection
    future = (datetime.now(UTC) + timedelta(hours=1)).replace(microsecond=0)
    assert publish(c, 1, (0.5, 0.25, 0.25), (0.46, 0.26, 0.28), 0.7, {}, future) == 0
    c.execute("UPDATE f_i_matches SET source_active=false WHERE match_id=1")
    assert publish(c, 1, (0.5, 0.25, 0.25), (0.46, 0.26, 0.28), 0.7, {}, future) == 0


def rust_query(name):
    import re

    source = (
        Path(__file__).parents[2]
        / "football_insight_service_backend_rs/src/match_outlook/adapters/persistence.rs"
    ).read_text()
    return (
        re.search(r"pub const " + name + r': &str = r#"(.*?)"#;', source, re.S)
        .group(1)
        .replace("$1", "%s")
    )


@pytest.mark.parametrize(
    "p,expected_hit,improved",
    [
        ((0.5, 0.25, 0.25), 1, True),
        ((0.05, 0.05, 0.9), 0, False),
        ((0.4, 0.4, 0.2), 1, False),
    ],
)
def test_actual_rust_read_queries_return_stored_values_and_honest_record(
    connection, p, expected_hit, improved
):
    c = connection
    assert c.execute(rust_query("OUTLOOK_SQL"), (999,)).fetchone() is None
    assert c.execute(rust_query("RECORD_SQL"), (2026,)).fetchone() == (
        0,
        0,
        None,
        None,
        None,
        None,
    )
    c.execute(
        """INSERT INTO f_i_match_outlooks(match_id,home_win,draw,away_win,baseline_home,baseline_draw,baseline_away,tau,model_version,params,generated_at,kickoff_at)
        VALUES(2,%s,%s,%s,.46,.26,.28,.7,'elo-v2','{}',clock_timestamp()-interval '2h',clock_timestamp()-interval '1h')""",
        p,
    )
    assert score_finished(c) == 1
    outlook = c.execute(rust_query("OUTLOOK_SQL"), (2,)).fetchone()
    assert outlook[1:4] == p and outlook[8] is True and outlook[9] == 0
    record = c.execute(rust_query("RECORD_SQL"), (2026,)).fetchone()
    assert record[:2] == (1, expected_hit) and record[2] == expected_hit
    assert record[5] == pytest.approx(record[4] - record[3])
    assert (record[5] > 0) == improved
    c.execute("UPDATE f_i_matches SET source_active=false WHERE match_id=2")
    assert c.execute(rust_query("OUTLOOK_SQL"), (2,)).fetchone() is None
    assert c.execute(rust_query("RECORD_SQL"), (2026,)).fetchone()[0] == 0

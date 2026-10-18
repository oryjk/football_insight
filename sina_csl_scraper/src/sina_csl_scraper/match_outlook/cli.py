"""Resumable history download. Dry run by default; validate every season before writing."""

from datetime import datetime
import json
import time
from pathlib import Path
import psycopg
import typer
from .history import expected_count, game_from_dict, validate_history
from .service import refresh_match_outlooks
from ..client import SinaCslClient

app = typer.Typer(no_args_is_help=True)


def collect_history(client, years: list[int], cache: Path, interval: float):
    result = []
    for year in years:
        rows = []
        for rnd in range(1, expected_count(year) // (9 if year == 2022 else 8) + 1):
            path = cache / str(year) / f"round-{rnd}.json"
            if not path.exists():
                fetched = client.fetch_round_matches(year, rnd)
                path.parent.mkdir(parents=True, exist_ok=True)
                temporary = path.with_suffix(".tmp")
                temporary.write_text(
                    json.dumps([m.to_dict() for m in fetched], ensure_ascii=False)
                )
                temporary.replace(path)
                time.sleep(interval)
            rows.extend(game_from_dict(row) for row in json.loads(path.read_text()))
        validate_history(rows, [year])
        result.extend(rows)
    validate_history(result, years)
    return result


@app.command("backfill")
def backfill(
    start_season: int = 2014,
    end_season: int = 2025,
    cache_dir: Path = Path("data/outlook-history"),
    interval: float = typer.Option(0.3, min=0.1),
    write_db: bool = False,
    database_url: str = typer.Option("", envvar="DATABASE_URL", hidden=True),
):
    if not 2014 <= start_season <= end_season < datetime.now().year:
        raise typer.BadParameter("only completed past seasons from 2014 are supported")
    years = list(range(start_season, end_season + 1))
    rows = collect_history(SinaCslClient(), years, cache_dir, interval)
    typer.echo(
        f"Validated {len(rows)} scores across {len(years)} seasons (dry run={not write_db})"
    )
    if not write_db:
        return
    if not database_url:
        raise typer.BadParameter("DATABASE_URL required for --write-db")
    with psycopg.connect(database_url) as connection:
        with connection.cursor() as cursor:
            cursor.executemany(
                """INSERT INTO f_i_historical_matches
             (match_id,season,match_date,match_time,home_team_id,away_team_id,home_score,away_score,neutral)
             VALUES(%s,%s,%s,%s,%s,%s,%s,%s,%s) ON CONFLICT(match_id) DO UPDATE SET
             season=EXCLUDED.season,match_date=EXCLUDED.match_date,match_time=EXCLUDED.match_time,
             home_team_id=EXCLUDED.home_team_id,away_team_id=EXCLUDED.away_team_id,
             home_score=EXCLUDED.home_score,away_score=EXCLUDED.away_score,neutral=EXCLUDED.neutral""",
                [
                    (
                        g.match_id,
                        g.season,
                        g.kickoff.date(),
                        g.kickoff.time(),
                        g.home_team_id,
                        g.away_team_id,
                        g.home_score,
                        g.away_score,
                        g.neutral,
                    )
                    for g in rows
                ],
            )
    typer.echo("Historical score snapshot committed; no forecasts were backdated")


@app.command("refresh")
def refresh(
    season: int = typer.Option(..., min=2019, max=2100),
    database_url: str = typer.Option(..., envvar="DATABASE_URL", hidden=True),
):
    typer.echo(json.dumps(refresh_match_outlooks(database_url, season)))

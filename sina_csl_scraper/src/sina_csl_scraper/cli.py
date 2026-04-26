from __future__ import annotations

import json
import os
from datetime import datetime
from pathlib import Path
from typing import Any

import typer

from .auto_sync import (
    AutoSyncState,
    build_auto_sync_decision,
    load_auto_sync_state,
    save_auto_sync_state,
)
from .assets import AssetUploader, HttpAssetFetcher, MinioAssetTarget
from .catalog import build_player_profiles, build_team_profiles
from .client import SinaCslClient, serialize_datasets
from .constants import DEFAULT_LEAGUE_ID
from .leisu import LeisuBrowserClient, LeisuCornerEnricher, load_leisu_match_map
from .postgres_repository import PostgresInsightSyncRepository
from .sync import InsightSyncService, SyncPayload

app = typer.Typer(add_completion=False, no_args_is_help=True)


def _write_json(path: Path, payload: dict[str, Any] | list[dict[str, Any]]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(
        json.dumps(payload, ensure_ascii=False, indent=2),
        encoding="utf-8",
    )


@app.callback()
def root() -> None:
    """Sina CSL scraper CLI."""


def _env_default(name: str, default: str | None = None) -> str | None:
    return os.getenv(name, default)


def run_scrape(
    season: int | None = None,
    output_dir: Path = Path("data"),
    league_id: int = DEFAULT_LEAGUE_ID,
    player_page_size: int = 50,
    write_db: bool = False,
    database_url: str | None = _env_default("FI_DATABASE_URL") or _env_default("DATABASE_URL"),
    upload_avatars: bool = False,
    minio_endpoint: str = _env_default("FI_MINIO_ENDPOINT", "https://oryjk.cn:82/minio") or "https://oryjk.cn:82/minio",
    minio_access_key: str = _env_default("FI_MINIO_ACCESS_KEY", "") or "",
    minio_secret_key: str = _env_default("FI_MINIO_SECRET_KEY", "") or "",
    minio_bucket: str = _env_default("FI_MINIO_BUCKET", "football-insight") or "football-insight",
    minio_region: str = _env_default("FI_MINIO_REGION", "us-east-1") or "us-east-1",
    minio_prefix: str = _env_default("FI_MINIO_PREFIX", "summary") or "summary",
    minio_public_base_url: str | None = _env_default("FI_MINIO_PUBLIC_BASE_URL"),
    enrich_corners: bool = False,
    leisu_match_map: Path | None = Path(_env_default("FI_LEISU_MATCH_MAP")) if _env_default("FI_LEISU_MATCH_MAP") else None,
    client: SinaCslClient | None = None,
    corner_enricher: LeisuCornerEnricher | None = None,
) -> dict[str, Any]:
    client = client or SinaCslClient(league_id=league_id)
    league_info = client.fetch_league_info()
    target_season = season or league_info.current_season
    target_dir = output_dir / str(target_season)

    standings = client.fetch_standings(target_season)
    matches = client.fetch_all_matches(target_season, max_round=league_info.max_round)
    if enrich_corners:
        owns_corner_enricher = corner_enricher is None
        if corner_enricher is None:
            leisu_map = load_leisu_match_map(leisu_match_map) if leisu_match_map and leisu_match_map.exists() else {}
            corner_client = LeisuBrowserClient()
            corner_client.warmup()
            corner_enricher = LeisuCornerEnricher(
                client=corner_client,
                match_id_map=leisu_map,
            )
        try:
            matches = corner_enricher.enrich_matches(matches)
        finally:
            if owns_corner_enricher:
                corner_enricher.close()
    team_rankings = client.fetch_all_team_rankings(target_season)
    player_rankings = client.fetch_all_player_rankings(
        target_season,
        limit=player_page_size,
    )
    teams = build_team_profiles(matches, standings, team_rankings)
    players = build_player_profiles(player_rankings)

    if upload_avatars:
        if not minio_access_key or not minio_secret_key:
            raise typer.BadParameter("upload_avatars enabled but MinIO credentials are missing")

        uploader = AssetUploader(
            fetcher=HttpAssetFetcher(),
            target=MinioAssetTarget.from_credentials(
                endpoint=minio_endpoint,
                access_key=minio_access_key,
                secret_key=minio_secret_key,
                bucket=minio_bucket,
                region=minio_region,
                public_base_url=minio_public_base_url,
            ),
        )
        teams = uploader.upload_team_avatars(teams, prefix=minio_prefix)
        players = uploader.upload_player_avatars(players, prefix=minio_prefix)

    sync_result = None
    if write_db:
        if not database_url:
            raise typer.BadParameter("write_db enabled but PostgreSQL database_url is missing")

        repository = PostgresInsightSyncRepository(database_url=database_url)
        try:
            sync_service = InsightSyncService(repository=repository)
            sync_result = sync_service.sync(
                SyncPayload(
                    season=target_season,
                    current_round=league_info.current_round,
                    teams=teams,
                    players=players,
                    matches=matches,
                    standings=standings,
                    team_rankings=team_rankings,
                    player_rankings=player_rankings,
                )
            )
        finally:
            repository.close()

    _write_json(
        target_dir / "league_info.json",
        {
            "league": league_info.to_dict(),
            "season": target_season,
        },
    )
    _write_json(
        target_dir / "standings.json",
        [entry.to_dict() for entry in standings],
    )
    _write_json(
        target_dir / "matches.json",
        [entry.to_dict() for entry in matches],
    )
    _write_json(
        target_dir / "team_rankings.json",
        serialize_datasets(team_rankings),
    )
    _write_json(
        target_dir / "player_rankings.json",
        serialize_datasets(player_rankings),
    )
    _write_json(
        target_dir / "teams.json",
        [entry.to_dict() for entry in teams],
    )
    _write_json(
        target_dir / "players.json",
        [entry.to_dict() for entry in players],
    )

    typer.echo(f"Scraped season {target_season} into {target_dir}")
    typer.echo(f"Matches: {len(matches)}")
    typer.echo(f"Standings teams: {len(standings)}")
    typer.echo(f"Team profiles: {len(teams)}")
    typer.echo(f"Player profiles: {len(players)}")
    typer.echo(f"Team ranking categories: {len(team_rankings)}")
    typer.echo(f"Player ranking categories: {len(player_rankings)}")
    if sync_result is not None:
        typer.echo(f"PostgreSQL run id: {sync_result.run_id}")
        typer.echo(
            "PostgreSQL sync: "
            f"teams={sync_result.teams_upserted}, "
            f"players={sync_result.players_upserted}, "
            f"matches={sync_result.matches_upserted}, "
            f"live_standings={sync_result.live_standings_inserted}, "
            f"live_team_rankings={sync_result.live_team_ranking_categories}, "
            f"live_player_rankings={sync_result.live_player_ranking_categories}, "
            f"round_final={sync_result.round_finalized_for}, "
            f"round_final_standings={sync_result.round_final_standings_inserted}, "
            f"round_final_team_rankings={sync_result.round_final_team_ranking_categories}, "
            f"round_final_player_rankings={sync_result.round_final_player_ranking_categories}"
        )

    return {
        "season": target_season,
        "run_id": sync_result.run_id if sync_result is not None else None,
        "matches": len(matches),
    }


@app.command()
def scrape(
    season: int | None = typer.Option(None, help="Season year. Defaults to the current season."),
    output_dir: Path = typer.Option(
        Path("data"),
        "--output-dir",
        "-o",
        help="Base output directory for exported JSON files.",
    ),
    league_id: int = typer.Option(DEFAULT_LEAGUE_ID, help="Sina league id, default is CSL."),
    player_page_size: int = typer.Option(
        50,
        min=1,
        max=100,
        help="Page size for player rankings.",
    ),
    write_db: bool = typer.Option(
        False,
        "--write-db",
        help="Write normalized data into PostgreSQL.",
    ),
    database_url: str | None = typer.Option(
        _env_default("FI_DATABASE_URL") or _env_default("DATABASE_URL"),
        help="PostgreSQL connection string used when --write-db is enabled.",
    ),
    upload_avatars: bool = typer.Option(
        False,
        "--upload-avatars",
        help="Download team/player avatars from Sina and upload them to MinIO.",
    ),
    minio_endpoint: str = typer.Option(
        _env_default("FI_MINIO_ENDPOINT", "https://oryjk.cn:82/minio"),
        help="MinIO S3 endpoint.",
    ),
    minio_access_key: str = typer.Option(
        _env_default("FI_MINIO_ACCESS_KEY", ""),
        help="MinIO access key.",
    ),
    minio_secret_key: str = typer.Option(
        _env_default("FI_MINIO_SECRET_KEY", ""),
        help="MinIO secret key.",
    ),
    minio_bucket: str = typer.Option(
        _env_default("FI_MINIO_BUCKET", "football-insight"),
        help="MinIO bucket name.",
    ),
    minio_region: str = typer.Option(
        _env_default("FI_MINIO_REGION", "us-east-1"),
        help="MinIO region name.",
    ),
    minio_prefix: str = typer.Option(
        _env_default("FI_MINIO_PREFIX", "summary"),
        help="Object prefix inside the MinIO bucket.",
    ),
    minio_public_base_url: str | None = typer.Option(
        _env_default("FI_MINIO_PUBLIC_BASE_URL"),
        help="Public URL prefix returned in exported JSON. Defaults to endpoint + bucket.",
    ),
    enrich_corners: bool = typer.Option(
        False,
        "--enrich-corners",
        help="Best-effort enrich Leisu match technical stats, including corners.",
    ),
    leisu_match_map: Path | None = typer.Option(
        Path(_env_default("FI_LEISU_MATCH_MAP")) if _env_default("FI_LEISU_MATCH_MAP") else None,
        "--leisu-match-map",
        help="JSON file mapping Sina match_id to Leisu detail id.",
    ),
) -> None:
    run_scrape(
        season=season,
        output_dir=output_dir,
        league_id=league_id,
        player_page_size=player_page_size,
        write_db=write_db,
        database_url=database_url,
        upload_avatars=upload_avatars,
        minio_endpoint=minio_endpoint,
        minio_access_key=minio_access_key,
        minio_secret_key=minio_secret_key,
        minio_bucket=minio_bucket,
        minio_region=minio_region,
        minio_prefix=minio_prefix,
        minio_public_base_url=minio_public_base_url,
        enrich_corners=enrich_corners,
        leisu_match_map=leisu_match_map,
    )


@app.command("auto-sync-due")
def auto_sync_due(
    season: int | None = typer.Option(None, help="Season year. Defaults to the current season."),
    state_file: Path = typer.Option(
        Path(".auto_sync_state.json"),
        "--state-file",
        help="Persistent state file used to avoid duplicate post-match syncs.",
    ),
    output_dir: Path = typer.Option(
        Path("data"),
        "--output-dir",
        "-o",
        help="Base output directory for exported JSON files.",
    ),
    league_id: int = typer.Option(DEFAULT_LEAGUE_ID, help="Sina league id, default is CSL."),
    player_page_size: int = typer.Option(
        50,
        min=1,
        max=100,
        help="Page size for player rankings.",
    ),
    match_duration_minutes: int = typer.Option(
        120,
        "--match-duration-minutes",
        min=1,
        help="Assumed match duration used to derive the finished time from kickoff time.",
    ),
    post_finish_delay_minutes: int = typer.Option(
        10,
        "--post-finish-delay-minutes",
        min=0,
        help="Minutes to wait after the assumed finished time before syncing.",
    ),
    active_sync_interval_minutes: int = typer.Option(
        1,
        "--active-sync-interval-minutes",
        min=1,
        help="While matches are in progress, minimum interval between live refresh syncs.",
    ),
    write_db: bool = typer.Option(
        False,
        "--write-db",
        help="Write normalized data into PostgreSQL when the sync window is due.",
    ),
    database_url: str | None = typer.Option(
        _env_default("FI_DATABASE_URL") or _env_default("DATABASE_URL"),
        help="PostgreSQL connection string used when --write-db is enabled.",
    ),
    upload_avatars: bool = typer.Option(
        False,
        "--upload-avatars",
        help="Download team/player avatars from Sina and upload them to MinIO when due.",
    ),
    minio_endpoint: str = typer.Option(
        _env_default("FI_MINIO_ENDPOINT", "https://oryjk.cn:82/minio"),
        help="MinIO S3 endpoint.",
    ),
    minio_access_key: str = typer.Option(
        _env_default("FI_MINIO_ACCESS_KEY", ""),
        help="MinIO access key.",
    ),
    minio_secret_key: str = typer.Option(
        _env_default("FI_MINIO_SECRET_KEY", ""),
        help="MinIO secret key.",
    ),
    minio_bucket: str = typer.Option(
        _env_default("FI_MINIO_BUCKET", "football-insight"),
        help="MinIO bucket name.",
    ),
    minio_region: str = typer.Option(
        _env_default("FI_MINIO_REGION", "us-east-1"),
        help="MinIO region name.",
    ),
    minio_prefix: str = typer.Option(
        _env_default("FI_MINIO_PREFIX", "summary"),
        help="Object prefix inside the MinIO bucket.",
    ),
    minio_public_base_url: str | None = typer.Option(
        _env_default("FI_MINIO_PUBLIC_BASE_URL"),
        help="Public URL prefix returned in exported JSON. Defaults to endpoint + bucket.",
    ),
    enrich_corners: bool = typer.Option(
        False,
        "--enrich-corners",
        help="Best-effort enrich Leisu match technical stats, including corners, when due.",
    ),
    leisu_match_map: Path | None = typer.Option(
        Path(_env_default("FI_LEISU_MATCH_MAP")) if _env_default("FI_LEISU_MATCH_MAP") else None,
        "--leisu-match-map",
        help="JSON file mapping Sina match_id to Leisu detail id.",
    ),
    dry_run: bool = typer.Option(
        False,
        "--dry-run",
        help="Only print the decision without running the scraper.",
    ),
) -> None:
    client = SinaCslClient(league_id=league_id)
    league_info = client.fetch_league_info()
    target_season = season or league_info.current_season
    matches = client.fetch_all_matches(target_season, max_round=league_info.max_round)
    state = load_auto_sync_state(state_file)
    now = datetime.now().astimezone()
    decision = build_auto_sync_decision(
        matches,
        now=now,
        state=state,
        match_duration_minutes=match_duration_minutes,
        post_finish_delay_minutes=post_finish_delay_minutes,
        active_sync_interval_minutes=active_sync_interval_minutes,
    )

    if not decision.should_run:
        latest_completed_due_at = decision.latest_due_at.isoformat() if decision.latest_due_at else "none"
        typer.echo(
            f"No auto sync needed. latest_completed_due_at={latest_completed_due_at} "
            f"last_processed_completed_due_at={state.last_processed_due_at.isoformat() if state.last_processed_due_at else 'none'} "
            f"active_match_ids={','.join(str(item) for item in decision.active_match_ids) if decision.active_match_ids else 'none'}"
        )
        return

    trigger_parts: list[str] = []
    if decision.newly_due_match_ids:
        trigger_parts.append(
            "completed_due_match_ids="
            + ",".join(str(item) for item in decision.newly_due_match_ids)
        )
    if decision.active_match_ids:
        trigger_parts.append(
            "active_match_ids="
            + ",".join(str(item) for item in decision.active_match_ids)
        )
    latest_completed_due_at_text = decision.latest_due_at.isoformat() if decision.latest_due_at else "none"
    typer.echo(
        "Auto sync triggered for "
        + " ".join(trigger_parts)
        + f" latest_completed_due_at={latest_completed_due_at_text}"
    )

    if dry_run:
        return

    run_scrape(
        season=target_season,
        output_dir=output_dir,
        league_id=league_id,
        player_page_size=player_page_size,
        write_db=write_db,
        database_url=database_url,
        upload_avatars=upload_avatars,
        minio_endpoint=minio_endpoint,
        minio_access_key=minio_access_key,
        minio_secret_key=minio_secret_key,
        minio_bucket=minio_bucket,
        minio_region=minio_region,
        minio_prefix=minio_prefix,
        minio_public_base_url=minio_public_base_url,
        enrich_corners=enrich_corners,
        leisu_match_map=leisu_match_map,
    )

    save_auto_sync_state(
        state_file,
        AutoSyncState(
            last_processed_due_at=decision.latest_due_at or state.last_processed_due_at,
            last_run_at=now,
            last_run_match_ids=decision.active_match_ids or decision.newly_due_match_ids,
        ),
    )


def main() -> None:
    app()

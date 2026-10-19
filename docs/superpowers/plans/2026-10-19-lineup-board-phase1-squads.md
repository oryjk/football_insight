# 战术板 阶段一：球队名单数据 实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 抓取器同步 16 支中超球队的完整名单（含往季前队员）到 `f_i_squad_players`，后端提供 `GET /api/v1/lineups/squads/{team_id}` 只读接口。

**Architecture:** 抓取器新增 `sina-csl-scraper sync-squads` 命令：新浪 `/op/api/team/player` → 头像转存 MinIO → 按"每队独立成败"规则 upsert。后端新建 `lineup` 模块（domain/ports/application/adapters + bootstrap），本阶段只实现名单读取；分组与排序规则放在 application 层。

**Tech Stack:** Python 3.13 + uv + psycopg3 + Typer + pytest；Rust + Axum + SQLx + tokio test。

**Spec:** `docs/superpowers/specs/2026-10-19-lineup-board-design.md`（第 2 节、3.3 名单接口、第 7 节阶段一）

## Global Constraints

- 抓取器负责写库，后端只读 `f_i_squad_players`（沿用项目职责边界）。
- 迁移文件放 `football_insight_service_backend_rs/migrations/`，时间戳 `20261019120000`（大于现有最新 `20261018140000`）；迁移不随部署自动执行，**在生产执行迁移与首次写库需负责人确认**（Task 6）。
- 每队独立成败：请求失败、结构异常或返回空名单 → 跳过该队，不改它的任何数据；只有当前赛季成功拿到非空名单才把缺席者置 `active=false`。
- 位置映射：守门员→`GK`、后卫→`DF`、中场→`MF`、前锋→`FW`，其他→空。
- 头像对象名 `{prefix}/players/{pid}{ext}`（与现有上传器一致）；已有 `avatar_storage_url` 的不重复上传；单个头像失败不影响该队写库。
- 后端代码改动后，本地验证前先重启本地后端（`./deploy/local-test/up.sh`）；汇报区分本地已重启 / 生产未部署 / 生产已部署。
- 本地后端连生产库：只做只读验证，不写测试数据。
- 提交：`feat(scraper): ...`、`feat(backend): ...`、`docs: ...`，subject 英文，末尾 `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`；提交并推送 `main`，提交前 `git pull --rebase`。
- 验证命令：抓取器 `cd sina_csl_scraper && uv run pytest`；后端 `cd football_insight_service_backend_rs && cargo test`。

## Review Focus

1. **新浪返回 `code=0` 但 `players` 为空或缺字段**：不得把该队所有球员标成离队。→ Task 3 测试 `test_empty_current_squad_is_a_failure_and_changes_nothing`。
2. **回填赛季比当前赛季更晚写入**：回填不得覆盖当前赛季的号码、位置或 active。→ Task 3 测试 `test_backfill_never_overrides_newer_season_or_active` 与 Task 2 postgres 测试。
3. **同一球员同时出现在两队**（赛季中转会到另一支中超队）：两队各一行，互不覆盖。→ Task 2 postgres 测试 `test_same_player_in_two_teams_keeps_two_rows`。
4. **表尚未迁移时请求名单接口**：后端返回 503 而不是 500 或 panic。→ Task 5 测试 `missing_table_is_unavailable`（持久层 42P01 → Unavailable）与路由测试。
5. **号码为空或非数字、位置为空的球员**：排序不 panic，号码空排在同位置最后。→ Task 4 测试 `sorts_active_by_position_then_number_with_blanks_last`。

---

## 文件结构

| 文件 | 动作 | 职责 |
|---|---|---|
| `football_insight_service_backend_rs/migrations/20261019120000_add_squad_players.sql` | 新增 | 建表与索引 |
| `sina_csl_scraper/src/sina_csl_scraper/models.py` | 改 | 新增 `SquadPlayer` |
| `sina_csl_scraper/src/sina_csl_scraper/client.py` | 改 | 新增 `fetch_team_squad`、`parse_squad_player`、`map_squad_position` |
| `sina_csl_scraper/src/sina_csl_scraper/squad_sync.py` | 新增 | `SquadStore` 协议、`PostgresSquadStore`、`sync_squads` 编排、头像转存 |
| `sina_csl_scraper/src/sina_csl_scraper/cli.py` | 改 | `sync-squads` 命令 |
| `sina_csl_scraper/sync_squads_daily.sh`、`deploy/squad-sync.cron.example` | 新增 | 每日调度 |
| `sina_csl_scraper/tests/test_squad_client.py`、`test_squad_sync.py`、`test_squad_postgres.py` | 新增 | 测试 |
| `football_insight_service_backend_rs/src/lineup/{mod,domain,ports,application,bootstrap}.rs`、`adapters/{mod,persistence,web}.rs` | 新增 | 名单读取 |
| `football_insight_service_backend_rs/src/lib.rs`、`src/app.rs` | 改 | 注册模块与路由 |
| `CHANGELOG.md`、`FEATURES.md`、`DEPLOYMENT.md` | 改 | 文档 |

---

### Task 1: 迁移 `f_i_squad_players`

**Files:**
- Create: `football_insight_service_backend_rs/migrations/20261019120000_add_squad_players.sql`

**Interfaces:**
- Produces: 表 `f_i_squad_players(team_id, player_id, player_name, shirt_number, position, country, avatar_source_url, avatar_storage_url, last_season, active, synced_at)`，主键 `(team_id, player_id)`，索引 `idx_f_i_squad_players_team_active`。

- [ ] **Step 1: 写迁移**

```sql
-- 球队完整名单（含往季前队员），由 sina_csl_scraper sync-squads 写入，后端 lineup 模块只读。
CREATE TABLE IF NOT EXISTS f_i_squad_players (
    team_id BIGINT NOT NULL,
    player_id BIGINT NOT NULL,
    player_name TEXT NOT NULL,
    shirt_number INTEGER,
    position TEXT CHECK (position IN ('GK', 'DF', 'MF', 'FW')),
    country TEXT,
    avatar_source_url TEXT,
    avatar_storage_url TEXT,
    last_season INTEGER NOT NULL,
    active BOOLEAN NOT NULL DEFAULT FALSE,
    synced_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (team_id, player_id)
);

CREATE INDEX IF NOT EXISTS idx_f_i_squad_players_team_active
    ON f_i_squad_players (team_id, active);
```

- [ ] **Step 2: 确认后端迁移嵌入不报错**

Run: `cd football_insight_service_backend_rs && cargo build 2>&1 | tail -3`
Expected: 编译成功（`sqlx::migrate!` 若嵌入迁移目录，新文件不会破坏编译）。

- [ ] **Step 3: 提交**

```bash
git pull --rebase
git add football_insight_service_backend_rs/migrations/20261019120000_add_squad_players.sql
git commit -m "feat(backend): add squad players table migration

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
git push
```

---

### Task 2: 抓取器 —— 新浪名单解析

**Files:**
- Modify: `sina_csl_scraper/src/sina_csl_scraper/models.py`
- Modify: `sina_csl_scraper/src/sina_csl_scraper/client.py`
- Test: `sina_csl_scraper/tests/test_squad_client.py`

**Interfaces:**
- Produces:

```python
@dataclass(frozen=True)
class SquadPlayer:
    team_id: int
    player_id: int
    player_name: str
    shirt_number: int | None
    position: str | None          # 'GK' | 'DF' | 'MF' | 'FW' | None
    country: str | None
    avatar_source_url: str | None

def map_squad_position(raw: str | None) -> str | None
def parse_squad_player(team_id: int, item: dict) -> SquadPlayer | None   # 缺 pid 或缺名字返回 None
class SinaCslClient:
    def fetch_team_squad(self, team_id: int, season: int | None = None) -> list[SquadPlayer]
```

- [ ] **Step 1: 写失败测试**

```python
# sina_csl_scraper/tests/test_squad_client.py
from __future__ import annotations

from sina_csl_scraper.client import SinaCslClient, map_squad_position, parse_squad_player


class FakeResponse:
    def __init__(self, payload):
        self.payload = payload

    def raise_for_status(self) -> None:
        return None

    def json(self):
        return self.payload


class FakeSession:
    def __init__(self, players):
        self.headers = {}
        self.calls = []
        self.players = players

    def get(self, url, params, timeout):
        self.calls.append((url, params))
        return FakeResponse({"result": {"status": {"code": 0, "msg": "team players"}, "data": {"players": self.players}}})


def test_maps_chinese_positions():
    assert map_squad_position("守门员") == "GK"
    assert map_squad_position("后卫") == "DF"
    assert map_squad_position("中场") == "MF"
    assert map_squad_position("前锋") == "FW"
    assert map_squad_position("教练") is None
    assert map_squad_position(None) is None


def test_parses_player_fields_and_tolerates_bad_numbers():
    player = parse_squad_player(77680, {
        "pid": "156047", "name": "Feng Zhuoyi", "name_cn": "冯卓毅", "number": "6",
        "country": "中国", "position": "中场", "logo": "http://n.sinaimg.cn/ty/opta/player150/156047.png",
    })
    assert player is not None
    assert (player.team_id, player.player_id, player.player_name, player.shirt_number, player.position) == (
        77680, 156047, "冯卓毅", 6, "MF")
    assert player.avatar_source_url == "http://n.sinaimg.cn/ty/opta/player150/156047.png"
    assert parse_squad_player(1, {"pid": "9", "name": "X", "number": "", "position": ""}).shirt_number is None
    assert parse_squad_player(1, {"pid": "9", "name": "X", "number": "1a"}).shirt_number is None
    assert parse_squad_player(1, {"pid": "9", "name_cn": "", "name": "Only English"}).player_name == "Only English"


def test_skips_items_without_pid_or_name():
    assert parse_squad_player(1, {"name_cn": "无编号"}) is None
    assert parse_squad_player(1, {"pid": "3", "name": "", "name_cn": ""}) is None


def test_fetch_team_squad_passes_team_and_season():
    session = FakeSession([{"pid": "1", "name_cn": "甲", "number": "1", "position": "守门员"}, {"name_cn": "无pid"}])
    client = SinaCslClient(session=session)
    players = client.fetch_team_squad(77680, season=2025)
    assert [p.player_id for p in players] == [1]
    url, params = session.calls[0]
    assert url.endswith("/op/api/team/player")
    assert params["tid"] == 77680 and params["season"] == 2025


def test_fetch_team_squad_without_season_omits_param():
    session = FakeSession([])
    SinaCslClient(session=session).fetch_team_squad(77680)
    assert "season" not in session.calls[0][1]
```

- [ ] **Step 2: 确认失败**

Run: `cd sina_csl_scraper && uv run pytest tests/test_squad_client.py -q`
Expected: FAIL（ImportError: `map_squad_position`）。

- [ ] **Step 3: 实现**

`models.py` 末尾追加：

```python
@dataclass(frozen=True)
class SquadPlayer:
    team_id: int
    player_id: int
    player_name: str
    shirt_number: int | None
    position: str | None
    country: str | None
    avatar_source_url: str | None

    def to_dict(self) -> dict[str, Any]:
        return asdict(self)
```

`client.py`：import 增加 `SquadPlayer`，在 `SinaApiError` 之后加入：

```python
_SQUAD_POSITIONS = {"守门员": "GK", "后卫": "DF", "中场": "MF", "前锋": "FW"}


def map_squad_position(raw: str | None) -> str | None:
    return _SQUAD_POSITIONS.get((raw or "").strip())


def parse_squad_player(team_id: int, item: dict[str, Any]) -> SquadPlayer | None:
    pid = str(item.get("pid") or "").strip()
    name = str(item.get("name_cn") or "").strip() or str(item.get("name") or "").strip()
    if not pid.isdigit() or not name:
        return None
    number = str(item.get("number") or "").strip()
    logo = str(item.get("logo") or "").strip()
    country = str(item.get("country") or "").strip()
    return SquadPlayer(
        team_id=team_id,
        player_id=int(pid),
        player_name=name,
        shirt_number=int(number) if number.isdigit() else None,
        position=map_squad_position(item.get("position")),
        country=country or None,
        avatar_source_url=logo or None,
    )
```

在 `SinaCslClient` 中加入方法：

```python
    def fetch_team_squad(self, team_id: int, season: int | None = None) -> list[SquadPlayer]:
        params: dict[str, Any] = {"tid": team_id}
        if season is not None:
            params["season"] = season
        data = self._request("/op/api/team/player", params)
        items = data.get("players") if isinstance(data, dict) else None
        players = [parse_squad_player(team_id, item) for item in items or [] if isinstance(item, dict)]
        return [player for player in players if player is not None]
```

- [ ] **Step 4: 确认通过**

Run: `cd sina_csl_scraper && uv run pytest tests/test_squad_client.py -q`
Expected: 5 passed。

- [ ] **Step 5: 提交**

```bash
git pull --rebase
git add sina_csl_scraper/src/sina_csl_scraper/models.py sina_csl_scraper/src/sina_csl_scraper/client.py sina_csl_scraper/tests/test_squad_client.py
git commit -m "feat(scraper): parse Sina team squads with numbers and positions

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
git push
```

---

### Task 3: 抓取器 —— 同步编排、头像转存与写库

**Files:**
- Create: `sina_csl_scraper/src/sina_csl_scraper/squad_sync.py`
- Test: `sina_csl_scraper/tests/test_squad_sync.py`、`sina_csl_scraper/tests/test_squad_postgres.py`

**Interfaces:**
- Consumes: `SquadPlayer`、`SinaCslClient.fetch_team_squad`（Task 2）；`assets.AssetFetcher`、`assets.AssetTarget`、`assets._extension_for`。
- Produces:

```python
class SquadStore(Protocol):
    def team_ids(self) -> list[int]: ...
    def avatar_urls(self, team_id: int) -> dict[int, str]: ...
    def apply(self, team_id: int, season: int, rows: list[dict], current: bool) -> None: ...

class PostgresSquadStore:  # 实现 SquadStore
    def __init__(self, connection) -> None

def build_squad_rows(players: list[SquadPlayer], season: int, storage_urls: dict[int, str]) -> list[dict]
def sync_squads(*, fetch: Callable[[int, int | None], list[SquadPlayer]], store: SquadStore,
                season: int, backfill_seasons: int,
                avatars: Callable[[SquadPlayer], str | None] | None = None) -> dict
def make_avatar_uploader(fetcher: AssetFetcher, target: AssetTarget, prefix: str) -> Callable[[SquadPlayer], str | None]
```

`sync_squads` 返回 `{"teams_ok": int, "teams_failed": list[int], "players_current": int, "avatars_uploaded": int}`。

- [ ] **Step 1: 写失败的编排测试（内存 store）**

```python
# sina_csl_scraper/tests/test_squad_sync.py
from __future__ import annotations

from sina_csl_scraper.models import SquadPlayer
from sina_csl_scraper.squad_sync import build_squad_rows, sync_squads


def player(pid, team=1, number=None, position="MF", logo=None):
    return SquadPlayer(team, pid, f"P{pid}", number, position, None, logo)


class MemoryStore:
    def __init__(self, teams, avatar_urls=None):
        self.teams = teams
        self.urls = avatar_urls or {}
        self.applied = []

    def team_ids(self):
        return list(self.teams)

    def avatar_urls(self, team_id):
        return dict(self.urls.get(team_id, {}))

    def apply(self, team_id, season, rows, current):
        self.applied.append((team_id, season, [r["player_id"] for r in rows], current))


def test_build_rows_keeps_existing_storage_url():
    rows = build_squad_rows([player(1), player(2)], 2026, {1: "https://minio/p1.png"})
    assert rows[0]["avatar_storage_url"] == "https://minio/p1.png"
    assert rows[1]["avatar_storage_url"] is None
    assert rows[0]["season"] == 2026


def test_history_seasons_apply_before_current_and_current_is_marked():
    calls = []
    def fetch(team_id, season):
        calls.append(season)
        return [player(season or 2026)]
    store = MemoryStore([7])
    summary = sync_squads(fetch=fetch, store=store, season=2026, backfill_seasons=2)
    assert calls == [2024, 2025, 2026]
    assert [(s, current) for _, s, _, current in store.applied] == [(2024, False), (2025, False), (2026, True)]
    assert summary["teams_ok"] == 1 and summary["teams_failed"] == []


def test_empty_current_squad_is_a_failure_and_changes_nothing():
    store = MemoryStore([7])
    summary = sync_squads(fetch=lambda t, s: [], store=store, season=2026, backfill_seasons=0)
    assert store.applied == []
    assert summary["teams_failed"] == [7]


def test_one_team_failing_does_not_stop_others():
    def fetch(team_id, season):
        if team_id == 1:
            raise RuntimeError("boom")
        return [player(5, team=team_id)]
    store = MemoryStore([1, 2])
    summary = sync_squads(fetch=fetch, store=store, season=2026, backfill_seasons=0)
    assert summary["teams_failed"] == [1]
    assert [t for t, *_ in store.applied] == [2]


def test_backfill_failure_still_applies_current_season():
    def fetch(team_id, season):
        if season == 2025:
            raise RuntimeError("history down")
        return [player(9)]
    store = MemoryStore([3])
    summary = sync_squads(fetch=fetch, store=store, season=2026, backfill_seasons=1)
    assert [(s, c) for _, s, _, c in store.applied] == [(2026, True)]
    assert summary["teams_ok"] == 1


def test_backfill_never_overrides_newer_season_or_active():
    # 回填只以 current=False 写入；"不覆盖更新赛季"由 SQL 保证（见 postgres 测试）
    store = MemoryStore([3])
    sync_squads(fetch=lambda t, s: [player(1)], store=store, season=2026, backfill_seasons=1)
    assert [c for *_, c in store.applied] == [False, True]


def test_avatars_upload_only_when_missing_and_failures_are_tolerated():
    uploaded = []
    def avatars(p):
        uploaded.append(p.player_id)
        if p.player_id == 3:
            raise RuntimeError("cdn down")
        return f"https://minio/{p.player_id}.png"
    store = MemoryStore([1], avatar_urls={1: {1: "https://minio/1.png"}})
    fetched = [player(1, logo="http://a/1.png"), player(2, logo="http://a/2.png"), player(3, logo="http://a/3.png"), player(4)]
    summary = sync_squads(fetch=lambda t, s: fetched, store=store, season=2026, backfill_seasons=0, avatars=avatars)
    assert uploaded == [2, 3]          # 1 已有、4 无源图
    assert summary["avatars_uploaded"] == 1
    assert len(store.applied[0][2]) == 4   # 头像失败不影响写库
```

- [ ] **Step 2: 确认失败**

Run: `cd sina_csl_scraper && uv run pytest tests/test_squad_sync.py -q`
Expected: FAIL（ModuleNotFoundError: `sina_csl_scraper.squad_sync`）。

- [ ] **Step 3: 实现 `squad_sync.py`**

```python
"""Sync full team squads (current + past seasons) into f_i_squad_players.

Each team succeeds or fails on its own: a failed or empty current-season fetch leaves that
team's rows untouched, so a provider hiccup can never mark a whole squad as departed.
"""
from __future__ import annotations

import sys
from collections.abc import Callable
from typing import Any, Protocol

from .assets import AssetFetcher, AssetTarget, _extension_for
from .models import SquadPlayer

UPSERT_CURRENT_SQL = """
INSERT INTO f_i_squad_players
    (team_id, player_id, player_name, shirt_number, position, country,
     avatar_source_url, avatar_storage_url, last_season, active, synced_at)
VALUES (%(team_id)s, %(player_id)s, %(player_name)s, %(shirt_number)s, %(position)s, %(country)s,
        %(avatar_source_url)s, %(avatar_storage_url)s, %(season)s, TRUE, NOW())
ON CONFLICT (team_id, player_id) DO UPDATE SET
    player_name = EXCLUDED.player_name,
    shirt_number = EXCLUDED.shirt_number,
    position = EXCLUDED.position,
    country = EXCLUDED.country,
    avatar_source_url = EXCLUDED.avatar_source_url,
    avatar_storage_url = COALESCE(EXCLUDED.avatar_storage_url, f_i_squad_players.avatar_storage_url),
    last_season = GREATEST(f_i_squad_players.last_season, EXCLUDED.last_season),
    active = TRUE,
    synced_at = NOW()
"""

UPSERT_HISTORY_SQL = """
INSERT INTO f_i_squad_players
    (team_id, player_id, player_name, shirt_number, position, country,
     avatar_source_url, avatar_storage_url, last_season, active, synced_at)
VALUES (%(team_id)s, %(player_id)s, %(player_name)s, %(shirt_number)s, %(position)s, %(country)s,
        %(avatar_source_url)s, %(avatar_storage_url)s, %(season)s, FALSE, NOW())
ON CONFLICT (team_id, player_id) DO UPDATE SET
    player_name = EXCLUDED.player_name,
    shirt_number = EXCLUDED.shirt_number,
    position = EXCLUDED.position,
    country = EXCLUDED.country,
    avatar_source_url = EXCLUDED.avatar_source_url,
    avatar_storage_url = COALESCE(EXCLUDED.avatar_storage_url, f_i_squad_players.avatar_storage_url),
    last_season = EXCLUDED.last_season,
    synced_at = NOW()
WHERE f_i_squad_players.last_season < EXCLUDED.last_season
"""

DEACTIVATE_MISSING_SQL = """
UPDATE f_i_squad_players SET active = FALSE, synced_at = NOW()
WHERE team_id = %s AND active AND NOT (player_id = ANY(%s))
"""


class SquadStore(Protocol):
    def team_ids(self) -> list[int]: ...
    def avatar_urls(self, team_id: int) -> dict[int, str]: ...
    def apply(self, team_id: int, season: int, rows: list[dict[str, Any]], current: bool) -> None: ...


class PostgresSquadStore:
    def __init__(self, connection) -> None:
        self.connection = connection

    def team_ids(self) -> list[int]:
        return [row[0] for row in self.connection.execute("SELECT team_id FROM f_i_teams ORDER BY team_id").fetchall()]

    def avatar_urls(self, team_id: int) -> dict[int, str]:
        rows = self.connection.execute(
            "SELECT player_id, avatar_storage_url FROM f_i_squad_players WHERE team_id=%s AND avatar_storage_url IS NOT NULL",
            (team_id,),
        ).fetchall()
        return {int(pid): url for pid, url in rows}

    def apply(self, team_id: int, season: int, rows: list[dict[str, Any]], current: bool) -> None:
        if not rows:
            raise ValueError("refusing to apply an empty squad")
        with self.connection.transaction():
            with self.connection.cursor() as cursor:
                cursor.executemany(UPSERT_CURRENT_SQL if current else UPSERT_HISTORY_SQL, rows)
                if current:
                    cursor.execute(DEACTIVATE_MISSING_SQL, (team_id, [row["player_id"] for row in rows]))


def build_squad_rows(players: list[SquadPlayer], season: int, storage_urls: dict[int, str]) -> list[dict[str, Any]]:
    return [
        {**player.to_dict(), "season": season, "avatar_storage_url": storage_urls.get(player.player_id)}
        for player in players
    ]


def make_avatar_uploader(fetcher: AssetFetcher, target: AssetTarget, prefix: str) -> Callable[[SquadPlayer], str | None]:
    def upload(player: SquadPlayer) -> str | None:
        if not player.avatar_source_url:
            return None
        content, content_type = fetcher.fetch(player.avatar_source_url)
        object_name = f"{prefix.strip('/')}/players/{player.player_id}{_extension_for(player.avatar_source_url, content_type)}"
        return target.upload_bytes(object_name, content, content_type)
    return upload


def _with_avatars(players: list[SquadPlayer], known: dict[int, str],
                  avatars: Callable[[SquadPlayer], str | None] | None, counter: list[int]) -> dict[int, str]:
    urls = dict(known)
    if avatars is None:
        return urls
    for player in players:
        if player.player_id in urls or not player.avatar_source_url:
            continue
        try:
            url = avatars(player)
        except Exception as error:  # 单个头像失败不影响该队写库
            print(f"[squads] avatar {player.player_id} failed: {type(error).__name__}", file=sys.stderr)
            continue
        if url:
            urls[player.player_id] = url
            counter[0] += 1
    return urls


def sync_squads(*, fetch: Callable[[int, int | None], list[SquadPlayer]], store: SquadStore,
                season: int, backfill_seasons: int,
                avatars: Callable[[SquadPlayer], str | None] | None = None) -> dict[str, Any]:
    summary: dict[str, Any] = {"teams_ok": 0, "teams_failed": [], "players_current": 0, "avatars_uploaded": 0}
    uploaded = [0]
    history = [season - offset for offset in range(max(backfill_seasons, 0), 0, -1)]
    for team_id in store.team_ids():
        known = store.avatar_urls(team_id)
        for past in history:
            try:
                players = fetch(team_id, past)
                if players:
                    known = _with_avatars(players, known, avatars, uploaded)
                    store.apply(team_id, past, build_squad_rows(players, past, known), current=False)
            except Exception as error:
                print(f"[squads] team {team_id} season {past} skipped: {type(error).__name__}", file=sys.stderr)
        try:
            players = fetch(team_id, season)
            if not players:
                raise ValueError("empty current squad")
            known = _with_avatars(players, known, avatars, uploaded)
            store.apply(team_id, season, build_squad_rows(players, season, known), current=True)
            summary["teams_ok"] += 1
            summary["players_current"] += len(players)
        except Exception as error:
            print(f"[squads] team {team_id} current season failed: {type(error).__name__}", file=sys.stderr)
            summary["teams_failed"].append(team_id)
    summary["avatars_uploaded"] = uploaded[0]
    return summary
```

注：`test_history_seasons_apply_before_current_and_current_is_marked` 里 `fetch(team_id, season)` 的第二个参数是具体赛季；当前赛季也显式传 `season`（不用 None），这样新浪返回的一定是指定赛季。

- [ ] **Step 4: 确认编排测试通过**

Run: `cd sina_csl_scraper && uv run pytest tests/test_squad_sync.py -q`
Expected: 7 passed。

- [ ] **Step 5: 写 postgres 测试（一次性本地库，未配置则跳过）**

```python
# sina_csl_scraper/tests/test_squad_postgres.py
import os
from pathlib import Path
from urllib.parse import urlsplit

import psycopg
import pytest

from sina_csl_scraper.models import SquadPlayer
from sina_csl_scraper.squad_sync import PostgresSquadStore, build_squad_rows


@pytest.fixture
def store():
    url = os.getenv('SQUAD_TEST_DATABASE_URL')
    if not url:
        pytest.skip('explicit disposable localhost _test database required')
    parsed = urlsplit(url)
    assert parsed.hostname in {'localhost', '127.0.0.1'} and parsed.path.endswith('_test')
    with psycopg.connect(url, autocommit=True) as c:
        c.execute('SET search_path TO pg_temp')
        migration = Path(__file__).parents[2] / 'football_insight_service_backend_rs/migrations/20261019120000_add_squad_players.sql'
        c.execute(migration.read_text())
        yield PostgresSquadStore(c)


def p(pid, team=1, number=None, position='MF', name=None):
    return SquadPlayer(team, pid, name or f'P{pid}', number, position, None, None)


def rows(store, team=1):
    return store.connection.execute(
        'SELECT player_id, shirt_number, last_season, active FROM f_i_squad_players WHERE team_id=%s ORDER BY player_id',
        (team,)).fetchall()


def test_current_marks_missing_players_as_former(store):
    store.apply(1, 2026, build_squad_rows([p(1), p(2)], 2026, {}), current=True)
    store.apply(1, 2026, build_squad_rows([p(1)], 2026, {}), current=True)
    assert rows(store) == [(1, None, 2026, True), (2, None, 2026, False)]


def test_backfill_adds_former_players_without_touching_current(store):
    store.apply(1, 2026, build_squad_rows([p(1, number=6)], 2026, {}), current=True)
    store.apply(1, 2024, build_squad_rows([p(1, number=30), p(9, number=7)], 2024, {}), current=False)
    assert rows(store) == [(1, 6, 2026, True), (9, 7, 2024, False)]


def test_backfill_moves_last_season_forward_only(store):
    store.apply(1, 2024, build_squad_rows([p(9, number=7)], 2024, {}), current=False)
    store.apply(1, 2025, build_squad_rows([p(9, number=10)], 2025, {}), current=False)
    store.apply(1, 2023, build_squad_rows([p(9, number=1)], 2023, {}), current=False)
    assert rows(store) == [(9, 10, 2025, False)]


def test_same_player_in_two_teams_keeps_two_rows(store):
    store.apply(1, 2026, build_squad_rows([p(5, team=1)], 2026, {}), current=True)
    store.apply(2, 2026, build_squad_rows([p(5, team=2)], 2026, {}), current=True)
    assert rows(store, 1) == [(5, None, 2026, True)]
    assert rows(store, 2) == [(5, None, 2026, True)]


def test_existing_avatar_is_kept_when_new_row_has_none(store):
    store.apply(1, 2026, build_squad_rows([p(1)], 2026, {1: 'https://minio/1.png'}), current=True)
    store.apply(1, 2026, build_squad_rows([p(1)], 2026, {}), current=True)
    assert store.avatar_urls(1) == {1: 'https://minio/1.png'}


def test_empty_apply_is_refused(store):
    with pytest.raises(ValueError):
        store.apply(1, 2026, [], current=True)
```

Run: `cd sina_csl_scraper && uv run pytest tests/test_squad_postgres.py -q`
Expected: 无 `SQUAD_TEST_DATABASE_URL` 时 6 skipped。若本机有一次性库（库名以 `_test` 结尾、host 为 localhost），设置变量后应 6 passed；执行者如能创建一次性库（例如 `docker run --rm -d -p 55499:5432 -e POSTGRES_PASSWORD=x postgres:16` 后 `createdb squad_test`），必须实际跑通并在汇报中写明；不能则写明"postgres 测试未在本机执行"。

- [ ] **Step 6: 提交**

```bash
git pull --rebase
git add sina_csl_scraper/src/sina_csl_scraper/squad_sync.py sina_csl_scraper/tests/test_squad_sync.py sina_csl_scraper/tests/test_squad_postgres.py
git commit -m "feat(scraper): sync team squads with per-team isolation and former players

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
git push
```

---

### Task 4: 抓取器 —— `sync-squads` 命令与每日调度

**Files:**
- Modify: `sina_csl_scraper/src/sina_csl_scraper/cli.py`
- Create: `sina_csl_scraper/sync_squads_daily.sh`、`sina_csl_scraper/deploy/squad-sync.cron.example`
- Test: `sina_csl_scraper/tests/test_squad_cli.py`

**Interfaces:**
- Consumes: `sync_squads`、`PostgresSquadStore`、`make_avatar_uploader`（Task 3）；`SinaCslClient.fetch_league_info` / `fetch_team_squad`。
- Produces: 命令 `sina-csl-scraper sync-squads [--season N] [--backfill-seasons N] [--upload-avatars] --database-url ...`，输出一行 JSON 摘要；有失败球队时退出码仍为 0（失败已隔离），但摘要里列出。

- [ ] **Step 1: 写失败测试（Typer CliRunner + monkeypatch）**

```python
# sina_csl_scraper/tests/test_squad_cli.py
import json

from typer.testing import CliRunner

from sina_csl_scraper import cli


def test_sync_squads_requires_database_url(monkeypatch):
    monkeypatch.delenv('FI_DATABASE_URL', raising=False)
    monkeypatch.delenv('DATABASE_URL', raising=False)
    result = CliRunner().invoke(cli.app, ['sync-squads'])
    assert result.exit_code != 0


def test_sync_squads_uses_league_season_and_prints_summary(monkeypatch):
    captured = {}

    class FakeClient:
        def fetch_league_info(self):
            return type('L', (), {'current_season': 2026})()

        def fetch_team_squad(self, team_id, season=None):
            return []

    class FakeConn:
        def __enter__(self):
            return self

        def __exit__(self, *a):
            return False

    def fake_sync(**kwargs):
        captured.update(kwargs)
        return {'teams_ok': 16, 'teams_failed': [], 'players_current': 500, 'avatars_uploaded': 0}

    monkeypatch.setattr(cli, 'SinaCslClient', lambda: FakeClient())
    monkeypatch.setattr(cli.psycopg, 'connect', lambda *a, **k: FakeConn())
    monkeypatch.setattr(cli, 'sync_squads', fake_sync)
    result = CliRunner().invoke(cli.app, ['sync-squads', '--database-url', 'postgresql://x/y', '--backfill-seasons', '3'])
    assert result.exit_code == 0, result.output
    assert captured['season'] == 2026 and captured['backfill_seasons'] == 3 and captured['avatars'] is None
    assert json.loads(result.output.strip().splitlines()[-1])['teams_ok'] == 16
```

Run: `cd sina_csl_scraper && uv run pytest tests/test_squad_cli.py -q`
Expected: FAIL（无 `sync-squads` 命令 / `cli.sync_squads` 不存在）。

- [ ] **Step 2: 实现命令**

`cli.py` 顶部 import 增加 `import psycopg` 与 `from .squad_sync import PostgresSquadStore, make_avatar_uploader, sync_squads`，在 `sync-player-goals` 命令之后加入：

```python
@app.command("sync-squads")
def sync_squads_command(
    season: int | None = typer.Option(None, help="Current season; defaults to Sina league current season."),
    backfill_seasons: int = typer.Option(0, help="Also import N past seasons as former players (first run: 3)."),
    upload_avatars: bool = typer.Option(False, "--upload-avatars", help="Upload missing avatars to MinIO."),
    database_url: str | None = typer.Option(None, envvar=["FI_DATABASE_URL", "DATABASE_URL"]),
    minio_endpoint: str = typer.Option(_env_default("FI_MINIO_ENDPOINT", "https://oryjk.cn:82/minio")),
    minio_access_key: str = typer.Option(_env_default("FI_MINIO_ACCESS_KEY", "") or ""),
    minio_secret_key: str = typer.Option(_env_default("FI_MINIO_SECRET_KEY", "") or ""),
    minio_bucket: str = typer.Option(_env_default("FI_MINIO_BUCKET", "football-insight") or "football-insight"),
    minio_region: str = typer.Option(_env_default("FI_MINIO_REGION", "us-east-1") or "us-east-1"),
    minio_prefix: str = typer.Option(_env_default("FI_MINIO_PREFIX", "summary") or "summary"),
    minio_public_base_url: str | None = typer.Option(_env_default("FI_MINIO_PUBLIC_BASE_URL")),
) -> None:
    """Sync full team squads (current + former players) into f_i_squad_players."""
    if not database_url:
        raise typer.BadParameter("PostgreSQL database_url is required")
    client = SinaCslClient()
    target_season = season or client.fetch_league_info().current_season
    avatars = None
    if upload_avatars:
        if not minio_access_key or not minio_secret_key:
            raise typer.BadParameter("upload_avatars enabled but MinIO credentials are missing")
        avatars = make_avatar_uploader(
            HttpAssetFetcher(),
            MinioAssetTarget.from_credentials(
                endpoint=minio_endpoint, access_key=minio_access_key, secret_key=minio_secret_key,
                bucket=minio_bucket, region=minio_region, public_base_url=minio_public_base_url,
            ),
            minio_prefix,
        )
    with psycopg.connect(database_url, autocommit=True) as connection:
        summary = sync_squads(
            fetch=lambda team_id, s: client.fetch_team_squad(team_id, season=s),
            store=PostgresSquadStore(connection),
            season=target_season,
            backfill_seasons=backfill_seasons,
            avatars=avatars,
        )
    typer.echo(json.dumps({"season": target_season, **summary}, ensure_ascii=False))
```

- [ ] **Step 3: 确认通过并跑全量**

Run: `cd sina_csl_scraper && uv run pytest -q 2>&1 | tail -3`
Expected: 全部通过（postgres 测试未配置时为 skipped）。

- [ ] **Step 4: 每日脚本与 cron 模板**

`sina_csl_scraper/sync_squads_daily.sh`：复制 `sync_avatars_daily.sh` 的环境加载、`flock` 与校验部分（锁文件改为 `.squad_sync.lock`、变量前缀 `FI_SQUAD_SYNC_*`），最后执行：

```bash
# 首次运行用 FI_SQUAD_SYNC_BACKFILL=3 回填往季前队员；每日调度保持默认 0。
BACKFILL="${FI_SQUAD_SYNC_BACKFILL:-0}"
cd "${SCRIPT_DIR}"
exec uv run sina-csl-scraper sync-squads --upload-avatars --backfill-seasons "${BACKFILL}" "$@"
```

`chmod +x sina_csl_scraper/sync_squads_daily.sh`。

`sina_csl_scraper/deploy/squad-sync.cron.example`：

```
# Football Insight daily squad sync (current-season rosters; former players are kept).
# Runner: local233. First run must be done manually with --backfill-seasons 3.
# Runs every day at 03:45 local machine time (after avatar sync at 03:30).
45 3 * * * /bin/zsh -lc 'cd /home/betalpha/projects/football_insight/sina_csl_scraper && ./sync_squads_daily.sh >> /home/betalpha/projects/football_insight/sina_csl_scraper/squad_sync.log 2>&1'
```

- [ ] **Step 5: 只读冒烟（不写库）**

Run（只验证新浪接口与解析，不连库）：

```bash
cd sina_csl_scraper && uv run python -I -c "
from sina_csl_scraper.client import SinaCslClient
c = SinaCslClient()
for s in (None, 2025):
    ps = c.fetch_team_squad(77680, season=s)
    print(s, len(ps), sum(1 for p in ps if p.position), sum(1 for p in ps if p.shirt_number is not None))
"
```

Expected: 当前赛季约 33 人、2025 约 41 人，位置与号码绝大多数非空。

- [ ] **Step 6: 提交**

```bash
git pull --rebase
git add sina_csl_scraper/src/sina_csl_scraper/cli.py sina_csl_scraper/tests/test_squad_cli.py sina_csl_scraper/sync_squads_daily.sh sina_csl_scraper/deploy/squad-sync.cron.example
git commit -m "feat(scraper): add sync-squads command and daily schedule

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
git push
```

---

### Task 5: 后端 —— `lineup` 模块名单接口

**Files:**
- Create: `football_insight_service_backend_rs/src/lineup/mod.rs`、`domain.rs`、`ports.rs`、`application.rs`、`bootstrap.rs`、`adapters/mod.rs`、`adapters/persistence.rs`、`adapters/web.rs`
- Modify: `football_insight_service_backend_rs/src/lib.rs`、`src/app.rs`

**Interfaces:**
- Produces（阶段二、三继续扩展）：

```rust
// domain.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Position { GK, DF, MF, FW }
pub struct SquadTeam { pub team_id: i64, pub team_name: String, pub avatar_url: Option<String> }
pub struct SquadPlayer { pub player_id: i64, pub name: String, pub shirt_number: Option<i32>,
    pub position: Option<Position>, pub avatar_url: Option<String>, pub last_season: i32,
    #[serde(skip)] pub active: bool }
pub struct SquadResponse { pub team: SquadTeam, pub active: Vec<SquadPlayer>, pub former: Vec<SquadPlayer> }
pub enum LineupReadError { Invalid, NotFound, Unavailable }   // thiserror，中文消息
// ports.rs
#[async_trait] pub trait SquadRepository: Send + Sync {
    async fn find_team(&self, team_id: i64) -> Result<Option<SquadTeam>, LineupReadError>;
    async fn list_squad(&self, team_id: i64) -> Result<Vec<SquadPlayer>, LineupReadError>;
}
// application.rs
pub struct LineupService { .. } impl LineupService {
    pub fn new(squads: Arc<dyn SquadRepository>) -> Self;
    pub async fn squad(&self, team_id: i64) -> Result<SquadResponse, LineupReadError>;
}
// bootstrap.rs
pub fn build_lineup_routes(pool: PgPool) -> Router
// 路由：GET /api/v1/lineups/squads/{team_id}
```

- [ ] **Step 1: 写 domain 与 ports（无行为）**

```rust
// src/lineup/domain.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Position {
    GK,
    DF,
    MF,
    FW,
}

impl Position {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "GK" => Some(Self::GK),
            "DF" => Some(Self::DF),
            "MF" => Some(Self::MF),
            "FW" => Some(Self::FW),
            _ => None,
        }
    }

    /// 排序权重：门将、后卫、中场、前锋，未知排最后。
    pub fn rank(position: Option<Self>) -> u8 {
        match position {
            Some(Self::GK) => 0,
            Some(Self::DF) => 1,
            Some(Self::MF) => 2,
            Some(Self::FW) => 3,
            None => 4,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SquadTeam {
    pub team_id: i64,
    pub team_name: String,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SquadPlayer {
    pub player_id: i64,
    pub name: String,
    pub shirt_number: Option<i32>,
    pub position: Option<Position>,
    pub avatar_url: Option<String>,
    pub last_season: i32,
    #[serde(skip)]
    pub active: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SquadResponse {
    pub team: SquadTeam,
    pub active: Vec<SquadPlayer>,
    pub former: Vec<SquadPlayer>,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum LineupReadError {
    #[error("球队参数无效")]
    Invalid,
    #[error("球队不存在")]
    NotFound,
    #[error("名单暂不可用")]
    Unavailable,
}
```

```rust
// src/lineup/ports.rs
use super::domain::{LineupReadError, SquadPlayer, SquadTeam};
use async_trait::async_trait;

#[async_trait]
pub trait SquadRepository: Send + Sync {
    async fn find_team(&self, team_id: i64) -> Result<Option<SquadTeam>, LineupReadError>;
    async fn list_squad(&self, team_id: i64) -> Result<Vec<SquadPlayer>, LineupReadError>;
}
```

```rust
// src/lineup/mod.rs
pub mod adapters;
pub mod application;
pub mod bootstrap;
pub mod domain;
pub mod ports;
```

```rust
// src/lineup/adapters/mod.rs
pub mod persistence;
pub mod web;
```

`src/lib.rs` 按字母序加 `pub mod lineup;`。

- [ ] **Step 2: 写 application 失败测试**

`src/lineup/application.rs`：

```rust
use super::{
    domain::{LineupReadError, Position, SquadPlayer, SquadResponse},
    ports::SquadRepository,
};
use std::sync::Arc;

pub struct LineupService {
    squads: Arc<dyn SquadRepository>,
}

impl LineupService {
    pub fn new(squads: Arc<dyn SquadRepository>) -> Self {
        Self { squads }
    }

    pub async fn squad(&self, _team_id: i64) -> Result<SquadResponse, LineupReadError> {
        todo!()
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::lineup::domain::SquadTeam;
    use async_trait::async_trait;

    pub(crate) struct FakeSquads {
        pub players: Vec<SquadPlayer>,
        pub missing_table: bool,
    }

    #[async_trait]
    impl SquadRepository for FakeSquads {
        async fn find_team(&self, team_id: i64) -> Result<Option<SquadTeam>, LineupReadError> {
            if self.missing_table {
                return Err(LineupReadError::Unavailable);
            }
            Ok((team_id == 77680).then(|| SquadTeam {
                team_id,
                team_name: "成都蓉城".into(),
                avatar_url: None,
            }))
        }
        async fn list_squad(&self, _team_id: i64) -> Result<Vec<SquadPlayer>, LineupReadError> {
            Ok(self.players.clone())
        }
    }

    pub(crate) fn player(id: i64, number: Option<i32>, position: Option<Position>, season: i32, active: bool) -> SquadPlayer {
        SquadPlayer {
            player_id: id,
            name: format!("P{id}"),
            shirt_number: number,
            position,
            avatar_url: None,
            last_season: season,
            active,
        }
    }

    fn service(players: Vec<SquadPlayer>) -> LineupService {
        LineupService::new(Arc::new(FakeSquads { players, missing_table: false }))
    }

    #[tokio::test]
    async fn rejects_non_positive_team_ids() {
        assert_eq!(service(vec![]).squad(0).await.unwrap_err(), LineupReadError::Invalid);
    }

    #[tokio::test]
    async fn unknown_team_is_not_found() {
        assert_eq!(service(vec![]).squad(1).await.unwrap_err(), LineupReadError::NotFound);
    }

    #[tokio::test]
    async fn missing_table_is_unavailable() {
        let s = LineupService::new(Arc::new(FakeSquads { players: vec![], missing_table: true }));
        assert_eq!(s.squad(77680).await.unwrap_err(), LineupReadError::Unavailable);
    }

    #[tokio::test]
    async fn sorts_active_by_position_then_number_with_blanks_last() {
        let res = service(vec![
            player(1, Some(9), Some(Position::FW), 2026, true),
            player(2, None, Some(Position::MF), 2026, true),
            player(3, Some(6), Some(Position::MF), 2026, true),
            player(4, Some(1), Some(Position::GK), 2026, true),
            player(5, Some(3), None, 2026, true),
        ])
        .squad(77680)
        .await
        .unwrap();
        let ids: Vec<i64> = res.active.iter().map(|p| p.player_id).collect();
        assert_eq!(ids, vec![4, 3, 2, 1, 5]);
        assert!(res.former.is_empty());
    }

    #[tokio::test]
    async fn former_players_are_grouped_by_latest_season_first() {
        let res = service(vec![
            player(10, Some(7), Some(Position::FW), 2024, false),
            player(11, Some(2), Some(Position::DF), 2025, false),
            player(12, Some(8), Some(Position::MF), 2025, false),
            player(13, Some(1), Some(Position::GK), 2026, true),
        ])
        .squad(77680)
        .await
        .unwrap();
        assert_eq!(res.active.iter().map(|p| p.player_id).collect::<Vec<_>>(), vec![13]);
        assert_eq!(res.former.iter().map(|p| p.player_id).collect::<Vec<_>>(), vec![11, 12, 10]);
        assert_eq!(res.team.team_name, "成都蓉城");
    }
}
```

Run: `cd football_insight_service_backend_rs && cargo test lineup::application -- --nocapture 2>&1 | tail -8`
Expected: FAIL（`todo!()` panic）。

- [ ] **Step 3: 实现 `squad`**

```rust
    pub async fn squad(&self, team_id: i64) -> Result<SquadResponse, LineupReadError> {
        if team_id <= 0 {
            return Err(LineupReadError::Invalid);
        }
        let team = self
            .squads
            .find_team(team_id)
            .await?
            .ok_or(LineupReadError::NotFound)?;
        let (mut active, mut former): (Vec<SquadPlayer>, Vec<SquadPlayer>) = self
            .squads
            .list_squad(team_id)
            .await?
            .into_iter()
            .partition(|player| player.active);
        let by_role = |a: &SquadPlayer, b: &SquadPlayer| {
            Position::rank(a.position)
                .cmp(&Position::rank(b.position))
                .then(a.shirt_number.unwrap_or(i32::MAX).cmp(&b.shirt_number.unwrap_or(i32::MAX)))
                .then(a.name.cmp(&b.name))
        };
        active.sort_by(by_role);
        former.sort_by(|a, b| b.last_season.cmp(&a.last_season).then(by_role(a, b)));
        Ok(SquadResponse { team, active, former })
    }
```

Run: `cd football_insight_service_backend_rs && cargo test lineup::application 2>&1 | tail -4`
Expected: 5 passed。

- [ ] **Step 4: 持久层**

```rust
// src/lineup/adapters/persistence.rs
use crate::lineup::{
    domain::{LineupReadError, Position, SquadPlayer, SquadTeam},
    ports::SquadRepository,
};
use async_trait::async_trait;
use sqlx::PgPool;

pub struct PostgresSquadRepository {
    pool: PgPool,
}

impl PostgresSquadRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn read_error(e: sqlx::Error) -> LineupReadError {
    let missing = e.as_database_error().is_some_and(|d| d.code().as_deref() == Some("42P01"));
    tracing::warn!(missing_table = missing, "lineup squad read unavailable");
    LineupReadError::Unavailable
}

#[async_trait]
impl SquadRepository for PostgresSquadRepository {
    async fn find_team(&self, team_id: i64) -> Result<Option<SquadTeam>, LineupReadError> {
        sqlx::query_as::<_, (i64, String, Option<String>)>(
            "SELECT team_id, team_name, avatar_storage_url FROM f_i_teams WHERE team_id = $1",
        )
        .bind(team_id)
        .fetch_optional(&self.pool)
        .await
        .map(|row| row.map(|(team_id, team_name, avatar_url)| SquadTeam { team_id, team_name, avatar_url }))
        .map_err(read_error)
    }

    async fn list_squad(&self, team_id: i64) -> Result<Vec<SquadPlayer>, LineupReadError> {
        type Row = (i64, String, Option<i32>, Option<String>, Option<String>, i32, bool);
        sqlx::query_as::<_, Row>(
            "SELECT player_id, player_name, shirt_number, position, avatar_storage_url, last_season, active
             FROM f_i_squad_players WHERE team_id = $1",
        )
        .bind(team_id)
        .fetch_all(&self.pool)
        .await
        .map(|rows| {
            rows.into_iter()
                .map(|(player_id, name, shirt_number, position, avatar_url, last_season, active)| SquadPlayer {
                    player_id,
                    name,
                    shirt_number,
                    position: position.as_deref().and_then(Position::parse),
                    avatar_url,
                    last_season,
                    active,
                })
                .collect()
        })
        .map_err(read_error)
    }
}
```

先确认 `f_i_teams.team_id` 的类型（`\d f_i_teams` 或查已有查询），若为 `BIGINT` 则如上；若不同，按实际类型调整 `query_as` 元组并在汇报里写明。

- [ ] **Step 5: 路由与路由测试**

```rust
// src/lineup/adapters/web.rs
use crate::lineup::{application::LineupService, domain::{LineupReadError, SquadResponse}};
use axum::{Json, Router, extract::{Path, State}, http::StatusCode, routing::get};
use std::sync::Arc;

pub fn routes(service: Arc<LineupService>) -> Router {
    Router::new()
        .route("/api/v1/lineups/squads/{team_id}", get(get_squad))
        .with_state(service)
}

fn map_error(e: LineupReadError) -> (StatusCode, String) {
    let status = match e {
        LineupReadError::Invalid => StatusCode::BAD_REQUEST,
        LineupReadError::NotFound => StatusCode::NOT_FOUND,
        LineupReadError::Unavailable => StatusCode::SERVICE_UNAVAILABLE,
    };
    (status, e.to_string())
}

async fn get_squad(
    State(s): State<Arc<LineupService>>,
    Path(team_id): Path<i64>,
) -> Result<Json<SquadResponse>, (StatusCode, String)> {
    s.squad(team_id).await.map(Json).map_err(map_error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lineup::{application::tests::{FakeSquads, player}, domain::Position};
    use axum::{body::{Body, to_bytes}, http::Request};
    use tower::ServiceExt;

    fn app(missing_table: bool) -> Router {
        routes(Arc::new(LineupService::new(Arc::new(FakeSquads {
            players: vec![player(1, Some(1), Some(Position::GK), 2026, true), player(2, Some(9), Some(Position::FW), 2025, false)],
            missing_table,
        }))))
    }

    async fn call(app: Router, uri: &str) -> (u16, String) {
        let res = app.oneshot(Request::get(uri).body(Body::empty()).unwrap()).await.unwrap();
        let status = res.status().as_u16();
        let body = String::from_utf8(to_bytes(res.into_body(), usize::MAX).await.unwrap().to_vec()).unwrap();
        (status, body)
    }

    #[tokio::test]
    async fn returns_grouped_squad_without_internal_flags() {
        let (status, body) = call(app(false), "/api/v1/lineups/squads/77680").await;
        assert_eq!(status, 200);
        let json: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(json["team"]["team_name"], "成都蓉城");
        assert_eq!(json["active"][0]["position"], "GK");
        assert_eq!(json["former"][0]["last_season"], 2025);
        assert!(json["active"][0].get("active").is_none());
    }

    #[tokio::test]
    async fn maps_errors_to_statuses() {
        assert_eq!(call(app(false), "/api/v1/lineups/squads/0").await.0, 400);
        assert_eq!(call(app(false), "/api/v1/lineups/squads/1").await.0, 404);
        assert_eq!(call(app(false), "/api/v1/lineups/squads/abc").await.0, 400);
        assert_eq!(call(app(true), "/api/v1/lineups/squads/77680").await.0, 503);
    }
}
```

```rust
// src/lineup/bootstrap.rs
use super::{adapters::{persistence::PostgresSquadRepository, web::routes}, application::LineupService};
use axum::Router;
use sqlx::PgPool;
use std::sync::Arc;

pub fn build_lineup_routes(pool: PgPool) -> Router {
    routes(Arc::new(LineupService::new(Arc::new(PostgresSquadRepository::new(pool)))))
}
```

`src/app.rs`：仿照 `match_outlook_routes`，在其后加 `let lineup_routes = crate::lineup::bootstrap::build_lineup_routes(pool.clone());`，并在 Router 链中 `.merge(match_outlook_routes)` 之后加 `.merge(lineup_routes)`。

Run: `cd football_insight_service_backend_rs && cargo test lineup 2>&1 | tail -4`
Expected: 7 passed（application 5 + web 2）。

- [ ] **Step 6: 全量测试与格式**

Run: `cd football_insight_service_backend_rs && cargo fmt && cargo test 2>&1 | grep -E "^test result" | tail -5`
Expected: 所有 `test result: ok`，0 failed（数据库测试按现有约定 ignored/skip）。

- [ ] **Step 7: 重启本地后端并只读验证**

```bash
cd /home/betalpha/projects/football_insight && ./deploy/local-test/up.sh
curl -s -m 10 -o /dev/null -w "%{http_code}\n" http://127.0.0.1:18092/api/v1/lineups/squads/77680
```

Expected: 迁移尚未在生产执行时返回 `503`（表不存在被正确映射为暂不可用）；执行迁移与同步后（Task 6）返回 `200`。汇报写明"本地后端已重启"。

- [ ] **Step 8: 提交**

```bash
git pull --rebase
git add football_insight_service_backend_rs/src/lineup football_insight_service_backend_rs/src/lib.rs football_insight_service_backend_rs/src/app.rs
git commit -m "feat(backend): add lineup module with squad read endpoint

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
git push
```

---

### Task 6: 生产迁移与首次同步（需负责人确认后执行）

此任务有生产副作用：**执行前必须停下来询问负责人**，得到明确同意再做。

- [ ] **Step 1: 询问负责人**：是否在 peiqian 执行迁移 `20261019120000_add_squad_players.sql`，并在 local233 运行带 `--backfill-seasons 3 --upload-avatars` 的首次同步（写生产 `football_data`、上传约数百张头像到 MinIO）。

- [ ] **Step 2（同意后）: peiqian 执行迁移**

```bash
ssh peiqian 'cd /root/projects/football_insight && git pull --ff-only && cd football_insight_service_backend_rs && cargo run --bin run_migrations'
```

Expected: `_sqlx_migrations` 中出现 `20261019120000` 且 success。

- [ ] **Step 3: local233 首次同步**

```bash
cd sina_csl_scraper && FI_SQUAD_SYNC_BACKFILL=3 ./sync_squads_daily.sh
```

Expected: 输出 JSON `teams_ok` 为 16、`teams_failed` 为空；只读查询每队 `active` 人数约 25–35、`former` 若干。

- [ ] **Step 4: 本地后端验证**

```bash
curl -s -m 10 http://127.0.0.1:18092/api/v1/lineups/squads/77680 | python3 -c "import sys,json; d=json.load(sys.stdin); print(d['team']['team_name'], len(d['active']), len(d['former']), d['active'][0])"
```

Expected: `成都蓉城 33 N {...position 'GK'...}`。

- [ ] **Step 5: 安装 cron（同意后）**：把 `deploy/squad-sync.cron.example` 的行加入 local233 用户 crontab，`crontab -l | grep sync_squads` 确认。

---

### Task 7: 文档

**Files:** `CHANGELOG.md`、`FEATURES.md`、`DEPLOYMENT.md`

- [ ] **Step 1**: `CHANGELOG.md` 顶部加一条"战术板阶段一：球队完整名单同步与读取接口"，写明变更、原因（为最佳阵容功能提供含号码/位置/前队员的名单）、验证结果与发布状态（本地后端已重启 / 生产迁移是否执行 / 生产后端未部署）。
- [ ] **Step 2**: `FEATURES.md` "进行中"加"战术板（最佳阵容）：阶段一名单数据完成，阶段二编辑与保存、阶段三分享待开发"。
- [ ] **Step 3**: `DEPLOYMENT.md` 加"球队名单同步"小节：迁移名、首次回填命令、每日 cron、失败隔离说明、如何只读核对。
- [ ] **Step 4: 提交**

```bash
git pull --rebase
git add CHANGELOG.md FEATURES.md DEPLOYMENT.md
git commit -m "docs: record lineup board phase one squad data

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
git push
```

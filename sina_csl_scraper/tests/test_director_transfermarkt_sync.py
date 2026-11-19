from __future__ import annotations

import json
from datetime import date
from typing import Any

import pytest
from typer.testing import CliRunner

from sina_csl_scraper.cli import app
from sina_csl_scraper.director_transfermarkt_sync import (
    TRANSFERMARKT_LEAGUE_CODES,
    UPDATE_DIRECTOR_CANDIDATE_SQL,
    UPDATE_SQUAD_PLAYER_SQL,
    DirectorCandidateRecord,
    SquadPlayerRecord,
    TransfermarktClub,
    TransfermarktPlayer,
    match_director_candidate,
    match_squad_player,
    normalize_club_name,
    normalize_player_name,
    parse_club_squad,
    parse_league_clubs,
    parse_market_value_eur,
    parse_tm_date,
    parse_tm_leagues_arg,
    sync_transfermarkt_valuations,
)


# ---------------------------------------------------------------------------
# Fixtures: realistic Transfermarkt HTML snippets
# ---------------------------------------------------------------------------

SAMPLE_LEAGUE_HTML = """
<html>
<body>
  <nav>
    <a href="/other-club/startseite/verein/99999">Nav Noise FC</a>
  </nav>
  <div id="yw1" class="grid-view">
    <table class="items">
      <thead>
        <tr><th>俱乐部</th><th>人数</th><th>总身价</th></tr>
      </thead>
      <tbody>
        <tr class="odd">
          <td class="zentriert no-border-rechts">
            <a title="上海海港" href="/shanghai-port/startseite/verein/27190/saison_id/2025">
              <img src="/crest/27190.png" alt="上海海港" />
            </a>
          </td>
          <td class="hauptlink no-border-links">
            <a title="上海海港" href="/shanghai-port/startseite/verein/27190/saison_id/2025">上海海港</a>
          </td>
          <td class="zentriert">
            <a href="/shanghai-port/kader/verein/27190/saison_id/2025">30</a>
          </td>
        </tr>
        <tr class="even">
          <td class="zentriert no-border-rechts">
            <a title="成都蓉城" href="/chengdu-rongcheng/startseite/verein/67621/saison_id/2025">
              <img src="/crest/67621.png" alt="成都蓉城" />
            </a>
          </td>
          <td class="hauptlink no-border-links">
            <a title="成都蓉城" href="/chengdu-rongcheng/startseite/verein/67621/saison_id/2025">成都蓉城</a>
          </td>
          <td class="zentriert">
            <a href="/chengdu-rongcheng/kader/verein/67621/saison_id/2025">29</a>
          </td>
        </tr>
      </tbody>
    </table>
  </div>
</body>
</html>
"""

SAMPLE_ZH_SQUAD_HTML = """
<html>
<body>
  <div id="yw1" class="grid-view">
    <table class="items">
      <thead>
        <tr>
          <th id="yw1_c0">#</th>
          <th id="yw1_c1">球员</th>
          <th id="yw1_c2">出生日期/年龄</th>
          <th id="yw1_c3">国籍</th>
          <th id="yw1_c4">身高</th>
          <th id="yw1_c5">惯用脚</th>
          <th id="yw1_c6">加盟时间</th>
          <th id="yw1_c7">来自</th>
          <th id="yw1_c8">合同到期</th>
          <th id="yw1_c9">身价</th>
        </tr>
      </thead>
      <tbody>
        <tr class="odd">
          <td class="zentriert rueckennummer bg_Sturm"><div class="rn_nummer">7</div></td>
          <td class="posrela">
            <table class="inline-table">
              <tr>
                <td rowspan="2"><img src="/photo/140436.jpg" alt="武磊" /></td>
                <td class="hauptlink"><a href="/wu-lei/profil/spieler/140436">武磊</a></td>
              </tr>
              <tr><td>右边锋</td></tr>
            </table>
          </td>
          <td class="zentriert">1991年11月19日 (34)</td>
          <td class="zentriert"><img title="中国" alt="中国" class="flaggenrahmen" /></td>
          <td class="zentriert">1,74 m</td>
          <td class="zentriert">右脚</td>
          <td class="zentriert">2022年8月11日</td>
          <td class="zentriert"><a href="/espanyol/startseite/verein/714"><img title="西班牙人" /></a></td>
          <td class="zentriert">2026年12月31日</td>
          <td class="rechts hauptlink"><a href="/wu-lei/marktwertverlauf/spieler/140436">120万欧</a></td>
        </tr>
        <tr class="even">
          <td class="zentriert rueckennummer bg_Mittelfeld"><div class="rn_nummer">10</div></td>
          <td class="posrela">
            <table class="inline-table">
              <tr>
                <td rowspan="2"><img src="/photo/85314.jpg" alt="奥斯卡" /></td>
                <td class="hauptlink"><a href="/oscar/profil/spieler/85314">奥斯卡</a></td>
              </tr>
              <tr><td>前腰</td></tr>
            </table>
          </td>
          <td class="zentriert">1991年9月9日 (34)</td>
          <td class="zentriert"><img title="巴西" alt="巴西" class="flaggenrahmen" /></td>
          <td class="zentriert">1,79 m</td>
          <td class="zentriert">右脚</td>
          <td class="zentriert">2017年1月1日</td>
          <td class="zentriert"><a href="/chelsea/startseite/verein/631"><img title="切尔西" /></a></td>
          <td class="zentriert">-</td>
          <td class="rechts hauptlink"><a href="/oscar/marktwertverlauf/spieler/85314">450.00万欧元</a></td>
        </tr>
        <tr class="odd">
          <td class="zentriert rueckennummer bg_Torwart"><div class="rn_nummer">-</div></td>
          <td class="posrela">
            <table class="inline-table">
              <tr>
                <td rowspan="2"><img src="/photo/999001.jpg" alt="青训门将" /></td>
                <td class="hauptlink"><a href="/youth-gk/profil/spieler/999001">青训门将</a></td>
              </tr>
              <tr><td>门将</td></tr>
            </table>
          </td>
          <td class="zentriert">2005年3月4日 (21)</td>
          <td class="zentriert"><img title="中国" /></td>
          <td class="zentriert">1,88 m</td>
          <td class="zentriert">右脚</td>
          <td class="zentriert">2025年1月1日</td>
          <td class="zentriert">-</td>
          <td class="zentriert">2028年12月31日</td>
          <td class="rechts hauptlink">-</td>
        </tr>
      </tbody>
    </table>
  </div>
</body>
</html>
"""

SAMPLE_EN_DE_SQUAD_HTML = """
<table class="items">
  <thead>
    <tr>
      <th>#</th>
      <th>Player</th>
      <th>Date of birth/Age</th>
      <th>Nat.</th>
      <th>Height</th>
      <th>Foot</th>
      <th>Joined</th>
      <th>Signed from</th>
      <th>Contract</th>
      <th>Market value</th>
    </tr>
  </thead>
  <tbody>
    <tr class="odd">
      <td class="zentriert rueckennummer"><div class="rn_nummer">9</div></td>
      <td class="posrela">
        <table class="inline-table">
          <tr>
            <td rowspan="2"><img src="/p1.jpg" /></td>
            <td class="hauptlink"><a href="/felipe/profil/spieler/5001">Felipe Silva</a></td>
          </tr>
          <tr><td>Centre-Forward</td></tr>
        </table>
      </td>
      <td class="zentriert">May 14, 1997 (29)</td>
      <td class="zentriert"><img title="Brazil" /></td>
      <td class="zentriert">1,93 m</td>
      <td class="zentriert">left</td>
      <td class="zentriert">Jul 1, 2023</td>
      <td class="zentriert">-</td>
      <td class="zentriert">Dec 31, 2026</td>
      <td class="rechts hauptlink"><a href="/felipe/marktwertverlauf/spieler/5001">€1.50m</a></td>
    </tr>
    <tr class="even">
      <td class="zentriert rueckennummer"><div class="rn_nummer">21</div></td>
      <td class="posrela">
        <table class="inline-table">
          <tr>
            <td rowspan="2"><img src="/p2.jpg" /></td>
            <td class="hauptlink"><a href="/ruben/profil/spieler/5002">Rúben Neves</a></td>
          </tr>
          <tr><td>Defensive Midfield</td></tr>
        </table>
      </td>
      <td class="zentriert">13.03.1997 (29)</td>
      <td class="zentriert"><img title="Portugal" /></td>
      <td class="zentriert">1,80 m</td>
      <td class="zentriert">right</td>
      <td class="zentriert">01.07.2023</td>
      <td class="zentriert">-</td>
      <td class="zentriert">30.06.2027</td>
      <td class="rechts hauptlink"><a href="/ruben/marktwertverlauf/spieler/5002">1,50 Mio. €</a></td>
    </tr>
  </tbody>
</table>
"""


class FakeTransfermarktClient:
    def __init__(
        self,
        *,
        league_pages: dict[str, str | Exception],
        club_pages: dict[int, str | Exception],
    ) -> None:
        self.league_pages = league_pages
        self.club_pages = club_pages
        self.fetched_leagues: list[str] = []
        self.fetched_clubs: list[int] = []

    def fetch_league_html(self, tm_code: str) -> str:
        self.fetched_leagues.append(tm_code)
        val = self.league_pages[tm_code]
        if isinstance(val, Exception):
            raise val
        return val

    def fetch_club_squad_html(self, club_id: int) -> str:
        self.fetched_clubs.append(club_id)
        val = self.club_pages[club_id]
        if isinstance(val, Exception):
            raise val
        return val


class FakeTransfermarktStore:
    def __init__(
        self,
        *,
        squad_players: list[SquadPlayerRecord] | None = None,
        director_candidates: list[DirectorCandidateRecord] | None = None,
    ) -> None:
        self._squad_players = list(squad_players or [])
        self._director_candidates = list(director_candidates or [])
        self.updated_squad_rows: list[dict[str, Any]] = []
        self.updated_candidate_rows: list[dict[str, Any]] = []

    def load_squad_players(self) -> list[SquadPlayerRecord]:
        return list(self._squad_players)

    def load_director_candidates(self) -> list[DirectorCandidateRecord]:
        return list(self._director_candidates)

    def update_squad_players(self, rows: list[dict[str, Any]]) -> int:
        self.updated_squad_rows.extend(rows)
        return len(rows)

    def update_director_candidates(self, rows: list[dict[str, Any]]) -> int:
        self.updated_candidate_rows.extend(rows)
        return len(rows)


# ---------------------------------------------------------------------------
# Unit tests: value & date parsing
# ---------------------------------------------------------------------------


@pytest.mark.parametrize(
    ("raw", "expected"),
    [
        ("€1.50m", 1_500_000),
        ("€800k", 800_000),
        ("€50k", 50_000),
        ("€1.20bn", 1_200_000_000),
        ("€800Th.", 800_000),
        ("150万欧元", 1_500_000),
        ("150.00万欧", 1_500_000),
        ("80万", 800_000),
        ("1.50亿欧", 150_000_000),
        ("50千欧", 50_000),
        ("1,50 Mio. €", 1_500_000),
        ("800 Tsd. €", 800_000),
        ("1,20 Mrd. €", 1_200_000_000),
        ("€250,000", 250_000),
        ("€1.15m", 1_150_000),
        ("-", None),
        ("—", None),
        ("", None),
        ("暂无", None),
        ("0", None),
    ],
)
def test_parse_market_value_eur(raw: str, expected: int | None) -> None:
    assert parse_market_value_eur(raw) == expected


@pytest.mark.parametrize(
    ("raw", "expected"),
    [
        ("1997-05-14", date(1997, 5, 14)),
        ("1997/05/14", date(1997, 5, 14)),
        ("14/05/1997", date(1997, 5, 14)),
        ("14/05/1997 (29)", date(1997, 5, 14)),
        ("May 14, 1997", date(1997, 5, 14)),
        ("May 14, 1997 (29)", date(1997, 5, 14)),
        ("14.05.1997", date(1997, 5, 14)),
        ("14.05.1997 (29)", date(1997, 5, 14)),
        ("1997年5月14日", date(1997, 5, 14)),
        ("1997年5月14日 (29)", date(1997, 5, 14)),
        ("31.12.2026", date(2026, 12, 31)),
        ("Dec 31, 2026", date(2026, 12, 31)),
        ("2026年12月31日", date(2026, 12, 31)),
        ("14. Mai 1997", date(1997, 5, 14)),
        ("-", None),
        ("?", None),
        ("", None),
        ("2026-02-30", None),
    ],
)
def test_parse_tm_date(raw: str, expected: date | None) -> None:
    assert parse_tm_date(raw) == expected


def test_league_codes_and_arg_parser() -> None:
    assert TRANSFERMARKT_LEAGUE_CODES == {
        "csl": "CSL",
        "k1": "RSK1",
        "j1": "JAP1",
        "epl": "GB1",
        "laliga": "ES1",
        "seriea": "IT1",
        "bundesliga": "L1",
        "ligue1": "FR1",
    }
    assert parse_tm_leagues_arg("csl, k1, CSL, epl") == ("csl", "k1", "epl")
    with pytest.raises(ValueError, match="empty"):
        parse_tm_leagues_arg(" , ")
    with pytest.raises(ValueError, match="Unsupported league key"):
        parse_tm_leagues_arg("csl,unknown")


# ---------------------------------------------------------------------------
# Unit tests: HTML parsing
# ---------------------------------------------------------------------------


def test_parse_league_clubs_extracts_unique_clubs_and_ignores_nav_and_counts() -> None:
    clubs = parse_league_clubs(SAMPLE_LEAGUE_HTML)
    assert clubs == [
        TransfermarktClub(
            club_id=27190,
            club_name="上海海港",
            href="/shanghai-port/startseite/verein/27190/saison_id/2025",
        ),
        TransfermarktClub(
            club_id=67621,
            club_name="成都蓉城",
            href="/chengdu-rongcheng/startseite/verein/67621/saison_id/2025",
        ),
    ]


def test_parse_club_squad_chinese_plus1() -> None:
    players = parse_club_squad(SAMPLE_ZH_SQUAD_HTML, club_name="上海海港", league_key="csl")
    assert players == [
        TransfermarktPlayer(
            player_name="武磊",
            club_name="上海海港",
            league_key="csl",
            shirt_number=7,
            birth_date=date(1991, 11, 19),
            contract_until=date(2026, 12, 31),
            market_value_eur=1_200_000,
            tm_player_id=140436,
        ),
        TransfermarktPlayer(
            player_name="奥斯卡",
            club_name="上海海港",
            league_key="csl",
            shirt_number=10,
            birth_date=date(1991, 9, 9),
            contract_until=None,
            market_value_eur=4_500_000,
            tm_player_id=85314,
        ),
        TransfermarktPlayer(
            player_name="青训门将",
            club_name="上海海港",
            league_key="csl",
            shirt_number=None,
            birth_date=date(2005, 3, 4),
            contract_until=date(2028, 12, 31),
            market_value_eur=None,
            tm_player_id=999001,
        ),
    ]


def test_parse_club_squad_english_and_german_formats() -> None:
    players = parse_club_squad(SAMPLE_EN_DE_SQUAD_HTML, club_name="Test FC", league_key="epl")
    assert len(players) == 2
    assert players[0] == TransfermarktPlayer(
        player_name="Felipe Silva",
        club_name="Test FC",
        league_key="epl",
        shirt_number=9,
        birth_date=date(1997, 5, 14),
        contract_until=date(2026, 12, 31),
        market_value_eur=1_500_000,
        tm_player_id=5001,
    )
    assert players[1] == TransfermarktPlayer(
        player_name="Rúben Neves",
        club_name="Test FC",
        league_key="epl",
        shirt_number=21,
        birth_date=date(1997, 3, 13),
        contract_until=date(2027, 6, 30),
        market_value_eur=1_500_000,
        tm_player_id=5002,
    )


def test_parse_club_squad_headerless_fallback() -> None:
    compact_html = """
    <table class="items">
      <tbody>
        <tr class="odd">
          <td class="zentriert rueckennummer"><div class="rn_nummer">11</div></td>
          <td class="posrela"><a href="/son/profil/spieler/91845">孙兴慜</a></td>
          <td class="zentriert">08/07/1992 (33)</td>
          <td class="zentriert">30/06/2026</td>
          <td class="rechts hauptlink">€30.00m</td>
        </tr>
      </tbody>
    </table>
    """
    players = parse_club_squad(compact_html, club_name="热刺", league_key="epl")
    assert players == [
        TransfermarktPlayer(
            player_name="孙兴慜",
            club_name="热刺",
            league_key="epl",
            shirt_number=11,
            birth_date=date(1992, 7, 8),
            contract_until=date(2026, 6, 30),
            market_value_eur=30_000_000,
            tm_player_id=91845,
        )
    ]


# ---------------------------------------------------------------------------
# Unit tests: matching & sync orchestration
# ---------------------------------------------------------------------------


def test_name_and_club_normalization() -> None:
    assert normalize_player_name("费利佩·席尔瓦") == normalize_player_name("费利佩-席尔瓦")
    assert normalize_player_name("Rúben Neves") == normalize_player_name("ruben neves")
    assert normalize_club_name("上海海港足球俱乐部") == normalize_club_name("上海海港")
    assert normalize_club_name("Ulsan HD FC") == normalize_club_name("Ulsan HD")


def test_match_squad_player_tie_breaks_by_club_and_shirt_number() -> None:
    records = [
        SquadPlayerRecord(team_id=1, player_id=101, player_name="张伟", team_name="山东泰山", shirt_number=5, active=True),
        SquadPlayerRecord(team_id=2, player_id=202, player_name="张伟", team_name="上海海港", shirt_number=18, active=True),
        SquadPlayerRecord(team_id=2, player_id=203, player_name="张伟", team_name="上海海港", shirt_number=7, active=True),
    ]
    by_name = {normalize_player_name("张伟"): records}

    tm_player = TransfermarktPlayer(
        player_name="张伟",
        club_name="上海海港足球俱乐部",
        league_key="csl",
        shirt_number=7,
        market_value_eur=500_000,
    )
    matched = match_squad_player(tm_player, by_name)
    assert matched is not None
    assert (matched.team_id, matched.player_id) == (2, 203)


def test_match_director_candidate_tie_breaks_by_current_club() -> None:
    records = [
        DirectorCandidateRecord(id=10, player_name="Felipe Silva", current_club="全北现代", league_tag="K1"),
        DirectorCandidateRecord(id=20, player_name="Felipe Silva", current_club="蔚山HD", league_tag="K1"),
    ]
    by_name = {normalize_player_name("Felipe Silva"): records}

    tm_player = TransfermarktPlayer(
        player_name="Felipe·Silva",
        club_name="蔚山HD FC",
        league_key="k1",
        market_value_eur=1_500_000,
    )
    matched = match_director_candidate(tm_player, by_name)
    assert matched is not None
    assert matched.id == 20


def test_sync_transfermarkt_valuations_dry_run_vs_write_db() -> None:
    k1_league_html = """
    <table class="items"><tbody>
      <tr class="odd"><td class="hauptlink"><a href="/ulsan/startseite/verein/3535">蔚山HD</a></td></tr>
    </tbody></table>
    """
    client = FakeTransfermarktClient(
        league_pages={"CSL": SAMPLE_LEAGUE_HTML, "RSK1": k1_league_html},
        club_pages={
            27190: SAMPLE_ZH_SQUAD_HTML,
            67621: RuntimeError("429 Too Many Requests"),
            3535: SAMPLE_EN_DE_SQUAD_HTML,
        },
    )
    store = FakeTransfermarktStore(
        squad_players=[
            SquadPlayerRecord(team_id=10, player_id=1001, player_name="武磊", team_name="上海海港", shirt_number=7),
            SquadPlayerRecord(team_id=10, player_id=1002, player_name="Felipe Silva", team_name="上海海港", shirt_number=9),
        ],
        director_candidates=[
            DirectorCandidateRecord(id=501, player_name="奥斯卡", current_club="上海海港", league_tag="CSL"),
            DirectorCandidateRecord(id=502, player_name="Felipe Silva", current_club="蔚山HD", league_tag="K1"),
            DirectorCandidateRecord(id=503, player_name="Ruben Neves", current_club="蔚山HD", league_tag="K1"),
        ],
    )
    sleeps: list[float] = []

    # 1) Dry run: matches computed, zero DB writes
    dry_summary = sync_transfermarkt_valuations(
        client=client,
        store=store,
        leagues=("csl", "k1"),
        dry_run=True,
        delay=0.5,
        sleep=sleeps.append,
    )
    assert dry_summary["dry_run"] is True
    assert dry_summary["clubs_fetched"] == 2
    assert dry_summary["clubs_failed"] == 1
    assert dry_summary["players_parsed"] == 5
    assert dry_summary["squad_matches"] == 1  # Only 武磊 from CSL; Felipe Silva from K1 does not match squad
    assert dry_summary["squad_updated"] == 0
    assert dry_summary["candidate_matches"] == 3  # 奥斯卡, Felipe Silva, Ruben Neves (diacritics normalized)
    assert dry_summary["candidate_updated"] == 0
    assert store.updated_squad_rows == []
    assert store.updated_candidate_rows == []
    assert sleeps and all(s == 0.5 for s in sleeps)

    # 2) Write DB: applies updates
    write_summary = sync_transfermarkt_valuations(
        client=client,
        store=store,
        leagues=("csl", "k1"),
        dry_run=False,
        delay=0.0,
    )
    assert write_summary["dry_run"] is False
    assert write_summary["squad_updated"] == 1
    assert write_summary["candidate_updated"] == 3
    assert store.updated_squad_rows == [
        {
            "team_id": 10,
            "player_id": 1001,
            "market_value_eur": 1_200_000,
            "birth_date": date(1991, 11, 19),
            "contract_until": date(2026, 12, 31),
        }
    ]
    updated_by_id = {row["id"]: row for row in store.updated_candidate_rows}
    assert updated_by_id[501]["market_value_eur"] == 4_500_000
    assert updated_by_id[501]["contract_until"] is None
    assert updated_by_id[502]["market_value_eur"] == 1_500_000
    assert updated_by_id[503]["birth_date"] == date(1997, 3, 13)


def test_update_sql_uses_coalesce_and_sets_synced_at() -> None:
    for sql in (UPDATE_SQUAD_PLAYER_SQL, UPDATE_DIRECTOR_CANDIDATE_SQL):
        assert "COALESCE(%(market_value_eur)s, market_value_eur)" in sql
        assert "COALESCE(%(birth_date)s, birth_date)" in sql
        assert "COALESCE(%(contract_until)s, contract_until)" in sql
        assert "transfermarkt_synced_at = NOW()" in sql


def test_cli_sync_director_transfermarkt(monkeypatch: pytest.MonkeyPatch) -> None:
    runner = CliRunner()

    # Invalid league key
    bad_res = runner.invoke(app, ["sync-director-transfermarkt", "--leagues", "invalid"])
    assert bad_res.exit_code != 0

    # --write-db without DATABASE_URL fails fast
    monkeypatch.delenv("FI_DATABASE_URL", raising=False)
    monkeypatch.delenv("DATABASE_URL", raising=False)
    no_db_res = runner.invoke(app, ["sync-director-transfermarkt", "--leagues", "csl", "--write-db"])
    assert no_db_res.exit_code != 0

    # Default (no --write-db) runs in dry-run mode and prints JSON summary
    fake_client = FakeTransfermarktClient(
        league_pages={"CSL": SAMPLE_LEAGUE_HTML},
        club_pages={27190: SAMPLE_ZH_SQUAD_HTML, 67621: SAMPLE_EN_DE_SQUAD_HTML},
    )
    monkeypatch.setattr(
        "sina_csl_scraper.cli.HttpTransfermarktClient",
        lambda base_url: fake_client,
    )
    ok_res = runner.invoke(
        app,
        ["sync-director-transfermarkt", "--live", "--leagues", "csl", "--delay", "0", "--limit-clubs", "1"],
    )
    assert ok_res.exit_code == 0, ok_res.output
    payload = json.loads(ok_res.output)
    assert payload["dry_run"] is True
    assert payload["clubs_fetched"] == 1
    assert payload["players_parsed"] == 3
    assert payload["squad_matches"] == 0
    assert payload["candidate_matches"] == 0


def test_match_squad_player_falls_back_to_english_club_and_shirt_number() -> None:
    from sina_csl_scraper.director_transfermarkt_sync import canonicalize_csl_club

    assert canonicalize_csl_club("Chengdu Rongcheng") == "chengdu_rongcheng"
    assert canonicalize_csl_club("成都蓉城") == "chengdu_rongcheng"
    assert canonicalize_csl_club("Shanghai Port") == "shanghai_port"
    assert canonicalize_csl_club("上海海港") == "shanghai_port"

    inactive_old_no9 = SquadPlayerRecord(
        team_id=77680,
        team_name="成都蓉城",
        player_id=9001,
        player_name="旧九号",
        shirt_number=9,
        active=False,
    )
    active_felipe = SquadPlayerRecord(
        team_id=77680,
        team_name="成都蓉城",
        player_id=9002,
        player_name="费利佩",
        shirt_number=9,
        active=True,
    )
    squad_by_name = {
        normalize_player_name(inactive_old_no9.player_name): [inactive_old_no9],
        normalize_player_name(active_felipe.player_name): [active_felipe],
    }
    squad_by_club_shirt = {
        ("chengdu_rongcheng", 9): [inactive_old_no9, active_felipe],
    }
    tm_player = TransfermarktPlayer(
        player_name="Felipe Silva",
        club_name="Chengdu Rongcheng",
        league_key="csl",
        shirt_number=9,
        birth_date=date(1992, 4, 10),
        contract_until=date(2026, 12, 31),
        market_value_eur=350_000,
    )
    matched = match_squad_player(tm_player, squad_by_name, squad_by_club_shirt)
    assert matched is active_felipe


def test_parse_players_csv_supports_dataset_gz_and_custom_zh_csv(tmp_path) -> None:
    import gzip
    from sina_csl_scraper.director_transfermarkt_sync import parse_players_csv

    # 1) dcaribou/transfermarkt-datasets gzipped CSV format
    gz_path = tmp_path / "players.csv.gz"
    gz_content = (
        "player_id,first_name,last_name,name,current_club_name,current_club_domestic_competition_id,"
        "date_of_birth,contract_expiration_date,market_value_in_eur\n"
        "12345,Shihao,Wei,Shihao Wei,Chengdu Rongcheng,CSL,1995-04-08 00:00:00,2026-12-31 00:00:00,750000.0\n"
        "67890,,Felipe,Felipe,Chengdu Rongcheng,CSL,1992-04-10,2026-12-31,350000\n"
    )
    with gzip.open(gz_path, "wt", encoding="utf-8") as handle:
        handle.write(gz_content)

    ds_players = parse_players_csv(gz_path)
    assert len(ds_players) == 2
    assert ds_players[0].player_name == "Shihao Wei"
    assert ds_players[0].league_key == "csl"
    assert ds_players[0].birth_date == date(1995, 4, 8)
    assert ds_players[0].contract_until == date(2026, 12, 31)
    assert ds_players[0].market_value_eur == 750_000

    # 2) Simple hand-crafted Chinese CSV
    zh_path = tmp_path / "custom.csv"
    zh_path.write_text(
        "player_name,club_name,market_value_eur,birth_date,contract_until\n"
        "罗慕洛,成都蓉城,180万欧,1995-10-27,2027-12-31\n",
        encoding="utf-8",
    )
    zh_players = parse_players_csv(zh_path)
    assert len(zh_players) == 1
    assert zh_players[0].player_name == "罗慕洛"
    assert zh_players[0].market_value_eur == 1_800_000
    assert zh_players[0].birth_date == date(1995, 10, 27)


def test_parse_local_html_dir_and_sync_with_sina_english_names(tmp_path) -> None:
    from sina_csl_scraper.director_transfermarkt_sync import (
        parse_local_html_dir,
        sync_transfermarkt_from_players,
    )

    html_file = tmp_path / "chengdu_rongcheng.html"
    html_file.write_text(
        "<html><head><title>Chengdu Rongcheng - Detailed squad 2026 | Transfermarkt</title></head>"
        "<body><h1>Chengdu Rongcheng</h1>" + SAMPLE_EN_DE_SQUAD_HTML + "</body></html>",
        encoding="utf-8",
    )
    html_players = parse_local_html_dir(tmp_path)
    assert len(html_players) == 2
    assert html_players[0].club_name == "Chengdu Rongcheng"

    # Bridge English Transfermarkt dataset names to Chinese f_i_squad_players via Sina English names
    store = FakeTransfermarktStore(
        squad_players=[
            SquadPlayerRecord(team_id=77680, player_id=101, player_name="韦世豪", team_name="成都蓉城", shirt_number=7, active=True),
            SquadPlayerRecord(team_id=77680, player_id=102, player_name="费利佩", team_name="成都蓉城", shirt_number=9, active=True),
        ],
        director_candidates=[
            DirectorCandidateRecord(id=201, player_name="罗慕洛", current_club="成都蓉城", league_tag="CSL", active=True),
        ],
    )
    players = [
        # Reversed Chinese name order: Wei Shihao (Sina) <-> Shihao Wei (TM)
        TransfermarktPlayer("Shihao Wei", "Chengdu Rongcheng", "csl", None, date(1995, 4, 8), date(2026, 12, 31), 750_000),
        # Wrong random club single-word "Felipe" should be ignored
        TransfermarktPlayer("Felipe", "Some Random Club", "bra1", None, date(1999, 1, 1), date(2027, 1, 1), 99_000_000),
        # Correct Chengdu Rongcheng "Felipe" should match
        TransfermarktPlayer("Felipe", "Chengdu Rongcheng", "csl", None, date(1992, 4, 10), date(2026, 12, 31), 350_000),
        # Direct Chinese custom CSV row for candidate
        TransfermarktPlayer("罗慕洛", "成都蓉城", "csv", None, date(1995, 10, 27), date(2027, 12, 31), 1_800_000),
    ]
    summary = sync_transfermarkt_from_players(
        players=players,
        store=store,
        dry_run=False,
        source_label="test",
        sina_english_names={101: "Wei Shihao", 102: "Felipe"},
    )
    assert summary["squad_matches"] == 2
    assert summary["squad_updated"] == 2
    assert summary["candidate_matches"] == 1
    assert summary["candidate_updated"] == 1
    by_pid = {row["player_id"]: row for row in store.updated_squad_rows}
    assert by_pid[101]["market_value_eur"] == 750_000
    assert by_pid[102]["market_value_eur"] == 350_000


def test_cli_sync_director_transfermarkt_from_csv(tmp_path) -> None:
    csv_path = tmp_path / "manual.csv"
    csv_path.write_text(
        "player_name,club_name,market_value_eur,birth_date,contract_until\n"
        "费利佩,成都蓉城,35万欧,1992-04-10,2026-12-31\n",
        encoding="utf-8",
    )
    runner = CliRunner()
    res = runner.invoke(
        app,
        ["sync-director-transfermarkt", "--from-csv", str(csv_path), "--no-sina-names"],
    )
    assert res.exit_code == 0, res.output
    payload = json.loads(res.output)
    assert payload["dry_run"] is True
    assert payload["players_loaded"] == 1
    assert payload["source"] == f"csv:{csv_path}"



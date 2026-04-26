from __future__ import annotations

from sina_csl_scraper.models import MatchResult


def build_match() -> MatchResult:
    return MatchResult(
        match_id=288579,
        season=2026,
        round_number=1,
        round_name="第1轮",
        date="2026-03-08",
        time="19:00",
        status="3",
        home_team_id=110645,
        home_team_name="武汉三镇",
        home_score="0",
        away_team_id=136,
        away_team_name="北京国安",
        away_score="2",
        home_logo="home.png",
        away_logo="away.png",
    )


def test_match_result_supports_corner_fields() -> None:
    match = MatchResult(
        match_id=288579,
        season=2026,
        round_number=1,
        round_name="第1轮",
        date="2026-03-08",
        time="19:00",
        status="3",
        home_team_id=110645,
        home_team_name="武汉三镇",
        home_score="0",
        away_team_id=136,
        away_team_name="北京国安",
        away_score="2",
        home_logo="home.png",
        away_logo="away.png",
        leisu_match_id=4422785,
        home_corners=7,
        away_corners=3,
        corner_source="leisu_detail",
        technical_stats=[
            {
                "slug": "corners",
                "label": "角球",
                "home_value": 7,
                "away_value": 3,
                "unit": None,
            }
        ],
    )

    payload = match.to_dict()

    assert payload["leisu_match_id"] == 4422785
    assert payload["home_corners"] == 7
    assert payload["away_corners"] == 3
    assert payload["corner_source"] == "leisu_detail"
    assert payload["technical_stats"] == [
        {
            "slug": "corners",
            "label": "角球",
            "home_value": 7,
            "away_value": 3,
            "unit": None,
        }
    ]


def test_build_leisu_urls_follow_known_patterns() -> None:
    from sina_csl_scraper.leisu import LeisuClient

    client = LeisuClient()

    assert client.build_live_url() == "https://live.leisu.com/"
    assert client.build_finished_url("2026-04-17") == "https://live.leisu.com/wanchang-20260417"
    assert client.build_finished_url("20260417") == "https://live.leisu.com/wanchang-20260417"
    assert client.build_detail_url(4422785) == "https://live.leisu.com/detail-4422785"


def test_parse_detail_page_counts_corners_from_corner_events() -> None:
    from sina_csl_scraper.leisu import LeisuClient

    html = """
    <html>
      <body>
        <div>图例说明 角球 进球 点球 控球率 黄牌 红牌</div>
        <div>比赛第8分钟，北京国安取得了本场比赛的第1个角球</div>
        <div>第2个角球 - (武汉三镇)</div>
        <div>第3个角球，本场比赛的第3个角球已经产生！</div>
        <div>第4个角球 - (北京国安)</div>
      </body>
    </html>
    """

    client = LeisuClient()

    home_corners, away_corners = client.parse_detail_corners(
        html,
        home_team_name="武汉三镇",
        away_team_name="北京国安",
    )

    assert home_corners == 1
    assert away_corners == 2


def test_parse_detail_page_extracts_structured_technical_stats() -> None:
    from sina_csl_scraper.leisu import LeisuClient

    html = """
    <div class="technical-statistics">
      <div class="ts-top">
        <div class="ts-t-left">
          <span class="lab corner"><span class="text">6</span></span>
          <span class="lab card-red"><span class="text">0</span></span>
          <span class="lab card-yellow"><span class="text">4</span></span>
        </div>
        <div class="ts-t-center">
          <span class="bar-panel m-l-48">
            <span class="left"><span class="num"><span class="tnum">58%</span></span></span>
            <span class="barcenter">控球率</span>
            <span class="right"><span class="num"><span class="tnum">42%</span></span></span>
          </span>
        </div>
        <div class="ts-t-right">
          <span class="lab card-yellow"><span class="text">1</span></span>
          <span class="lab card-red"><span class="text">0</span></span>
          <span class="lab corner"><span class="text">3</span></span>
        </div>
      </div>
      <div class="ts-bottom">
        <div class="children">
          <span class="bar-panel">
            <span class="left"><span class="num"><span class="tnum">101</span></span></span>
            <span class="barcenter">进攻</span>
            <span class="right"><span class="num"><span class="tnum">73</span></span></span>
          </span>
          <span class="bar-panel">
            <span class="left"><span class="num"><span class="tnum">68</span></span></span>
            <span class="barcenter">危险进攻</span>
            <span class="right"><span class="num"><span class="tnum">44</span></span></span>
          </span>
        </div>
        <div class="children">
          <span class="bar-panel shoot">
            <span class="left"><span class="num"><span class="tnum">11(3)</span></span></span>
            <span class="barcenter">射门(射正)</span>
            <span class="right"><span class="num"><span class="tnum">7(2)</span></span></span>
          </span>
          <span class="bar-panel">
            <span class="left"><span class="num"><span class="tnum">1</span></span></span>
            <span class="barcenter">点球</span>
            <span class="right"><span class="num"><span class="tnum">0</span></span></span>
          </span>
        </div>
      </div>
    </div>
    <div id="discussion"></div>
    """

    client = LeisuClient()

    stats = client.parse_detail_technical_stats(html)

    assert [(stat.slug, stat.home_value, stat.away_value, stat.unit) for stat in stats] == [
        ("attacks", 101, 73, None),
        ("yellow_cards", 4, 1, None),
        ("dangerous_attacks", 68, 44, None),
        ("shots_on_target", 3, 2, None),
        ("red_cards", 0, 0, None),
        ("possession", 58, 42, "%"),
        ("penalties", 1, 0, None),
        ("shots_off_target", 8, 5, None),
        ("corners", 6, 3, None),
    ]


def test_parse_detail_page_ignores_placeholder_zero_technical_stats() -> None:
    from sina_csl_scraper.leisu import LeisuClient

    html = """
    <div class="technical-statistics">
      <div class="ts-top">
        <div class="ts-t-left">
          <span class="lab corner"><span class="text">0</span></span>
          <span class="lab card-red"><span class="text">0</span></span>
          <span class="lab card-yellow"><span class="text">0</span></span>
        </div>
        <div class="ts-t-center">
          <span class="bar-panel m-l-48">
            <span class="left"><span class="num"><span class="tnum">50%</span></span></span>
            <span class="barcenter">控球率</span>
            <span class="right"><span class="num"><span class="tnum">50%</span></span></span>
          </span>
        </div>
        <div class="ts-t-right">
          <span class="lab card-yellow"><span class="text">0</span></span>
          <span class="lab card-red"><span class="text">0</span></span>
          <span class="lab corner"><span class="text">0</span></span>
        </div>
      </div>
      <div class="ts-bottom">
        <div class="children">
          <span class="bar-panel">
            <span class="left"><span class="num"><span class="tnum">0</span></span></span>
            <span class="barcenter">进攻</span>
            <span class="right"><span class="num"><span class="tnum">0</span></span></span>
          </span>
          <span class="bar-panel">
            <span class="left"><span class="num"><span class="tnum">0</span></span></span>
            <span class="barcenter">危险进攻</span>
            <span class="right"><span class="num"><span class="tnum">0</span></span></span>
          </span>
        </div>
        <div class="children">
          <span class="bar-panel shoot">
            <span class="left"><span class="num"><span class="tnum">0(0)</span></span></span>
            <span class="barcenter">射门(射正)</span>
            <span class="right"><span class="num"><span class="tnum">0(0)</span></span></span>
          </span>
          <span class="bar-panel">
            <span class="left"><span class="num"><span class="tnum">0</span></span></span>
            <span class="barcenter">点球</span>
            <span class="right"><span class="num"><span class="tnum">0</span></span></span>
          </span>
        </div>
      </div>
    </div>
    <div id="discussion"></div>
    """

    client = LeisuClient()

    assert client.parse_detail_technical_stats(html) == []


def test_parse_detail_page_matches_csl_sponsored_team_names() -> None:
    from sina_csl_scraper.leisu import LeisuClient

    html = """
    <html>
      <body>
        <div>比赛第11分钟，大连英博海发取得了本场比赛的第1个角球</div>
        <div>第2个角球 (河南俱乐部彩陶坊)</div>
        <div>第3个角球 (大连英博海发)</div>
      </body>
    </html>
    """

    client = LeisuClient()

    home_corners, away_corners = client.parse_detail_corners(
        html,
        home_team_name="大连英博",
        away_team_name="河南队",
    )

    assert home_corners == 2
    assert away_corners == 1


def test_match_candidate_detail_id_handles_shenzhen_simplified_and_traditional_names() -> None:
    from sina_csl_scraper.leisu import LeisuMatchCandidate, _match_candidate_detail_id

    candidates = [
        LeisuMatchCandidate(
            detail_id=4498779,
            home_team_name="深圳新鹏城",
            home_score="",
            away_score="",
            away_team_name="北京国安",
        )
    ]

    match = MatchResult(
        match_id=288620,
        season=2026,
        round_number=7,
        round_name="第7轮",
        date="2026-04-21",
        time="19:00",
        status="2",
        home_team_id=1,
        home_team_name="深圳新鵬城",
        home_score="0",
        away_team_id=2,
        away_team_name="北京国安",
        away_score="0",
        home_logo="home.png",
        away_logo="away.png",
    )

    assert _match_candidate_detail_id(candidates, match) == 4498779


def test_parse_match_candidates_extracts_detail_ids_teams_and_scores() -> None:
    from sina_csl_scraper.leisu import LeisuClient

    html = """
    <div class="match-card">
      <span class="home">武汉三镇</span>
      <span class="score">0-2</span>
      <span class="away">北京国安</span>
      <a href="/detail-4422785">数据</a>
    </div>
    <div class="match-card">
      <span class="home">成都蓉城</span>
      <span class="score">5-1</span>
      <span class="away">深圳新鹏城</span>
      <a href="/detail-4422786">数据</a>
    </div>
    """

    client = LeisuClient()
    candidates = client.parse_match_candidates(html)

    assert [(item.detail_id, item.home_team_name, item.home_score, item.away_score, item.away_team_name) for item in candidates] == [
        (4422785, "武汉三镇", "0", "2", "北京国安"),
        (4422786, "成都蓉城", "5", "1", "深圳新鹏城"),
    ]


def test_corner_enricher_applies_detail_page_corners_from_override_map() -> None:
    from sina_csl_scraper.leisu import LeisuClient, LeisuCornerEnricher

    class FakeSession:
        def get(self, url, timeout):
            assert url == "https://live.leisu.com/detail-4422785"

            class Response:
                text = """
                <div class="technical-statistics">
                  <div class="ts-top">
                    <div class="ts-t-left">
                      <span class="lab corner"><span class="text">1</span></span>
                      <span class="lab card-red"><span class="text">0</span></span>
                      <span class="lab card-yellow"><span class="text">2</span></span>
                    </div>
                    <div class="ts-t-center">
                      <span class="bar-panel m-l-48">
                        <span class="left"><span class="num"><span class="tnum">40%</span></span></span>
                        <span class="barcenter">控球率</span>
                        <span class="right"><span class="num"><span class="tnum">60%</span></span></span>
                      </span>
                    </div>
                    <div class="ts-t-right">
                      <span class="lab card-yellow"><span class="text">1</span></span>
                      <span class="lab card-red"><span class="text">0</span></span>
                      <span class="lab corner"><span class="text">2</span></span>
                    </div>
                  </div>
                  <div class="ts-bottom">
                    <div class="children">
                      <span class="bar-panel">
                        <span class="left"><span class="num"><span class="tnum">88</span></span></span>
                        <span class="barcenter">进攻</span>
                        <span class="right"><span class="num"><span class="tnum">109</span></span></span>
                      </span>
                    </div>
                  </div>
                </div>
                """

                def raise_for_status(self) -> None:
                    return None

            return Response()

    enricher = LeisuCornerEnricher(
        client=LeisuClient(session=FakeSession()),
        match_id_map={288579: 4422785},
    )

    enriched = enricher.enrich_matches([build_match()])

    assert enriched[0].leisu_match_id == 4422785
    assert enriched[0].home_corners == 1
    assert enriched[0].away_corners == 2
    assert enriched[0].corner_source == "leisu_detail"
    assert [(item["slug"], item["home_value"], item["away_value"]) for item in enriched[0].technical_stats or []] == [
        ("attacks", 88, 109),
        ("yellow_cards", 2, 1),
        ("red_cards", 0, 0),
        ("possession", 40, 60),
        ("corners", 1, 2),
    ]


def test_corner_enricher_discovers_finished_match_detail_id_from_finished_page() -> None:
    from sina_csl_scraper.leisu import LeisuClient, LeisuCornerEnricher

    class FakeSession:
        def __init__(self) -> None:
            self.calls: list[str] = []
            self.headers = {}

        def get(self, url, timeout):
            self.calls.append(url)

            class Response:
                def __init__(self, text: str) -> None:
                    self.text = text

                def raise_for_status(self) -> None:
                    return None

            if url == "https://live.leisu.com/wanchang-20260308":
                return Response(
                    """
                    <div class="match-card">
                      <span>武汉三镇</span>
                      <span>0-2</span>
                      <span>北京国安</span>
                      <a href="/detail-4422785">数据</a>
                    </div>
                    """
                )
            if url == "https://live.leisu.com/detail-4422785":
                return Response(
                    """
                    <div>第1个角球 - (北京国安)</div>
                    <div>第2个角球 - (武汉三镇)</div>
                    <div>第3个角球 - (北京国安)</div>
                    """
                )
            raise AssertionError(url)

    enricher = LeisuCornerEnricher(client=LeisuClient(session=FakeSession()))

    enriched = enricher.enrich_matches([build_match()])

    assert enriched[0].leisu_match_id == 4422785
    assert enriched[0].home_corners == 1
    assert enriched[0].away_corners == 2


def test_corner_enricher_discovers_live_match_detail_id_from_live_page() -> None:
    from sina_csl_scraper.leisu import LeisuClient, LeisuCornerEnricher

    class FakeSession:
        def __init__(self) -> None:
            self.calls: list[str] = []
            self.headers = {}

        def get(self, url, timeout):
            self.calls.append(url)

            class Response:
                def __init__(self, text: str) -> None:
                    self.text = text

                def raise_for_status(self) -> None:
                    return None

            if url == "https://live.leisu.com/":
                return Response(
                    """
                    <div class="match-card">
                      <span>武汉三镇</span>
                      <span>1-0</span>
                      <span>北京国安</span>
                      <a href="/detail-5522785">数据</a>
                    </div>
                    """
                )
            if url == "https://live.leisu.com/detail-5522785":
                return Response(
                    """
                    <div>第1个角球 - (武汉三镇)</div>
                    <div>第2个角球 - (北京国安)</div>
                    """
                )
            raise AssertionError(url)

    enricher = LeisuCornerEnricher(client=LeisuClient(session=FakeSession()))
    live_match = MatchResult(
        match_id=288579,
        season=2026,
        round_number=1,
        round_name="第1轮",
        date="2026-03-08",
        time="19:00",
        status="2",
        home_team_id=110645,
        home_team_name="武汉三镇",
        home_score="1",
        away_team_id=136,
        away_team_name="北京国安",
        away_score="0",
        home_logo="home.png",
        away_logo="away.png",
    )

    enriched = enricher.enrich_matches([live_match])

    assert enriched[0].leisu_match_id == 5522785
    assert enriched[0].home_corners == 1
    assert enriched[0].away_corners == 1


def test_corner_enricher_retries_candidate_fetch_when_cached_page_misses_match() -> None:
    from sina_csl_scraper.leisu import LeisuClient, LeisuCornerEnricher, LeisuMatchCandidate

    class FakeClient(LeisuClient):
        def __init__(self) -> None:
            super().__init__()
            self.fetch_calls = 0

        def build_live_url(self) -> str:
            return "https://live.leisu.com/"

        def fetch_match_candidates(self, url: str) -> list[LeisuMatchCandidate]:
            self.fetch_calls += 1
            if self.fetch_calls == 1:
                return []
            return [
                LeisuMatchCandidate(
                    detail_id=4498779,
                    home_team_name="深圳新鹏城",
                    home_score="",
                    away_score="",
                    away_team_name="北京国安",
                )
            ]

        def fetch_detail_page(self, detail_id: int) -> str:
            assert detail_id == 4498779
            return """
            <div class="technical-statistics">
              <div class="ts-top">
                <div class="ts-t-left">
                  <span class="lab corner"><span class="text">3</span></span>
                </div>
                <div class="ts-t-right">
                  <span class="lab corner"><span class="text">7</span></span>
                </div>
              </div>
            </div>
            """

    match = MatchResult(
        match_id=288620,
        season=2026,
        round_number=7,
        round_name="第7轮",
        date="2026-04-21",
        time="19:00",
        status="2",
        home_team_id=1,
        home_team_name="深圳新鵬城",
        home_score="0",
        away_team_id=2,
        away_team_name="北京国安",
        away_score="0",
        home_logo="home.png",
        away_logo="away.png",
    )

    client = FakeClient()
    enricher = LeisuCornerEnricher(client=client)

    enriched = enricher.enrich_matches([match])

    assert client.fetch_calls == 2
    assert enriched[0].leisu_match_id == 4498779
    assert enriched[0].home_corners == 3
    assert enriched[0].away_corners == 7


def test_browser_client_retries_until_waf_challenge_clears() -> None:
    from sina_csl_scraper.leisu import LeisuBrowserClient

    challenge_html = """
    <html>
      <head>
        <meta name="aliyun_waf_aa" content="challenge" />
      </head>
      <body>
        <textarea id="renderData">{"l2":"GET"}</textarea>
      </body>
    </html>
    """
    live_html = """
    <div class="match-card">
      <span>武汉三镇</span>
      <span>0-2</span>
      <span>北京国安</span>
      <a href="/detail-4422785">数据</a>
    </div>
    """

    class FakeBrowserSession:
        def __init__(self) -> None:
            self.contents = [challenge_html, live_html]
            self.visited_urls: list[str] = []

        def fetch_html(self, url: str) -> str:
            self.visited_urls.append(url)
            return self.contents.pop(0)

    session = FakeBrowserSession()
    client = LeisuBrowserClient(browser_session=session, max_attempts=2)

    html = client.fetch_page("https://live.leisu.com/")

    assert html == live_html
    assert session.visited_urls == [
        "https://live.leisu.com/",
        "https://live.leisu.com/",
    ]


def test_corner_enricher_uses_browser_client_for_finished_page_and_detail_page() -> None:
    from sina_csl_scraper.leisu import LeisuBrowserClient, LeisuCornerEnricher

    class FakeBrowserSession:
        def __init__(self) -> None:
            self.responses = {
                "https://live.leisu.com/wanchang-20260308": """
                    <div class="match-card">
                      <span>武汉三镇</span>
                      <span>0-2</span>
                      <span>北京国安</span>
                      <a href="/detail-4422785">数据</a>
                    </div>
                """,
                "https://live.leisu.com/detail-4422785": """
                    <div>第1个角球 - (北京国安)</div>
                    <div>第2个角球 - (武汉三镇)</div>
                    <div>第3个角球 - (北京国安)</div>
                """,
            }
            self.visited_urls: list[str] = []

        def fetch_html(self, url: str) -> str:
            self.visited_urls.append(url)
            return self.responses[url]

    session = FakeBrowserSession()
    enricher = LeisuCornerEnricher(client=LeisuBrowserClient(browser_session=session))

    enriched = enricher.enrich_matches([build_match()])

    assert enriched[0].leisu_match_id == 4422785
    assert enriched[0].home_corners == 1
    assert enriched[0].away_corners == 2
    assert session.visited_urls == [
        "https://live.leisu.com/wanchang-20260308",
        "https://live.leisu.com/detail-4422785",
    ]


def test_browser_client_prefers_structured_match_candidates_from_browser_session() -> None:
    from sina_csl_scraper.leisu import LeisuBrowserClient

    class FakeBrowserSession:
        def fetch_match_candidates(self, url: str, *, competition_name: str) -> list[dict[str, str | int]]:
            assert url == "https://live.leisu.com/wanchang-20260308"
            assert competition_name == "中超"
            return [
                {
                    "detail_id": 4498709,
                    "home_team_name": "武汉三镇",
                    "away_team_name": "北京国安",
                }
            ]

    client = LeisuBrowserClient(browser_session=FakeBrowserSession())

    candidates = client.fetch_match_candidates("https://live.leisu.com/wanchang-20260308")

    assert len(candidates) == 1
    assert candidates[0].detail_id == 4498709
    assert candidates[0].home_team_name == "武汉三镇"
    assert candidates[0].away_team_name == "北京国安"

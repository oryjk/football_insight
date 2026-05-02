from __future__ import annotations

import logging
import re
from collections.abc import Mapping
from dataclasses import dataclass, replace
from datetime import date, datetime
import json
from pathlib import Path
from typing import Any

import requests

from .models import MatchResult, MatchTechnicalStat

logger = logging.getLogger(__name__)

DEFAULT_LEISU_USER_AGENT = (
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) "
    "AppleWebKit/537.36 (KHTML, like Gecko) "
    "Chrome/135.0.0.0 Safari/537.36"
)

CSL_TEAM_ALIASES: dict[str, tuple[str, ...]] = {
    "上海海港": ("上海海港", "上海海港俱乐部"),
    "上海申花": ("上海申花", "上海申花俱乐部"),
    "北京国安": ("北京国安", "北京国安俱乐部"),
    "成都蓉城": ("成都蓉城", "成都蓉城俱乐部"),
    "重庆铜梁龙": ("重庆铜梁龙", "重庆铜梁龙俱乐部"),
    "云南玉昆": ("云南玉昆", "云南玉昆俱乐部"),
    "天津津门虎": ("天津津门虎", "天津津门虎俱乐部"),
    "大连英博": ("大连英博", "大连英博海发"),
    "山东泰山": ("山东泰山", "山东泰山俱乐部"),
    "深圳新鵬城": ("深圳新鵬城", "深圳新鹏城", "深圳新鹏城足球俱乐部"),
    "浙江队": ("浙江队", "浙江俱乐部", "浙江俱乐部绿城"),
    "武汉三镇": ("武汉三镇", "武汉三镇俱乐部"),
    "河南队": ("河南队", "河南俱乐部", "河南俱乐部彩陶坊"),
    "辽宁铁人": ("辽宁铁人", "辽宁铁人农商银行"),
    "青岛海牛": ("青岛海牛", "青岛海牛俱乐部"),
    "青岛西海岸": ("青岛西海岸", "青岛西海岸俱乐部"),
}


class LeisuApiError(RuntimeError):
    """Raised when Leisu pages cannot be fetched."""


@dataclass(frozen=True)
class LeisuMatchCandidate:
    detail_id: int
    home_team_name: str
    home_score: str
    away_score: str
    away_team_name: str


class LeisuClient:
    def __init__(
        self,
        base_url: str = "https://live.leisu.com",
        timeout: int = 20,
        session: requests.Session | None = None,
    ) -> None:
        self.base_url = base_url.rstrip("/")
        self.timeout = timeout
        self.session = session or requests.Session()
        headers = getattr(self.session, "headers", None)
        if headers is not None:
            headers.update({"User-Agent": DEFAULT_LEISU_USER_AGENT})

    def build_live_url(self) -> str:
        return f"{self.base_url}/"

    def build_finished_url(self, value: str | date | datetime) -> str:
        if isinstance(value, datetime):
            normalized = value.date().strftime("%Y%m%d")
        elif isinstance(value, date):
            normalized = value.strftime("%Y%m%d")
        else:
            normalized = value.replace("-", "").strip()
        return f"{self.base_url}/wanchang-{normalized}"

    def build_detail_url(self, detail_id: int) -> str:
        return f"{self.base_url}/detail-{detail_id}"

    def fetch_detail_page(self, detail_id: int) -> str:
        response = self.session.get(
            self.build_detail_url(detail_id),
            timeout=self.timeout,
        )
        response.raise_for_status()
        return response.text

    def fetch_page(self, url: str) -> str:
        response = self.session.get(url, timeout=self.timeout)
        response.raise_for_status()
        return response.text

    def close(self) -> None:
        close = getattr(self.session, "close", None)
        if callable(close):
            close()

    def parse_match_candidates(self, html: str) -> list[LeisuMatchCandidate]:
        candidates: list[LeisuMatchCandidate] = []
        block_matches = re.findall(r"(?is)<div[^>]*match-card[^>]*>(.*?)</div>", html)
        if block_matches:
            for block in block_matches:
                detail_match = re.search(r"/detail-(\d+)", block)
                if detail_match is None:
                    continue
                parsed = _parse_candidate_window(_strip_html(block))
                if parsed is None:
                    continue
                home_team_name, home_score, away_score, away_team_name = parsed
                candidates.append(
                    LeisuMatchCandidate(
                        detail_id=int(detail_match.group(1)),
                        home_team_name=home_team_name,
                        home_score=home_score,
                        away_score=away_score,
                        away_team_name=away_team_name,
                    )
                )
            return _dedupe_candidates(candidates)

        for match in re.finditer(r"/detail-(\d+)", html):
            detail_id = int(match.group(1))
            start = max(0, match.start() - 600)
            end = min(len(html), match.end() + 600)
            window_text = _strip_html(html[start:end])
            parsed = _parse_candidate_window(window_text)
            if parsed is None:
                continue

            home_team_name, home_score, away_score, away_team_name = parsed
            candidates.append(
                LeisuMatchCandidate(
                    detail_id=detail_id,
                    home_team_name=home_team_name,
                    home_score=home_score,
                    away_score=away_score,
                    away_team_name=away_team_name,
                )
            )
        return _dedupe_candidates(candidates)

    def parse_detail_corners(
        self,
        html: str,
        *,
        home_team_name: str,
        away_team_name: str,
    ) -> tuple[int | None, int | None]:
        technical_stats = self.parse_detail_technical_stats(html)
        corners = _find_stat_values(technical_stats, "corners")
        if corners is not None:
            return corners

        home_count = 0
        away_count = 0

        for team_name in self._extract_corner_event_teams(html):
            if _team_names_match(team_name, home_team_name):
                home_count += 1
            elif _team_names_match(team_name, away_team_name):
                away_count += 1

        if home_count == 0 and away_count == 0:
            return (None, None)
        return (home_count, away_count)

    def parse_detail_technical_stats(self, html: str) -> list[MatchTechnicalStat]:
        block = _extract_technical_statistics_block(html)
        if block is None:
            return []

        top_values = _parse_top_technical_values(block)
        panel_values = _parse_bar_panel_values(block)

        stats: list[MatchTechnicalStat] = []

        for slug, label in (
            ("attacks", "进攻"),
            ("yellow_cards", "黄牌"),
            ("dangerous_attacks", "危险进攻"),
            ("shots_on_target", "射正"),
            ("red_cards", "红牌"),
            ("possession", "控球率"),
            ("penalties", "点球"),
            ("shots_off_target", "射偏"),
            ("corners", "角球"),
        ):
            values = _resolve_technical_stat_values(slug, top_values=top_values, panel_values=panel_values)
            if values is None:
                continue

            home_value, away_value, unit = values
            stats.append(
                MatchTechnicalStat(
                    slug=slug,
                    label=label,
                    home_value=home_value,
                    away_value=away_value,
                    unit=unit,
                )
            )

        if _looks_like_placeholder_technical_stats(stats):
            return []

        return stats

    @staticmethod
    def _extract_corner_event_teams(html: str) -> list[str]:
        normalized = _strip_html(html)
        patterns = (
            re.compile(r"第\d+个角球\s*-\s*[（(]([^()（）]+)[)）]"),
            re.compile(r"第\d+个角球\s*[（(]([^()（）]+)[)）]"),
            re.compile(r"比赛第\d+分钟[，,]\s*([^，。,.\n]+?)取得了本场比赛的第\d+个角球"),
        )

        teams: list[str] = []
        for pattern in patterns:
            teams.extend(pattern.findall(normalized))
        return teams


class LeisuPlaywrightSession:
    def __init__(
        self,
        *,
        headless: bool = True,
        timeout: int = 20,
        user_agent: str = DEFAULT_LEISU_USER_AGENT,
    ) -> None:
        self.headless = headless
        self.timeout = timeout
        self.user_agent = user_agent
        self._playwright: Any | None = None
        self._browser: Any | None = None
        self._context: Any | None = None
        self._page: Any | None = None

    def fetch_html(self, url: str) -> str:
        page = self._ensure_page()
        try:
            page.goto(url, wait_until="domcontentloaded", timeout=self.timeout * 1000)
            page.wait_for_timeout(1500)
            page.wait_for_load_state("networkidle", timeout=self.timeout * 1000)
        except Exception:
            # Some Leisu pages keep background requests open; DOM content is enough for parsing.
            pass
        return page.content()

    def fetch_match_candidates(self, url: str, *, competition_name: str) -> list[dict[str, str | int]]:
        page = self._ensure_page()
        page.goto(url, wait_until="domcontentloaded", timeout=self.timeout * 1000)
        page.wait_for_timeout(1500)
        if "/detail-" not in url:
            self._try_click_competition_filter(page, competition_name)
        raw_candidates = page.evaluate(
            """
            () => Array.from(document.querySelectorAll('[id^="match_layout_"]')).map((row) => {
              const anchors = Array.from(row.querySelectorAll('a'));
              const competitionLink = anchors.find((anchor) => anchor.classList.contains('event-name'));
              const teamLinks = anchors.filter((anchor) => anchor.classList.contains('name'));
              const detailLink = anchors.find((anchor) => anchor.href.includes('/detail-') && anchor.textContent.trim() === '数据')
                || anchors.find((anchor) => anchor.href.includes('/detail-'));
              const detailMatch = detailLink?.href.match(/detail-(\\d+)/);
              return {
                competition_name: competitionLink?.textContent?.trim() || '',
                detail_id: detailMatch ? Number(detailMatch[1]) : null,
                home_team_name: teamLinks[0]?.textContent?.trim() || '',
                away_team_name: teamLinks[1]?.textContent?.trim() || '',
              };
            })
            """
        )
        return [
            candidate
            for candidate in raw_candidates
            if candidate.get("detail_id")
            and candidate.get("home_team_name")
            and candidate.get("away_team_name")
            and candidate.get("competition_name") == competition_name
        ]

    def close(self) -> None:
        for resource in (self._page, self._context, self._browser):
            if resource is None:
                continue
            try:
                resource.close()
            except Exception:
                pass
        if self._playwright is not None:
            try:
                self._playwright.stop()
            except Exception:
                pass
        self._page = None
        self._context = None
        self._browser = None
        self._playwright = None

    def _ensure_page(self) -> Any:
        if self._page is not None:
            return self._page

        try:
            from playwright.sync_api import sync_playwright
        except ImportError as exc:
            raise LeisuApiError(
                "Playwright is required for browser-based Leisu scraping. "
                "Install dependencies and run `uv run playwright install chromium`."
            ) from exc

        self._playwright = sync_playwright().start()
        self._browser = self._playwright.chromium.launch(
            channel="chromium",
            headless=self.headless,
            args=["--disable-blink-features=AutomationControlled", "--no-sandbox"],
        )
        self._context = self._browser.new_context(
            user_agent=self.user_agent,
            locale="zh-CN",
            viewport={"width": 1440, "height": 2400},
        )
        self._page = self._context.new_page()
        self._page.set_default_timeout(self.timeout * 1000)
        return self._page

    @staticmethod
    def _try_click_competition_filter(page: Any, competition_name: str) -> None:
        try:
            page.get_by_text(competition_name, exact=True).click(timeout=3000)
            page.wait_for_timeout(1500)
        except Exception:
            return None


class LeisuBrowserClient(LeisuClient):
    def __init__(
        self,
        base_url: str = "https://live.leisu.com",
        timeout: int = 20,
        *,
        browser_session: Any | None = None,
        headless: bool = True,
        max_attempts: int = 3,
    ) -> None:
        super().__init__(base_url=base_url, timeout=timeout)
        self.browser_session = browser_session or LeisuPlaywrightSession(
            headless=headless,
            timeout=timeout,
        )
        self.max_attempts = max_attempts

    def fetch_detail_page(self, detail_id: int) -> str:
        return self._fetch_html_with_browser(self.build_detail_url(detail_id))

    def fetch_page(self, url: str) -> str:
        return self._fetch_html_with_browser(url)

    def close(self) -> None:
        close = getattr(self.browser_session, "close", None)
        if callable(close):
            close()
        super().close()

    def warmup(self) -> None:
        ensure_ready = getattr(self.browser_session, "_ensure_page", None)
        if callable(ensure_ready):
            ensure_ready()

    def fetch_match_candidates(self, url: str) -> list[LeisuMatchCandidate]:
        fetch_candidates = getattr(self.browser_session, "fetch_match_candidates", None)
        if callable(fetch_candidates):
            raw_candidates = fetch_candidates(url, competition_name="中超")
            return [
                LeisuMatchCandidate(
                    detail_id=int(candidate["detail_id"]),
                    home_team_name=str(candidate["home_team_name"]),
                    home_score="",
                    away_score="",
                    away_team_name=str(candidate["away_team_name"]),
                )
                for candidate in raw_candidates
            ]
        return self.parse_match_candidates(self.fetch_page(url))

    def _fetch_html_with_browser(self, url: str) -> str:
        html = ""
        for _ in range(self.max_attempts):
            html = self.browser_session.fetch_html(url)
            if not _looks_like_waf_challenge(html):
                return html
        raise LeisuApiError(f"Leisu page remained behind WAF challenge: {url}")


class LeisuCornerEnricher:
    def __init__(
        self,
        *,
        client: LeisuClient,
        match_id_map: Mapping[int, int] | None = None,
    ) -> None:
        self.client = client
        self.match_id_map = dict(match_id_map or {})
        self._candidate_cache: dict[str, list[LeisuMatchCandidate]] = {}

    def enrich_matches(self, matches: list[MatchResult]) -> list[MatchResult]:
        enriched_matches: list[MatchResult] = []
        for match in matches:
            leisu_match_id = self.match_id_map.get(match.match_id) or self._discover_detail_id(match)
            if leisu_match_id is None:
                logger.warning(
                    "Leisu detail_id not found for match %s (%s vs %s)",
                    match.match_id,
                    match.home_team_name,
                    match.away_team_name,
                )
                enriched_matches.append(match)
                continue

            try:
                detail_html = self.client.fetch_detail_page(leisu_match_id)
                technical_stats = self.client.parse_detail_technical_stats(detail_html)
                corners = _find_stat_values(technical_stats, "corners")
                if corners is None:
                    corners = self.client.parse_detail_corners(
                        detail_html,
                        home_team_name=match.home_team_name,
                        away_team_name=match.away_team_name,
                    )
                home_corners, away_corners = corners
            except Exception:
                logger.exception(
                    "Failed to enrich match %s (%s vs %s) from leisu detail %s",
                    match.match_id,
                    match.home_team_name,
                    match.away_team_name,
                    leisu_match_id,
                )
                enriched_matches.append(match)
                continue

            enriched_matches.append(
                replace(
                    match,
                    leisu_match_id=leisu_match_id,
                    home_corners=home_corners,
                    away_corners=away_corners,
                    corner_source=(
                        "leisu_detail"
                        if technical_stats or home_corners is not None or away_corners is not None
                        else None
                    ),
                    technical_stats=[item.to_dict() for item in technical_stats] or None,
                )
            )
        return enriched_matches

    def _discover_detail_id(self, match: MatchResult) -> int | None:
        page_url = self.client.build_finished_url(match.date) if match.status == "3" else self.client.build_live_url()
        candidates = self._candidate_cache.get(page_url)
        if candidates is None:
            candidates = self._refresh_candidate_cache(page_url)
            if candidates is None:
                return None

        detail_id = _match_candidate_detail_id(candidates, match)
        if detail_id is not None:
            return detail_id

        refreshed_candidates = self._refresh_candidate_cache(page_url)
        if refreshed_candidates is None:
            return None
        return _match_candidate_detail_id(refreshed_candidates, match)

    def _refresh_candidate_cache(self, page_url: str) -> list[LeisuMatchCandidate] | None:
        try:
            fetch_candidates = getattr(self.client, "fetch_match_candidates", None)
            if callable(fetch_candidates):
                candidates = fetch_candidates(page_url)
            else:
                page_html = self.client.fetch_page(page_url)
                candidates = self.client.parse_match_candidates(page_html)
        except Exception:
            logger.exception("Failed to refresh Leisu candidate cache from %s", page_url)
            return None
        self._candidate_cache[page_url] = candidates
        return candidates

    def close(self) -> None:
        close = getattr(self.client, "close", None)
        if callable(close):
            close()


def load_leisu_match_map(path: Path) -> dict[int, int]:
    payload = json.loads(path.read_text(encoding="utf-8"))
    return {int(match_id): int(detail_id) for match_id, detail_id in payload.items()}


def _normalize_team_name(value: str) -> str:
    return re.sub(r"\s+", "", value).strip().lower()


def _team_names_match(source: str, target: str) -> bool:
    normalized_source = _normalize_team_name(source)
    normalized_target = _normalize_team_name(target)
    if normalized_source == normalized_target:
        return True

    aliases = {
        _normalize_team_name(alias)
        for alias in CSL_TEAM_ALIASES.get(target, ())
    }
    if normalized_source in aliases:
        return True

    if normalized_target in normalized_source or normalized_source in normalized_target:
        return True

    return False


def _strip_html(html: str) -> str:
    text = re.sub(r"<[^>]+>", " ", html)
    text = text.replace("&nbsp;", " ")
    return re.sub(r"\s+", " ", text)


def _looks_like_waf_challenge(html: str) -> bool:
    normalized = html.lower()
    waf_markers = (
        'meta name="aliyun_waf_aa"',
        'meta name="aliyun_waf_bb"',
        'id="renderdata"',
        "acw_sc__v2",
    )
    if any(marker in normalized for marker in waf_markers):
        return True
    return False


def _parse_candidate_window(window_text: str) -> tuple[str, str, str, str] | None:
    score_match = re.search(r"(\d+)\s*[-:]\s*(\d+)", window_text)
    if score_match is None:
        return None

    left_tokens = _extract_team_tokens(window_text[: score_match.start()])
    right_tokens = _extract_team_tokens(window_text[score_match.end() :])
    if not left_tokens or not right_tokens:
        return None

    return (
        left_tokens[-1],
        score_match.group(1),
        score_match.group(2),
        right_tokens[0],
    )


def _extract_team_tokens(value: str) -> list[str]:
    tokens = re.findall(r"[A-Za-z0-9\u4e00-\u9fff·（）()]+", value)
    ignored_tokens = {"数据", "直播", "动画", "视频", "情报", "析", "vs", "VS"}
    return [
        token
        for token in tokens
        if token not in ignored_tokens and not token.isdigit() and len(token) >= 2
    ]


def _dedupe_candidates(candidates: list[LeisuMatchCandidate]) -> list[LeisuMatchCandidate]:
    unique: dict[int, LeisuMatchCandidate] = {}
    for candidate in candidates:
        unique[candidate.detail_id] = candidate
    return list(unique.values())


def _match_candidate_detail_id(candidates: list[LeisuMatchCandidate], match: MatchResult) -> int | None:
    exact_score_matches = [
        candidate
        for candidate in candidates
        if _team_names_match(candidate.home_team_name, match.home_team_name)
        and _team_names_match(candidate.away_team_name, match.away_team_name)
        and candidate.home_score == match.home_score
        and candidate.away_score == match.away_score
    ]
    if len(exact_score_matches) == 1:
        return exact_score_matches[0].detail_id

    team_only_matches = [
        candidate
        for candidate in candidates
        if _team_names_match(candidate.home_team_name, match.home_team_name)
        and _team_names_match(candidate.away_team_name, match.away_team_name)
    ]
    if len(team_only_matches) == 1:
        return team_only_matches[0].detail_id

    return None


def _extract_technical_statistics_block(html: str) -> str | None:
    match = re.search(r'<div class="technical-statistics">(.*?)<div id="discussion"', html, re.S)
    if match is not None:
        return match.group(1)

    match = re.search(r'<div class="technical-statistics">(.*)', html, re.S)
    return match.group(1) if match is not None else None


def _parse_top_technical_values(block: str) -> dict[str, tuple[int, int]]:
    top_values: dict[str, tuple[int, int]] = {}
    left_block_match = re.search(r'<div class="ts-t-left">(.*?)</div>', block, re.S)
    right_block_match = re.search(r'<div class="ts-t-right">(.*?)</div>', block, re.S)
    if left_block_match is None or right_block_match is None:
        return top_values

    left_values = {
        css_class: _parse_int(value)
        for css_class, value in re.findall(
            r'<span class="lab ([^"]+)".*?<span class="text">(.*?)</span>',
            left_block_match.group(1),
            re.S,
        )
    }
    right_values = {
        css_class: _parse_int(value)
        for css_class, value in re.findall(
            r'<span class="lab ([^"]+)".*?<span class="text">(.*?)</span>',
            right_block_match.group(1),
            re.S,
        )
    }

    for slug, css_class in (
        ("corners", "corner"),
        ("yellow_cards", "card-yellow"),
        ("red_cards", "card-red"),
    ):
        left_value = left_values.get(css_class)
        right_value = right_values.get(css_class)
        if left_value is None or right_value is None:
            continue
        top_values[slug] = (left_value, right_value)

    return top_values


def _parse_bar_panel_values(block: str) -> dict[str, tuple[str, str]]:
    return {
        label: (home_value, away_value)
        for home_value, label, away_value in re.findall(
            r'<span class="left"><span class="num"><span class="tnum">(.*?)</span>.*?'
            r'<span class="barcenter">(.*?)</span>.*?'
            r'<span class="right"><span class="num"><span class="tnum">(.*?)</span>',
            block,
            re.S,
        )
    }


def _resolve_technical_stat_values(
    slug: str,
    *,
    top_values: Mapping[str, tuple[int, int]],
    panel_values: Mapping[str, tuple[str, str]],
) -> tuple[int, int, str | None] | None:
    if slug in {"corners", "yellow_cards", "red_cards"}:
        values = top_values.get(slug)
        if values is None:
            return None
        return (values[0], values[1], None)

    if slug == "possession":
        values = panel_values.get("控球率")
        if values is None:
            return None
        home = _try_parse_int(values[0])
        away = _try_parse_int(values[1])
        if home is None or away is None:
            return None
        return (home, away, "%")

    if slug == "attacks":
        values = panel_values.get("进攻")
        if values is None:
            return None
        home = _try_parse_int(values[0])
        away = _try_parse_int(values[1])
        if home is None or away is None:
            return None
        return (home, away, None)

    if slug == "dangerous_attacks":
        values = panel_values.get("危险进攻")
        if values is None:
            return None
        home = _try_parse_int(values[0])
        away = _try_parse_int(values[1])
        if home is None or away is None:
            return None
        return (home, away, None)

    if slug == "penalties":
        values = panel_values.get("点球")
        if values is None:
            return None
        home = _try_parse_int(values[0])
        away = _try_parse_int(values[1])
        if home is None or away is None:
            return None
        return (home, away, None)

    shots_values = panel_values.get("射门(射正)")
    if shots_values is None:
        return None

    home_shots = _parse_shots_value(shots_values[0])
    away_shots = _parse_shots_value(shots_values[1])
    if home_shots is None or away_shots is None:
        return None
    home_shots_total, home_shots_on_target = home_shots
    away_shots_total, away_shots_on_target = away_shots

    if slug == "shots_on_target":
        return (home_shots_on_target, away_shots_on_target, None)
    if slug == "shots_off_target":
        return (
            max(home_shots_total - home_shots_on_target, 0),
            max(away_shots_total - away_shots_on_target, 0),
            None,
        )

    return None


def _find_stat_values(
    technical_stats: list[MatchTechnicalStat],
    slug: str,
) -> tuple[int, int] | None:
    for stat in technical_stats:
        if stat.slug == slug:
            return (stat.home_value, stat.away_value)
    return None


def _looks_like_placeholder_technical_stats(technical_stats: list[MatchTechnicalStat]) -> bool:
    if not technical_stats:
        return False

    for stat in technical_stats:
        if stat.slug == "possession":
            if stat.home_value != 50 or stat.away_value != 50:
                return False
            continue
        if stat.home_value != 0 or stat.away_value != 0:
            return False

    return True


def _parse_int(value: str) -> int:
    match = re.search(r"-?\d+", value)
    if match is None:
        raise ValueError(f"Unable to parse integer value from {value!r}")
    return int(match.group(0))


def _try_parse_int(value: str) -> int | None:
    try:
        return _parse_int(value)
    except ValueError:
        return None


def _parse_shots_value(value: str) -> tuple[int, int] | None:
    match = re.fullmatch(r"\s*(\d+)\((\d+)\)\s*", value)
    if match is not None:
        return (int(match.group(1)), int(match.group(2)))
    parsed = _try_parse_int(value)
    if parsed is None:
        return None
    return (parsed, parsed)

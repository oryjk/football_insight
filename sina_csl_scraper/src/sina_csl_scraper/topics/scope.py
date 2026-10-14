"""Dynamic news coverage and stable club identities, independent of article ownership."""
from __future__ import annotations

from dataclasses import dataclass
from datetime import datetime
import copy
import re

from ..client import SinaCslClient

# Aliases identify clubs; rankings are always read at runtime, never encoded here.
ALIASES = {
    '成都蓉城': ('蓉城',), '天津津门虎': ('津门虎',), '大连英博': ('英博',),
    '北京国安': ('国安',), '上海海港': ('海港',), '上海申花': ('申花',),
    '山东泰山': ('泰山队', '山东队'), '武汉三镇': ('三镇',), '青岛海牛': ('海牛',),
    '青岛西海岸': ('西海岸',), '辽宁铁人': ('铁人',), '广东广州豹': ('广州豹',),
    '南通支云': ('支云',), '深圳新鹏城': ('深圳新鵬城', '新鹏城'),
    '云南玉昆': ('玉昆',), '重庆铜梁龙': ('铜梁龙',),
    '浙江队': ('浙江俱乐部', '浙江绿城'), '河南队': ('河南俱乐部',),
}
FOOTBALL = re.compile(r'足球|中超|中甲|中乙|球队|球员|主帅|主教练|联赛|足协杯|杯赛|赛季|控球|传球|射门|夺冠|争冠|保级|降级|进球|绝杀|比赛|训练|备战|赛程|赛后|转会|续约|引援|签约|外援|伤愈|停赛|禁赛|\d+[胜平负]|[0-9]\s*[:比]\s*[0-9]')


@dataclass(frozen=True)
class TeamTag:
    team_id: int
    name: str

    @property
    def aliases(self) -> tuple[str, ...]:
        return (self.name, *ALIASES.get(self.name, ()))

    def as_tag(self) -> dict:
        return {'team_id': self.team_id, 'name': self.name}


@dataclass(frozen=True)
class NewsScope:
    selected: tuple[TeamTag, ...]
    catalog: tuple[TeamTag, ...]


def tags_for_text(text: str, catalog) -> list[TeamTag]:
    """Tag substantive football sentences, not city nicknames or youth-name prefixes."""
    sentences = re.split(r'[。！？\n；]', text)
    result = []
    for team in catalog:
        for sentence in sentences:
            if team.name not in sentence and not FOOTBALL.search(sentence):
                continue
            if team.name == '成都蓉城' and ('希拉谷' in sentence or '中乙' in sentence):
                continue
            if any(alias in sentence for alias in team.aliases):
                result.append(team)
                break
    return result


def tag_existing_items(items: list[dict], catalog) -> list[dict]:
    updated = copy.deepcopy(items)
    for item in updated:
        item['team_tags'] = [tag.as_tag() for tag in topic_subject_tags(item, catalog)]
    return updated


def topic_subject_tags(item: dict, catalog) -> list[TeamTag]:
    """Prefer the article's subjects over incidental comparison clubs in its body."""
    headline = item['title'] + '\n' + item.get('summary', '')
    primary = tags_for_text(headline, catalog)
    sentences = re.split(r'[。！？\n；]', '\n'.join(item['paragraphs']))
    counts = {team.team_id: sum(bool(tags_for_text(sentence, [team])) for sentence in sentences) for team in catalog}
    maximum = max(counts.values(), default=0)
    ids = {team.team_id for team in primary}
    ids.update(team_id for team_id, count in counts.items() if count >= 2 and count * 2 >= maximum)
    if not ids:
        return tags_for_text('\n'.join(item['paragraphs']), catalog)
    ids.update(team.team_id for team in match_team_tags('\n'.join(item['paragraphs']), catalog, ids))
    return [team for team in catalog if team.team_id in ids]


def match_team_tags(text: str, catalog, subject_ids: set[int]) -> list[TeamTag]:
    """A direct match opponent is relevant even when mentioned only once."""
    ids = set()
    for clause in re.split(r'[，,。！？\n；]', text):
        if not re.search(r'对阵|迎战|战胜|击败|战平|不敌|负于|客战|绝杀|输给|\d+\s*[:比]\s*\d+', clause):
            continue
        teams = tags_for_text(clause, catalog)
        if len(teams) >= 2 and any(team.team_id in subject_ids for team in teams):
            ids.update(team.team_id for team in teams)
    return [team for team in catalog if team.team_id in ids]


def resolve_news_scope(config: dict, now: datetime, *, client_factory=SinaCslClient) -> NewsScope:
    selected, catalog = [], []
    selections = config.get('league_selections', [])
    if not selections:
        raise ValueError('news ranking selections are required')
    for selection in selections:
        league_id, top, bottom = selection['league_id'], selection['top'], selection['bottom']
        if league_id not in (213, 214) or any(type(n) is not int or n < 0 for n in (top, bottom)) or not top + bottom:
            raise ValueError('invalid news ranking selection')
        client = client_factory(league_id=league_id)
        league = client.fetch_league_info()
        if league.name != {213: '中超', 214: '中甲'}[league_id] or not now.year - 1 <= league.current_season <= now.year:
            raise ValueError('news ranking league or season mismatch')
        # These leagues play a home/away round robin. Use schedule metadata to
        # validate the complete participant count, including missing trailing ranks.
        if league.max_round <= 0 or league.max_round % 2:
            raise ValueError('news ranking has invalid league schedule metadata')
        expected_count = league.max_round // 2 + 1
        rows = sorted(client.fetch_standings(league.current_season), key=lambda row: row.order)
        if (len(rows) != expected_count or len(rows) < top + bottom or [r.order for r in rows] != list(range(1, len(rows) + 1))
                or len({r.team_id for r in rows}) != len(rows)
                or any(type(r.team_id) is not int or r.team_id <= 0 or not r.team_name.strip() for r in rows)):
            raise ValueError('news ranking is incomplete or inconsistent')
        teams = [TeamTag(row.team_id, row.team_name.replace('鵬', '鹏')) for row in rows]
        catalog.extend(teams)
        selected.extend(teams[:top])
        if bottom:
            selected.extend(teams[-bottom:])
    if len({team.team_id for team in catalog}) != len(catalog):
        raise ValueError('news ranking contains ambiguous club identities')
    return NewsScope(tuple(selected), tuple(catalog))

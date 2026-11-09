"""Dynamic news coverage and stable club identities, independent of article ownership."""
from __future__ import annotations

from dataclasses import dataclass
from datetime import datetime
import copy
import re

from ..client import SinaCslClient

# Aliases identify clubs; rankings are always read at runtime, never encoded here.
ALIASES = {
    '成都蓉城': ('蓉城',), '天津津门虎': ('津门虎', '天津队'), '大连英博': ('英博',),
    '北京国安': ('国安',), '上海海港': ('海港',), '上海申花': ('申花',),
    '山东泰山': ('泰山队', '山东队', '泰山'), '武汉三镇': ('三镇',), '青岛海牛': ('海牛',),
    '青岛西海岸': ('西海岸',), '辽宁铁人': ('铁人', '辽宁队'), '广东广州豹': ('广州豹',),
    '南通支云': ('支云', '南通队'), '深圳新鹏城': ('深圳新鵬城', '新鹏城', '深圳队'),
    '云南玉昆': ('玉昆', '云南队'), '重庆铜梁龙': ('铜梁龙', '重庆队'),
    '浙江队': ('浙江俱乐部', '浙江绿城'), '河南队': ('河南俱乐部', '河南建业'),
    '长春亚泰': ('亚泰',), '梅州客家': ('客家', '梅州队'), '陕西联合': ('陕西队',),
    '石家庄功夫': (), '大连鲲城': ('鲲城',), '延边龙鼎': ('龙鼎',), '苏州东吴': (),
    '广西恒宸': ('恒宸',), '深圳青年人': (), '无锡吴钩': ('吴钩',),
    '上海嘉定汇龙': ('嘉定汇龙', '汇龙'), '南京城市': ('南京队',), '黑龙江冰城': ('冰城',), '佛山南狮': ('南狮',),
}
FOOTBALL = re.compile(
    r'足球|中超|中甲|中乙|球队|球员|主帅|主教练|俱乐部|联赛|足协杯|杯赛|赛季|控球|传球|射门|夺冠|争冠|保级|降级|进球|绝杀|'
    r'比赛|训练|备战|赛程|赛后|转会|续约|引援|签约|外援|内援|国脚|伤愈|停赛|禁赛|主场|客场|对阵|迎战|战胜|击败|战平|不敌|'
    r'负于|输给|大胜|险胜|逆转|门将|守门员|后卫|中场|前锋|替补|首发|阵容|出场|登场|助攻|点球|任意球|角球|红牌|黄牌|金靴|'
    r'射手|火神杯|冲超|冲甲|准入|递补|离队|留队|留守|退役|加盟|租借|解约|合同|留洋|热身|集训|冬训|'
    r'\d+[胜平负]|[0-9]\s*[-:比]\s*[0-9]'
)
OVERSEAS_CONTEXT = re.compile(
    r'中国|国足|国脚|国青|国少|U\d+|王钰栋|武磊|吴少聪|李嗣镕|何小珂|杜月徵|魏祥鑫|张家鸣|刘邵子洋|'
    r'艾菲尔丁|拜合拉木|谢文能|胡荷韬|韦世豪|张玉宁|林良铭|蒋圣龙|朱辰杰|中日德兰|欧塞尔'
)


def is_chinese_overseas_football_news(text: str) -> bool:
    """Identify Chinese player overseas transfer/match news even when no domestic club is named."""
    return bool('留洋' in text and OVERSEAS_CONTEXT.search(text) and FOOTBALL.search(text))


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
        if client_factory is SinaCslClient:
            import requests
            session = requests.Session()
            session.trust_env = False
            client = client_factory(league_id=league_id, session=session)
        else:
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

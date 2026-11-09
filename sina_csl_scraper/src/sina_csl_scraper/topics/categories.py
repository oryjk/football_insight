"""每日电报的三个分类：国内、国际（五大联赛热门）、留洋。

分类由程序按文本里的证据判定，不由大模型说了算；大模型给的分类只在多个分类
都有证据时用来挑一个。热门名单和留洋球员名单都在新闻源配置里，可随时增删。
"""
from __future__ import annotations

from dataclasses import dataclass
import re

from .scope import FOOTBALL, tags_for_text

DOMESTIC, WORLD, ABROAD = 'domestic', 'world', 'abroad'
CATEGORY_LABELS = {DOMESTIC: '国内', WORLD: '国际', ABROAD: '留洋'}
# 同时有几类证据时的默认取舍：留洋新闻常顺带提国足或前东家，国内新闻常顺带提外国球员。
PRIORITY = (ABROAD, DOMESTIC, WORLD)
# 国内目前只收人事变动：主教练离任/上任，球员离队/加盟（含转会、租借、解约；传闻也算，但须标明）。
PERSONNEL = re.compile(r'离任|下课|辞职|卸任|解约|解雇|离队|告别|出走|加盟|签约|签下|引进|引入|转会|租借|回归|上任|出任|新帅|换帅|接任|执教|聘请|官宣.{0,8}(?:主帅|主教练|教练|球员|外援|内援)')


def is_personnel_move(text: str) -> bool:
    return bool(PERSONNEL.search(text))


NATIONAL_TEAM = re.compile(r'中超|中甲|中乙|足协杯|中国足协|国足|中国男足|中国队|中国国家队|国奥|国青|国少|中国U\d{2}')


@dataclass(frozen=True)
class Watchlist:
    """国际只收这些热门俱乐部、球员、教练的消息；名单来自配置 `world_watchlist`。"""
    clubs: tuple[tuple[str, ...], ...] = ()
    people: tuple[str, ...] = ()
    overseas_players: tuple[str, ...] = ()

    @classmethod
    def from_config(cls, config: dict) -> 'Watchlist':
        value = config.get('world_watchlist') or {}
        clubs = tuple(tuple(name for name in group if isinstance(name, str) and name.strip())
                      for group in value.get('clubs', []) if isinstance(group, list))
        people = tuple(name for name in value.get('people', []) if isinstance(name, str) and name.strip())
        overseas = tuple(name for name in config.get('overseas_players', []) if isinstance(name, str) and name.strip())
        return cls(tuple(group for group in clubs if group), people, overseas)

    def world_hits(self, text: str) -> list[str]:
        """命中的热门对象（俱乐部取主名）；只认带足球语境的句子，避免「巴黎」这类城市名误中。"""
        hits = []
        sentences = [s for s in re.split(r'[。！？\n；]', text) if FOOTBALL.search(s)]
        for group in self.clubs:
            if any(alias in sentence for sentence in sentences for alias in group):
                hits.append(group[0])
        hits.extend(name for name in self.people if any(name in sentence for sentence in sentences))
        return hits

    def is_abroad(self, text: str) -> bool:
        return ('留洋' in text and bool(FOOTBALL.search(text))) or any(name in text for name in self.overseas_players)


def candidate_categories(text: str, *, focus, watchlist: Watchlist) -> list[str]:
    """按优先级列出文本有证据支撑的分类；空列表 = 不在电报范围内。"""
    found = []
    if watchlist.is_abroad(text):
        found.append(ABROAD)
    if (focus is not None and tags_for_text(text, focus)) or NATIONAL_TEAM.search(text):
        found.append(DOMESTIC)
    if watchlist.world_hits(text):
        found.append(WORLD)
    return found


def choose_category(entry_text: str, reference_text: str, hint, *, focus, watchlist: Watchlist) -> str | None:
    """以引用原文的证据为准；条目自身也有证据的分类优先（标题常省略球队名，所以不强求）。
    大模型的分类只在证据允许的范围里采用；原文毫无证据则返回 None，条目丢弃。"""
    supported = candidate_categories(reference_text, focus=focus, watchlist=watchlist)
    if not supported:
        return None
    own = [category for category in candidate_categories(entry_text, focus=focus, watchlist=watchlist) if category in supported]
    allowed = own or supported
    return hint if hint in allowed else allowed[0]

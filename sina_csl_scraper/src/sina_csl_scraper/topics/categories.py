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
PERSONNEL = re.compile(
    r'离任|下课|辞职|卸任|解约|解雇|离队|告别|出走|不会留|不留队|离开.{0,6}(?:队|俱乐部)'
    r'|加盟|签约|签下|引进|引入|转会|租借|回归|上任|出任|新帅|换帅|接任|执教|聘请|选帅|选聘|续约'
    r'|官宣.{0,8}(?:主帅|主教练|教练|球员|外援|内援)'
)
HEADLINE_PERSONNEL = re.compile(
    r'离任|下课|辞职|卸任|解约|解雇|离队|告别|出走|不会留|不留队|离开.{0,6}(?:队|俱乐部)'
    r'|加盟|签约|签下|引进|引入|转会|租借|回归|上任|出任|新帅|换帅|接任|执教|聘请|选帅|选聘|续约'
    r'|(?:担任|成为|敲定).{0,12}(?:主帅|主教练|助教|助理教练|教练)'
    r'|(?:有意|商谈|接近|敲定|接触).{0,12}(?:主帅|主教练|前锋|中场|后卫|门将|外援|内援|球员)'
    r'|官宣.{0,8}(?:主帅|主教练|教练|球员|外援|内援)'
    r'|决定未来|与另一支.{0,6}球队谈|如果回|终有一别.{0,12}送别'
)
SYNTHETIC_TITLE = re.compile(r'^新闻\d+$')
DIGEST_TITLE = re.compile(r'^(?:【?早报】?|早报[:：])')
RETROSPECTIVE_TITLE = re.compile(
    r'世体关注|韩媒：.*陷入混乱|曾喊话.*丢冠一天后|简单告别.*的背后|到底是谁[“"]?逼[”"]?走'
    r'|都须找教练|是大考|前中超名帅回归|是放弃稳妥|是最优解'
)
LEAVE_RE = re.compile(r'离任|下课|辞职|卸任|解约|解雇|离队|告别|出走|不再担任|不会留|不留队')
JOIN_RE = re.compile(r'加盟|签约|签下|引进|引入|转会|租借|上任|出任|接任|执教|聘请|聘任|敲定|任命')
RUMOR_RE = re.compile(
    r'据报道|传闻|据.*(?:透露|消息|报道|称)|有意|接近|有望|或将|或|基本敲定|预计.*官宣'
    r'|商谈|接触|询价|询问|考虑|关注|候选|头号人选|达成.*(?:口头)?协议|进度|交易达成'
    r'|即将|将任|将出任|将执教|将解雇|计划解雇|帅位危机|商讨.*去留'
)
HEAD_COACH_RE = re.compile(
    r'主帅|主教练|新帅|执教|阿洛伊西|拉莫斯|斯托伊科维奇|常卫魏|里斯蒂奇|卡里克|波切蒂诺|冯峰|肇俊哲|刘俊威'
)
EXEC_RE = re.compile(r'董事长|总经理|王磊|吴博')
SCORE_RE = re.compile(r'(\d+)\s*(?:[-:：]|比)\s*(\d+)')
EVENT_TEAM_ALIASES: tuple[tuple[str, tuple[str, ...]], ...] = (
    ('成都蓉城', ('成都蓉城', '蓉城')),
    ('山东泰山', ('山东泰山', '泰山')),
    ('上海申花', ('上海申花', '申花')),
    ('上海海港', ('上海海港', '海港')),
    ('浙江', ('浙江队', '浙江')),
    ('河南', ('河南队', '河南')),
    ('青岛海牛', ('青岛海牛', '海牛')),
    ('深圳新鹏城', ('深圳新鹏城', '新鹏城')),
    ('云南玉昆', ('云南玉昆', '玉昆')),
    ('大连英博', ('大连英博',)),
    ('武汉三镇', ('武汉三镇',)),
    ('南通支云', ('南通支云',)),
    ('广州豹', ('广州豹',)),
    ('厦门飞鹭', ('厦门飞鹭',)),
    ('中国女足', ('中国女足', '女足')),
    ('中国男足', ('中国男足', '国足', '中国队')),
    ('曼联', ('曼联', '曼彻斯特联')),
    ('曼城', ('曼城', '曼彻斯特城')),
    ('利物浦', ('利物浦',)),
    ('阿森纳', ('阿森纳', '枪手')),
    ('切尔西', ('切尔西',)),
    ('热刺', ('热刺', '托特纳姆')),
    ('皇马', ('皇马', '皇家马德里')),
    ('巴萨', ('巴萨', '巴塞罗那')),
    ('马竞', ('马竞', '马德里竞技')),
    ('拜仁', ('拜仁', '拜仁慕尼黑')),
    ('多特蒙德', ('多特蒙德', '多特')),
    ('勒沃库森', ('勒沃库森', '药厂')),
    ('国米', ('国际米兰', '国米')),
    ('AC米兰', ('AC米兰', '米兰')),
    ('尤文', ('尤文图斯', '尤文')),
    ('那不勒斯', ('那不勒斯',)),
    ('巴黎圣日耳曼', ('巴黎圣日耳曼', '大巴黎', '巴黎')),
    ('维拉', ('阿斯顿维拉', '维拉')),
)


def is_personnel_move(text: str) -> bool:
    return bool(PERSONNEL.search(text))


def is_personnel_source(title: str, body: str = '') -> bool:
    """国内人事快讯必须由标题直接报道人事变动的原文支撑，拒绝从综述、评论、外媒反响里抠旧背景。"""
    cleaned = (title or '').strip()
    if SYNTHETIC_TITLE.fullmatch(cleaned):
        return is_personnel_move((body or '')[:160])
    if not cleaned or DIGEST_TITLE.search(cleaned) or RETROSPECTIVE_TITLE.search(cleaned):
        return False
    return bool(HEADLINE_PERSONNEL.search(cleaned))


def event_keys(title: str) -> tuple[set[str], str]:
    """从快讯标题提取事件指纹与阶段（RUMOR / OFFICIAL），用于同日及跨日去重。"""
    cleaned = (title or '').strip()
    stage = 'RUMOR' if RUMOR_RE.search(cleaned) else 'OFFICIAL'
    teams = sorted(canonical for canonical, aliases in EVENT_TEAM_ALIASES if any(alias in cleaned for alias in aliases))
    keys: set[str] = set()
    score = SCORE_RE.search(cleaned)
    if score and teams:
        s1, s2 = sorted((score.group(1), score.group(2)))
        keys.add(f'score:{"-".join(teams)}:{s1}-{s2}')
    is_leave = bool(LEAVE_RE.search(cleaned))
    is_join = bool(JOIN_RE.search(cleaned))
    if teams and (is_leave or is_join):
        if HEAD_COACH_RE.search(cleaned) and '助教' not in cleaned:
            for team in teams:
                if is_leave and any(w in cleaned for w in ('下课', '解雇', '离任', '离队', '告别', '不再担任', '解约')):
                    keys.add(f'coach_leave:{team}')
                if is_join and any(w in cleaned for w in ('出任', '上任', '执教', '新帅', '任命', '敲定')):
                    keys.add(f'coach_join:{team}')
        if EXEC_RE.search(cleaned) and is_leave:
            for team in teams:
                keys.add(f'exec_leave:{team}')
    if not keys:
        keys.add(f'exact:{cleaned}')
    return keys, stage


class EventDeduplicator:
    """记录已发布的事件指纹：传闻升级为官宣允许再发一次，但同一传闻或同一官宣不重复发。"""

    def __init__(self, existing_titles: list[str] | set[str] = ()):
        self.seen_titles: set[str] = set()
        self.seen_official: set[str] = set()
        self.seen_rumor: set[str] = set()
        for title in existing_titles:
            self.record(title)

    def is_duplicate(self, title: str) -> bool:
        cleaned = (title or '').strip()
        if not cleaned or cleaned in self.seen_titles:
            return True
        keys, stage = event_keys(cleaned)
        pool = self.seen_official if stage == 'OFFICIAL' else (self.seen_official | self.seen_rumor)
        return bool(keys) and keys.issubset(pool)

    def record(self, title: str) -> None:
        cleaned = (title or '').strip()
        if not cleaned:
            return
        self.seen_titles.add(cleaned)
        keys, stage = event_keys(cleaned)
        if stage == 'OFFICIAL':
            self.seen_official.update(keys)
            self.seen_rumor.update(keys)
        else:
            self.seen_rumor.update(keys)

    def accept(self, title: str) -> bool:
        if self.is_duplicate(title):
            return False
        self.record(title)
        return True


NATIONAL_TEAM = re.compile(r'中超|中甲|中乙|足协杯|中国足协|国足|中国男足|中国队|中国国家队|国奥|国青|国少|中国U\d{2}')


# 同时也是普通词语的别名：只在带足球语境的句子里才算命中。
AMBIGUOUS_ALIASES = {'巴黎', '大巴黎', '枪手', '药厂', '多特', '国际米兰'}


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
        """命中的热门对象（俱乐部取主名）。「曼联」「卡里克」这类名字本身就是足球的，直接算；
        「巴黎」「枪手」这类也是普通词的别名，要求所在句子有足球语境。"""
        sentences = re.split(r'[。！？\n；]', text)
        football = [s for s in sentences if FOOTBALL.search(s)]
        def found(alias: str) -> bool:
            pool = football if alias in AMBIGUOUS_ALIASES else sentences
            return any(alias in sentence for sentence in pool)
        hits = [group[0] for group in self.clubs if any(found(alias) for alias in group)]
        hits.extend(name for name in self.people if found(name))
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
    if own:
        return hint if hint in own else own[0]
    # 条目自己没有证据（标题常省略球队名）时才借原文的：国际不借（综述类原文常顺带提热门球队），
    # 而且只在原文的证据只指向一个分类、或大模型给的分类在其中时才借，否则宁可丢掉，也不硬归类。
    borrowable = [category for category in supported if category != WORLD]
    if hint in borrowable:
        return hint
    return borrowable[0] if len(supported) == 1 and borrowable else None

"""Reviewed head-coach portraits, keyed by Sina's English manager name (`TeamCoach.coach_key`).

Sina's team API has coach names but no photo. Each entry was checked by hand against a named
source: the 2026 CSL standard-photo galleries on Sohu (caption = coach name, club kit) or the
club's own announcement poster (name printed on it). Coaches without such a source keep no
avatar; the mini program then shows the name initial. Never guess from face or photo order.
Records and the coaches still missing a photo: docs/csl-player-portraits-2026.md.

Posters are cropped to the head; `crop` is (left, top, right, bottom) on the original image.
Versioned file names keep old objects and invalidate client image caches.
"""
from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True)
class CoachPortrait:
    source_url: str
    filename: str
    crop: tuple[int, int, int, int] | None = None


def _gallery(host: str, day: str, image: str) -> str:
    return f"https://{host}.itc.cn/q_85,c_zoom,w_600/images03/{day}/{image}.jpeg"


COACH_AVATAR_OVERRIDES: dict[str, CoachPortrait] = {
    # 成都蓉城 — club poster "欢迎 约翰·阿洛伊西 担任成都蓉城队主教练" (sohu.com/a/973120238, 2026-01-06)
    "John Aloisi": CoachPortrait(
        "https://q1.itc.cn/images03/20260106/da597f39f5b04273894c992b8aaf2aee.jpeg",
        "coach-john-aloisi-20260106.jpg",
        (120, 330, 840, 1050),
    ),
    # 辽宁铁人 — Chengdu farewell poster "功勋主教练 徐正源" (same article); kit is Chengdu's
    "Seo Jung-won": CoachPortrait(
        "https://q4.itc.cn/images03/20260106/2cf1594b73404c9a92594caf24fcbb58.jpeg",
        "coach-seo-jung-won-20260106.jpg",
        (300, 410, 700, 810),
    ),
    # 河南队 — gallery caption: 丹尼尔-拉莫斯 (sohu.com/picture/990315580)
    "Daniel Ramos": CoachPortrait(
        _gallery("q0", "20260226", "389638374e6348aab08e78877af675cb"),
        "coach-daniel-ramos-20260226.jpg",
    ),
    # 青岛西海岸 — gallery caption: 郑智 (sohu.com/picture/992394253)
    "Zheng Zhi": CoachPortrait(
        _gallery("q6", "20260304", "8e0a04804ecf40f3b91368bb98a66217"),
        "coach-zheng-zhi-20260304.jpg",
    ),
    # 青岛海牛 — gallery caption: 米兰 (sohu.com/picture/992439853), Hainiu kit
    "Milan Ristić": CoachPortrait(
        _gallery("q6", "20260304", "aa040eed11ed40a6b58bab6890b00107"),
        "coach-milan-ristic-20260304.jpg",
    ),
    # 上海海港 — gallery caption: 穆斯卡特 (sohu.com/picture/992953413)
    "Kevin Muscat": CoachPortrait(
        _gallery("q4", "20260305", "a3334785591d469c896abedba2152c1c"),
        "coach-kevin-muscat-20260305.jpg",
    ),
    # 浙江队 — gallery caption: 罗斯-阿洛伊西 (sohu.com/picture/993005456)
    "Ross Aloisi": CoachPortrait(
        _gallery("q3", "20260305", "c6cd8b765c6545949a9a4588c0591d9f"),
        "coach-ross-aloisi-20260305.jpg",
    ),
    # 重庆铜梁龙 — full-length gallery photo, caption: 刘建业 (sohu.com/picture/996153516)
    "Liu Jianye": CoachPortrait(
        "https://q5.itc.cn/images03/20260313/f705f1cd08494531a5b9344580ff522b.jpeg",
        "coach-liu-jianye-20260313.jpg",
        (1450, 250, 2550, 1350),
    ),
}

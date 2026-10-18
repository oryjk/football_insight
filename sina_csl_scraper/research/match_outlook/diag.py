import json, sys, math, collections
import numpy as np
from scipy.stats import spearmanr
src = open("backtest.py").read().split("# ---------------------------------------------------------------- 调参")[0]
exec(src)  # 复用数据加载、指标、模型函数；不重新调参
P = json.load(open("params.json"))
K, H, s, c, gexp, reg, ns = P["elo_params"]

# --- Elo（同 run_elo），额外记录每个赛季初评分
def elo_with_snaps():
    ratings, preds, season, snaps = {}, {}, None, {}
    for i, m in enumerate(matches):
        if m["season"] != season:
            season = m["season"]
            for t in ratings: ratings[t] = reg * ratings[t] + (1 - reg) * 1500.0
            snaps[season] = dict(ratings)
        rh = ratings.setdefault(m["h"], ns); ra = ratings.setdefault(m["a"], ns)
        snaps[season].setdefault(m["h"], ns); snaps[season].setdefault(m["a"], ns)
        x = (rh + (0 if m["neutral"] else H) - ra) / s
        ph = 1/(1+math.exp(-(x-c))); pa = 1/(1+math.exp(x+c)); pd = max(1-ph-pa,1e-6); z = ph+pd+pa
        p = (ph/z, pd/z, pa/z); preds[i] = p
        e = p[0]+0.5*p[1]; sc = (1.0,0.5,0.0)[m["o"]]
        d = K*(1+abs(m["hg"]-m["ag"]))**gexp*(sc-e)
        ratings[m["h"]] = rh+d; ratings[m["a"]] = ra-d
    return preds, snaps
elo, snaps = elo_with_snaps()
base = run_baseline()
N26 = sum(1 for m in matches if m["season"] == 2026)

print(f"2026 已赛 {N26} 场。为公平比较，各赛季均只取『该赛季前 {N26} 场』。\n")
print(f"{'赛季':<6}{'球队实力离散(赛季初评分SD)':>22}{'球队场均积分SD':>16}{'赛季初评分与场均积分的Spearman':>26}{'主胜':>7}{'平':>6}{'客胜':>6}{'Elo-基线RPS':>14}")
rows = {}
for sea in range(2016, 2027):
    idx = [i for i, m in enumerate(matches) if m["season"] == sea][:N26]
    pts = collections.defaultdict(lambda: [0, 0])
    for i in idx:
        m = matches[i]
        ph, pa = (3, 0) if m["o"] == 0 else ((1, 1) if m["o"] == 1 else (0, 3))
        pts[m["h"]][0] += ph; pts[m["h"]][1] += 1; pts[m["a"]][0] += pa; pts[m["a"]][1] += 1
    teams = list(pts)
    ppg = [pts[t][0]/pts[t][1] for t in teams]
    r0 = [snaps[sea].get(t, ns) for t in teams]
    rho = spearmanr(r0, ppg).statistic
    o = [matches[i]["o"] for i in idx]
    gap = np.mean([rps(elo[i], matches[i]["o"]) - rps(base[i], matches[i]["o"]) for i in idx])
    print(f"{sea:<6}{np.std(r0):>22.1f}{np.std(ppg):>16.3f}{rho:>26.2f}{o.count(0)/len(o):>7.0%}{o.count(1)/len(o):>6.0%}{o.count(2)/len(o):>6.0%}{gap:>+14.4f}")

# --- 赛季前段 vs 后段（按赛季内顺序）
print("\nElo 比基线好多少（RPS 差，越负越好）：赛季前 1/3 vs 后 2/3")
for sea in (2023, 2024, 2025, 2026):
    idx = [i for i, m in enumerate(matches) if m["season"] == sea]
    cut = len(idx)//3
    f = lambda part: np.mean([rps(elo[i], matches[i]["o"]) - rps(base[i], matches[i]["o"]) for i in part])
    print(f"  {sea}: 前段 {f(idx[:cut]):+.4f}  后段 {f(idx[cut:]):+.4f}  (n={cut}/{len(idx)-cut})")

# --- 蓉城 vs 其他
names = {}
for m in json.load(open(f"{DATA}/2026.json")):
    names[m["home_team_name"]] = m["home_team_id"]; names[m["away_team_name"]] = m["away_team_id"]
cd = names["成都蓉城"]
print(f"\n成都蓉城 team_id={cd}")
for sea in (2025, 2026):
    idx = [i for i, m in enumerate(matches) if m["season"] == sea]
    cdi = [i for i in idx if cd in (matches[i]["h"], matches[i]["a"])]
    oth = [i for i in idx if i not in set(cdi)]
    f = lambda part: (np.mean([rps(elo[i], matches[i]["o"]) for i in part]), np.mean([rps(base[i], matches[i]["o"]) for i in part]))
    a, b = f(cdi), f(oth)
    print(f"  {sea}: 蓉城参与 {len(cdi)} 场 Elo RPS {a[0]:.4f}/基线 {a[1]:.4f}；其余 {len(oth)} 场 Elo {b[0]:.4f}/基线 {b[1]:.4f}")

# --- 2026 新球队 / 评分排序
prev = {m["h"] for m in matches if m["season"] == 2025} | {m["a"] for m in matches if m["season"] == 2025}
cur = {m["h"] for m in matches if m["season"] == 2026} | {m["a"] for m in matches if m["season"] == 2026}
inv = {v: k for k, v in names.items()}
print("\n2026 新进入的球队:", [inv.get(t, t) for t in cur - prev] or "无", "| 离开:", len(prev - cur))

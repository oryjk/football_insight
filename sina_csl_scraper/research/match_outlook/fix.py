import json, sys, math, itertools, collections
import numpy as np
src = open("backtest.py").read().split("# ---------------------------------------------------------------- 调参")[0]
exec(src)
P = json.load(open("params.json"))
SEAS = list(range(2016, 2027))
by_season = collections.defaultdict(list)
for i, m in enumerate(matches): by_season[m["season"]].append(i)

TAU_GRID = np.array([0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0, 1.1, 1.2])

def probs(x, c):
    ph = 1/(1+math.exp(-(x-c))); pa = 1/(1+math.exp(x+c)); pd = max(1-ph-pa, 1e-6); z = ph+pd+pa
    return (ph/z, pd/z, pa/z)

def run_elo2(K, H, s, c, gexp, reg, ns, adaptive=0, record_tau=None):
    ratings, preds, season, hist = {}, {}, None, []
    for i, m in enumerate(matches):
        if m["season"] != season:
            season = m["season"]
            for t in ratings: ratings[t] = reg*ratings[t] + (1-reg)*1500.0
        rh = ratings.setdefault(m["h"], ns); ra = ratings.setdefault(m["a"], ns)
        x = (rh + (0.0 if m["neutral"] else H) - ra) / s
        tau = 1.0
        if adaptive and len(hist) >= 40:
            win = hist[-adaptive:]
            xs = np.array([w[0] for w in win]); os_ = [w[1] for w in win]
            best, bt = 1e18, 1.0
            for t in TAU_GRID:
                ll = 0.0
                for xv, ov in zip(xs, os_):
                    ll += -math.log(max(probs(xv*t, c)[ov], 1e-12))
                if ll < best: best, bt = ll, t
            tau = bt
        if record_tau is not None: record_tau.append((m["season"], tau))
        p = probs(x*tau, c); preds[i] = p
        e = p[0]+0.5*p[1]; sc = (1.0,0.5,0.0)[m["o"]]
        d = K*(1+abs(m["hg"]-m["ag"]))**gexp*(sc-e)
        ratings[m["h"]] = rh+d; ratings[m["a"]] = ra-d
        hist.append((x, m["o"]))
    return preds

print("网格：每组参数跑一次，记录各赛季平均 RPS …", flush=True)
grid = list(itertools.product((10,20,30,45),(30,60,90),(150,200,250),(0.3,0.5,0.7),(0,0.5,1.0),(0.4,0.6,0.8,1.0),(1400,1450,1500)))
table = {}
for g in grid:
    p = run_elo2(*g)
    table[g] = {s_: float(np.mean([rps(p[i], matches[i]["o"]) for i in by_season[s_]])) for s_ in SEAS}

def chosen(Y):
    # 只用 Y 之前的三个赛季选参数（不含中立赛季之外的任何未来信息）
    win = [s_ for s_ in (Y-3, Y-2, Y-1) if s_ >= 2016]
    return min(grid, key=lambda g: np.mean([table[g][s_] for s_ in win]))

TEST = list(range(2019, 2027))
v0 = run_elo2(*P["elo_params"])              # 原方案：固定参数（2016-2018 调出）
base = run_baseline()
v1, v2, taus = {}, {}, []
print("\n每个测试赛季滚动选出的参数 (K,H,s,c,gexp,reg,newstart)：")
for Y in TEST:
    g = chosen(Y); print(" ", Y, g)
    p1 = run_elo2(*g); rec = []
    p2 = run_elo2(*g, adaptive=150, record_tau=rec)
    for i in by_season[Y]: v1[i] = p1[i]; v2[i] = p2[i]
    k0 = sum(len(by_season[s_]) for s_ in range(2014, Y))
    taus.append((Y, float(np.mean([t for sv, t in rec if sv == Y]))))
poi = run_poisson(*P["poisson_params"], seasons=set(TEST))

models = {"基线": base, "Elo原方案": v0, "V1滚动调参": v1, "V2滚动+自适应收缩": v2, "泊松": poi}
norm = [i for Y in TEST for i in by_season[Y] if not matches[i]["neutral"]]
print("\n剔除 2020-21，样本外", len(norm), "场")
print(f"{'模型':<20}{'RPS':>8}{'对数损失':>10}{'命中率':>8}")
for n_, p in models.items():
    r, l, a = score(p, norm); print(f"{n_:<20}{r.mean():>8.4f}{l.mean():>10.4f}{a.mean():>8.1%}")
print("\n按赛季 RPS")
print(f"{'赛季':<6}" + "".join(f"{n_:>20}" for n_ in models))
for Y in TEST:
    print(f"{Y:<6}" + "".join(f"{score(p, by_season[Y])[0].mean():>20.4f}" for p in models.values()))
print("\nV2 自适应收缩系数 τ 的赛季均值（1=不收缩，越小越向平局/均衡收缩）：", {y: round(t, 2) for y, t in taus})

rng = np.random.default_rng(11)
def boot(a, b, idx):
    d = score(a, idx)[0] - score(b, idx)[0]
    bs = [d[rng.integers(0, len(d), len(d))].mean() for _ in range(3000)]
    return d.mean(), np.percentile(bs, 2.5), np.percentile(bs, 97.5)
print("\n配对自助法 RPS 差（负=前者更好）")
for (na, a), (nb, b) in ((("V1", v1), ("原方案", v0)), (("V2", v2), ("V1", v1)), (("V2", v2), ("原方案", v0)), (("V2", v2), ("基线", base))):
    m_, lo, hi = boot(a, b, norm)
    print(f"  {na} - {nb}: {m_:+.4f}  CI [{lo:+.4f}, {hi:+.4f}]  {'显著' if lo>0 or hi<0 else '不显著'}")
i26 = by_season[2026]
for (na, a), (nb, b) in ((("V2", v2), ("原方案", v0)), (("V2", v2), ("基线", base))):
    m_, lo, hi = boot(a, b, i26)
    print(f"  仅2026  {na} - {nb}: {m_:+.4f}  CI [{lo:+.4f}, {hi:+.4f}]")

print("\n校准（V2，剔除2020-21）")
edges = [0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.8, 1.01]
pairs = [(v2[i][k], 1.0 if matches[i]["o"] == k else 0.0) for i in norm for k in range(3)]
for lo_, hi_ in zip(edges, edges[1:]):
    sel = [(p, y) for p, y in pairs if lo_ <= p < hi_]
    if sel: print(f"  [{lo_:.1f},{min(hi_,1):.1f})  n={len(sel):>5}  预测 {np.mean([p for p,_ in sel]):.3f}  实际 {np.mean([y for _,y in sel]):.3f}")

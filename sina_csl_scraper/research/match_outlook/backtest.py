"""中超胜平负概率：基线 / Elo / 泊松(简化 Dixon-Coles) 的滚动前向回测。

只用比分，不用任何博彩赔率。调参只用 2016-2018，2019 起为未参与调参的样本外测试。
"""
import datetime as dt
import itertools
import json
import math
import sys

import numpy as np
from scipy.optimize import minimize

DATA = sys.argv[1]
NEUTRAL_SEASONS = {2020, 2021}  # 疫情赛会制，近乎中立场地
TUNE_SEASONS = (2016, 2017, 2018)
TEST_FROM = 2019

# ---------------------------------------------------------------- 数据
matches = []
for season in range(2014, 2027):
    for r in json.load(open(f"{DATA}/{season}.json")):
        if r["home_score"] == "" or r["away_score"] == "":
            continue
        matches.append(
            dict(
                season=season,
                day=dt.date.fromisoformat(r["date"]).toordinal(),
                time=r["time"],
                h=r["home_team_id"],
                a=r["away_team_id"],
                hg=int(r["home_score"]),
                ag=int(r["away_score"]),
                neutral=season in NEUTRAL_SEASONS,
            )
        )
matches.sort(key=lambda m: (m["day"], m["time"]))
for m in matches:
    m["o"] = 0 if m["hg"] > m["ag"] else (1 if m["hg"] == m["ag"] else 2)
print(f"比赛 {len(matches)} 场，赛季 2014-2026")

# ---------------------------------------------------------------- 指标
def rps(p, o):
    c1, c2 = p[0], p[0] + p[1]
    o1, o2 = (1.0 if o == 0 else 0.0), (1.0 if o <= 1 else 0.0)
    return ((c1 - o1) ** 2 + (c2 - o2) ** 2) / 2


def logloss(p, o):
    return -math.log(max(p[o], 1e-12))


def score(preds, idx):
    return (
        np.array([rps(preds[i], matches[i]["o"]) for i in idx]),
        np.array([logloss(preds[i], matches[i]["o"]) for i in idx]),
        np.array([int(np.argmax(preds[i]) == matches[i]["o"]) for i in idx]),
    )


# ---------------------------------------------------------------- M0 基线
def run_baseline(prior=30.0):
    preds, counts = {}, np.array([0.46, 0.26, 0.28]) * prior
    for i, m in enumerate(matches):
        preds[i] = counts / counts.sum()
        if not m["neutral"]:
            counts = counts.copy()
            counts[m["o"]] += 1
    return preds


# ---------------------------------------------------------------- M1 Elo
def run_elo(K, H, s, c, gexp, reg, newstart, only=None):
    ratings, preds, season = {}, {}, None
    for i, m in enumerate(matches):
        if m["season"] != season:
            season = m["season"]
            for t in ratings:
                ratings[t] = reg * ratings[t] + (1 - reg) * 1500.0
        rh = ratings.setdefault(m["h"], newstart)
        ra = ratings.setdefault(m["a"], newstart)
        x = (rh + (0.0 if m["neutral"] else H) - ra) / s
        ph = 1 / (1 + math.exp(-(x - c)))
        pa = 1 / (1 + math.exp(x + c))
        pd = max(1 - ph - pa, 1e-6)
        z = ph + pd + pa
        p = (ph / z, pd / z, pa / z)
        preds[i] = p
        e = p[0] + 0.5 * p[1]
        sc = (1.0, 0.5, 0.0)[m["o"]]
        delta = K * (1 + abs(m["hg"] - m["ag"])) ** gexp * (sc - e)
        ratings[m["h"]] = rh + delta
        ratings[m["a"]] = ra - delta
    return preds


# ---------------------------------------------------------------- M2 泊松
def fit_poisson(train, halflife, ridge, ref_day):
    ids = sorted({m["h"] for m in train} | {m["a"] for m in train})
    pos = {t: k for k, t in enumerate(ids)}
    n = len(ids)
    hi = np.array([pos[m["h"]] for m in train])
    ai = np.array([pos[m["a"]] for m in train])
    hg = np.array([m["hg"] for m in train], float)
    ag = np.array([m["ag"] for m in train], float)
    home = np.array([0.0 if m["neutral"] else 1.0 for m in train])
    w = 0.5 ** ((ref_day - np.array([m["day"] for m in train], float)) / halflife)

    def nll(theta):
        mu, h = theta[0], theta[1]
        att, dfn = theta[2 : 2 + n], theta[2 + n :]
        lh = np.exp(mu + h * home + att[hi] - dfn[ai])
        la = np.exp(mu + att[ai] - dfn[hi])
        rh, ra = w * (lh - hg), w * (la - ag)
        val = np.sum(w * (lh - hg * np.log(lh) + la - ag * np.log(la)))
        val += ridge * (att @ att + dfn @ dfn)
        g = np.zeros_like(theta)
        g[0] = rh.sum() + ra.sum()
        g[1] = (rh * home).sum()
        g[2 : 2 + n] = np.bincount(hi, rh, n) + np.bincount(ai, ra, n) + 2 * ridge * att
        g[2 + n :] = -np.bincount(ai, rh, n) - np.bincount(hi, ra, n) + 2 * ridge * dfn
        return val, g

    x0 = np.zeros(2 + 2 * n)
    x0[0] = 0.2
    res = minimize(nll, x0, jac=True, method="L-BFGS-B")
    th = res.x
    return pos, th[0], th[1], th[2 : 2 + n], th[2 + n :]


def score_matrix(lh, la, rho, g=10):
    ph = np.exp(-lh) * np.array([lh**k / math.factorial(k) for k in range(g + 1)])
    pa = np.exp(-la) * np.array([la**k / math.factorial(k) for k in range(g + 1)])
    mat = np.outer(ph, pa)
    mat[0, 0] *= 1 - lh * la * rho
    mat[0, 1] *= 1 + lh * rho
    mat[1, 0] *= 1 + la * rho
    mat[1, 1] *= 1 - rho
    mat /= mat.sum()
    return (np.tril(mat, -1).sum(), np.trace(mat), np.triu(mat, 1).sum())


def run_poisson(halflife, ridge, rho, seasons, window=1095):
    preds, i, n = {}, 0, len(matches)
    while i < n:
        j = i
        while j < n and matches[j]["day"] == matches[i]["day"]:
            j += 1
        day = matches[i]["day"]
        if matches[i]["season"] in seasons:
            train = [m for m in matches[:i] if day - m["day"] <= window]
            pos, mu, h, att, dfn = fit_poisson(train, halflife, ridge, day)
            for k in range(i, j):
                m = matches[k]
                ph = pos.get(m["h"]); pa_ = pos.get(m["a"])
                a_h = att[ph] if ph is not None else 0.0
                d_h = dfn[ph] if ph is not None else 0.0
                a_a = att[pa_] if pa_ is not None else 0.0
                d_a = dfn[pa_] if pa_ is not None else 0.0
                lh = math.exp(mu + (0.0 if m["neutral"] else h) + a_h - d_a)
                la = math.exp(mu + a_a - d_h)
                preds[k] = score_matrix(lh, la, rho)
        i = j
    return preds


# ---------------------------------------------------------------- 调参（仅 2016-2018）
tune_idx = [i for i, m in enumerate(matches) if m["season"] in TUNE_SEASONS]
test_idx = [i for i, m in enumerate(matches) if m["season"] >= TEST_FROM]

print("\n[调参] Elo 网格搜索，目标：2016-2018 平均 RPS")
best_elo, best_val = None, 9
grid = itertools.product(
    (10, 20, 30, 45), (30, 60, 90), (150, 200, 250), (0.3, 0.5, 0.7),
    (0, 0.5, 1.0), (0.6, 0.8, 1.0), (1400, 1450, 1500),
)
for K, H, s, c, gexp, reg, ns in grid:
    p = run_elo(K, H, s, c, gexp, reg, ns)
    v = np.mean([rps(p[i], matches[i]["o"]) for i in tune_idx])
    if v < best_val:
        best_val, best_elo = v, (K, H, s, c, gexp, reg, ns)
print("  最优 Elo 参数 (K,H,s,c,gexp,reg,newstart) =", best_elo, f"RPS={best_val:.4f}")

print("[调参] 泊松网格搜索")
best_poi, best_pv = None, 9
for hl, rd, rho in itertools.product((180, 365, 730), (2, 5, 10, 20), (0.0, -0.1)):
    p = run_poisson(hl, rd, rho, set(TUNE_SEASONS))
    v = np.mean([rps(p[i], matches[i]["o"]) for i in tune_idx])
    if v < best_pv:
        best_pv, best_poi = v, (hl, rd, rho)
print("  最优泊松参数 (半衰期天,ridge,rho) =", best_poi, f"RPS={best_pv:.4f}")

# ---------------------------------------------------------------- 样本外测试
print("\n[测试] 2019-2026 滚动前向（未参与调参）")
base = run_baseline()
elo = run_elo(*best_elo)
poi = run_poisson(*best_poi, seasons=set(range(TEST_FROM, 2027)))
ens = {i: tuple((np.array(elo[i]) + np.array(poi[i])) / 2) for i in test_idx}
models = {"基线(主平客频率)": base, "Elo": elo, "泊松(简化DC)": poi, "Elo+泊松平均": ens}

norm = [i for i in test_idx if not matches[i]["neutral"]]
for label, idx in (("全部测试样本", test_idx), ("剔除2020-21", norm)):
    print(f"\n{label}（{len(idx)} 场）")
    print(f"{'模型':<16}{'RPS':>8}{'对数损失':>10}{'命中率':>8}")
    for name, p in models.items():
        r, l, a = score(p, idx)
        print(f"{name:<16}{r.mean():>8.4f}{l.mean():>10.4f}{a.mean():>8.1%}")

print("\n按赛季 RPS")
seasons = sorted({matches[i]["season"] for i in test_idx})
print(f"{'赛季':<6}" + "".join(f"{n:>16}" for n in models))
for s_ in seasons:
    idx = [i for i in test_idx if matches[i]["season"] == s_]
    print(f"{s_:<6}" + "".join(f"{score(p, idx)[0].mean():>16.4f}" for p in models.values()))

# 配对自助法：RPS 差异的 95% 置信区间（负数 = 前者更好）
rng = np.random.default_rng(7)
def boot(a, b, idx):
    ra, rb = score(a, idx)[0], score(b, idx)[0]
    d = ra - rb
    bs = [d[rng.integers(0, len(d), len(d))].mean() for _ in range(3000)]
    return d.mean(), np.percentile(bs, 2.5), np.percentile(bs, 97.5)

print("\n配对自助法 RPS 差异（剔除2020-21，负数=前者更好）")
for (na, a), (nb, b) in (
    (("Elo", elo), ("基线", base)),
    (("泊松", poi), ("基线", base)),
    (("泊松", poi), ("Elo", elo)),
    (("Elo+泊松", ens), ("Elo", elo)),
):
    m_, lo, hi = boot(a, b, norm)
    verdict = "显著" if (lo > 0 or hi < 0) else "不显著"
    print(f"  {na} - {nb}: {m_:+.4f}  95%CI [{lo:+.4f}, {hi:+.4f}]  {verdict}")

# 校准：所有(场次,结果)对按预测概率分箱
print("\n校准（Elo+泊松平均，剔除2020-21）：预测概率 vs 实际频率")
edges = [0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.8, 1.01]
pairs = [(ens[i][k], 1.0 if matches[i]["o"] == k else 0.0) for i in norm for k in range(3)]
for lo_, hi_ in zip(edges, edges[1:]):
    sel = [(p, y) for p, y in pairs if lo_ <= p < hi_]
    if sel:
        print(f"  [{lo_:.1f},{min(hi_,1):.1f})  n={len(sel):>5}  预测 {np.mean([p for p,_ in sel]):.3f}  实际 {np.mean([y for _,y in sel]):.3f}")

json.dump(
    dict(elo_params=best_elo, poisson_params=best_poi),
    open("params.json", "w"),
)

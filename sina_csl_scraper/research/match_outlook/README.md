# 赛前胜平负倾向：研究脚本（参考实现）

这是回测研究用的一次性脚本，不属于抓取器正式代码。生产实现必须复现这里的结论，见
`docs/superpowers/specs/2026-10-18-match-outlook-design.md`。

- `backtest.py`：基线 / Elo / 泊松的滚动前向回测（调参仅用 2016–2018，2019 起样本外）。
- `fix.py`：滚动调参（V1）与自适应收缩（V2）。**生产模型即 V2。**
- `diag.py`：2026 年预测偏弱的诊断。
- `params.json`：`backtest.py` 跑出的固定参数，`diag.py` 依赖。

用法（需要各赛季比赛 JSON，每个赛季一个文件，字段同 `SinaCslClient.fetch_round_matches`
的 `MatchResult` 序列化）：

```bash
uv run --no-project --with numpy --with scipy python -I fix.py <历史数据目录>
```

历史数据来自新浪接口（2014–2026 全部可取），用 `SinaCslClient` 逐轮抓取即可。
这些脚本只用比分，不涉及任何博彩赔率，也不要引入。

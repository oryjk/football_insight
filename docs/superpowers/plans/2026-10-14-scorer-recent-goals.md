# 首页球员近十次出场进球 Implementation Plan

> **For agentic workers:** Use superpowers:executing-plans for inline implementation. User approved the compact bar design and requested implementation; preserve that execution authorization.

**Goal:** 首页射手榜同排展示本赛季最近十次联赛出场进球，3 球标注「帽」，点击查看比赛详情。

**Architecture:** Python 从中足联公开比赛日志同步经过身份和赛程校验的逐场数据。Rust 从数据库批量读取前五及并列球员的最新十次出场，随 overview 返回。Vue 使用原生 view 绘制共用高度刻度的微型柱状图。

**Tech Stack:** Python/uv/requests/psycopg, Rust/Axum/SQLx, Vue 3/uni-app/Bun.

**Spec:** 用户在当前聊天批准的射手榜迷你柱方案；追加要求单场 3 球有帽子戏法标识。

## Global Constraints

- 不将累计快照差分当作单场进球；未出场、缺数据不得伪造为 0 球。
- 仅本赛季、已完赛、有效来源、真实出场；按实际开球时间由旧到新，最多十场，不补造。
- 保留排名、头像、姓名、所属球队、赛季累计；0 球灰色短横，有进球红柱。
- 所有球员共用高度刻度；三球显示「帽」，更高进球保留真实数值。
- 后端变更后本地通过 Cargo 重启 18092 再验证；生产后端不自动部署。
- 新迁移先用隔离数据库验证；凭据不输出、不提交；共享库只执行已验证的增量迁移和真实数据同步。

## Review Focus

- 同名不同队、跨数据源身份不一致：拒绝含糊匹配。
- 缺失/负数/非整数字段与缺阵：不生成虚假零进球。
- 补赛、重复日志、旧赛季与历史快照：稳定排序、去重和截止过滤。
- 少于十场、无数据、旧后端：降级文案且不影响累计榜单。
- 窄屏、三球和四球以上：不溢出，柱高可比，点击显示真实数值。

### Task 1: 可靠逐场数据

**Files:** `sina_csl_scraper/src/sina_csl_scraper/player_match_goals.py`, `cli.py`, `tests/test_player_match_goals.py`, backend migration.

**Interfaces:** 中足联 `players/find` 与 `players/player/played` → `PlayerMatchGoals` → `f_i_player_match_stats`.

- [x] 身份匹配、零值/缺失值、分钟数、赛季、重复与赛程匹配测试先失败。
- [x] 实现严格解析、分页、身份匹配、独立事务更新；单球员失败保留旧数据并报告。
- [x] 普通同步完成后更新榜首五位及并列球员；提供独立补采入口。
- [x] 隔离数据库验证迁移及更新幂等性，运行 Python 全套测试。

### Task 2: 批量读取与协议

**Files:** insight domain overview, web DTO, persistence `recent_player_goals.rs` and overview repository.

**Interfaces:** `OverviewPlayer.recent_goals: Vec<PlayerMatchGoals>`，包含比赛 ID、日期、轮次、对阵、进球和分钟数。

- [x] 隔离 PostgreSQL 测试覆盖最新十场/少于十场、排序、历史截止、未完赛/失效赛程排除。
- [x] 批量读取球员逐场数据，保持已有架构边界；未迁移环境降级为空记录。
- [x] DTO 映射测试确认 0 球、3 球和顺序不丢失；后端全套测试及相关格式/边界检查。

### Task 3: 柱状图与验证

**Files:** mini insight types, home index, ranking panel, new goal bars component/helper, skeleton, design tokens.

- [x] 实现共用高度刻度、0 球短横、三球「帽」、点击比赛明细及缺数据降级。
- [x] 纯展示换算补必要边界测试；UI 用类型检查、H5/微信构建和浏览器验证。
- [x] 本地后端重启后验证真实五位球员数据，与来源日志独立对照。
- [x] 320/390/430 屏宽及空/短记录、3 球、4 球、点击详情检查。
- [x] 更新 FEATURES/CHANGELOG 和操作文档；提交、合并 main、推送并报告发布状态。

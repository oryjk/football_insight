@AGENTS.md

# Claude Code 补充

上面引入的 `AGENTS.md` 是本仓库的主约定（子项目、部署拓扑、统计口径、开发约定），这里只补充它没有覆盖的内容。

## 子项目约定在各自的 AGENTS.md

Claude Code 不会自动加载子目录的 `AGENTS.md`。进入某个子项目工作前，先读对应文件：

- `football_insight_service_backend_rs/AGENTS.md`
- `football_insight_mini/AGENTS.md`
- `football_insight_admin_web/AGENTS.md`
- `sina_csl_scraper/AGENTS.md`

`football_insight_h5` 没有 AGENTS.md，见其 README。`football_insight_admin_android` 已停止开发，不需要读。

## 各子项目的验证命令

| 子项目 | 命令 |
|---|---|
| 后端 | `cargo test`（`tests/*_postgres.rs` 需要可连的 PostgreSQL） |
| 小程序 | `bun run type-check`、`bun test`、`bun run check:boundaries`、`bun run verify:mp-components` |
| H5 | `bun run build`（先 `vue-tsc --noEmit` 再 `vite build`） |
| Web 后台 | `bun test`、`bun run build`（`tsc -b && vite build`） |
| 抓取器 | `uv run pytest` |

## 代码结构要点

- 后端按业务模块纵向切分，每个模块（如 `support`、`seat_swap`、`payment`）内部是 `domain` / `ports` / `application` / `adapters` 四层。业务逻辑放 `application`，不要写进 web handler 或 SQL 层。新增模块时沿用同样的目录和 `bootstrap.rs` 装配方式。
- 后端迁移在 `football_insight_service_backend_rs/migrations/`，文件名是 `YYYYMMDDHHMMSS_描述.sql`。新迁移的时间戳必须大于现有最新一条。迁移不会随部署自动执行。
- 小程序有机器化的依赖边界检查：`scripts/check-import-boundaries.mjs`（组件层不能 import `api/**`，页面之间不能互相 import，只有 `utils/request.ts` 能调用 `uni.request`）。改动涉及跨页面复用时，把逻辑放 `src/utils`、组件放 `src/components`，不要往 ALLOWLIST 里加例外。
- 小程序版本号由后端登记库分配，不要手改 `manifest.json` 的 `versionName`。提交里出现的 `chore(mini): sync allocated review version` 是发版脚本产生的。
- 抓取器入口是 `sina_csl_scraper/src/sina_csl_scraper/cli.py`（Typer，命令名 `sina-csl-scraper`）。洞察预计算在 `team_insights.py`，不要挪到后端或前端。

## 工作方式

- 提交信息沿用现有风格：`feat(mini): ...`、`fix(mini): ...`、`chore(mini): ...`、`docs: ...`，subject 用英文。
- 后端代码改动后，本地验证前先重启本地后端。汇报时分开说明：本地后端已重启 / 生产未部署 / 生产已部署并重启。
- 本地后端连接的是生产 `football_data`。涉及写操作（尤其会员商品、支付、用户数据）的测试，不要用生产的真实数据去创建或删除。
- 仓库根目录有大量 `.tmp-*.png` 截图和 `.tmp/`、`.superpowers/`、`.playwright-mcp/`，都被 `.gitignore` 忽略。临时产物放到会话的 scratchpad 目录，不要再往仓库根目录放。
- `football_insight_admin_android` 已停止开发，保留代码仅作存档。除非用户明确要求安卓相关改动，否则：
  - 不读取、不搜索、不修改、不构建该目录；
  - 全仓库 grep / 搜索时排除它（例如 `--glob '!football_insight_admin_android/**'`），也不要派子 agent 去探索它；
  - 后端、Web 后台、小程序、H5 的改动不需要同步到安卓端，也不用评估对它的影响。

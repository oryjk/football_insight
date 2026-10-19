# 战术板（最佳阵容）设计

日期：2026-10-19
状态：用户已逐节确认，待审阅书面 spec
范围：`sina_csl_scraper`（名单同步）、`football_insight_service_backend_rs`（新模块 `lineup`、`share_code` 扩展）、`football_insight_mini`（新分包页面与入口）
不涉及：独立 H5（`football_insight_h5`）、Web 后台、安卓管理端；小程序编译的 H5 端只显示"请在微信小程序中使用"

## 1. 背景与目标

用户希望做一个类似 PS 实况足球排阵型的功能：选基础阵型 → 把球员排进去 → 手动拖动微调站位 → 保存 → 分享。目标是**粘性**与**快速推广**。

### 1.1 已确认的产品决策

| 项 | 决策 |
|---|---|
| 场景 | "我心中的最佳阵容"：长期的个人表达与分享，不关联具体比赛 |
| 球队与数量 | 16 支中超球队都能排，**每队一套**（再保存即覆盖），进入时默认用户主队 |
| 编辑交互 | 点位置 → 底部弹层选人；拖动球员圆点微调站位；拖到另一名球员上即互换 |
| 前队员 | 离队与往季球员**可被选入阵容**（"如果没卖他会怎样"是讨论点），分享时带"前"角标 |
| 分享 | 分享图（含小程序码，可存相册发朋友圈）+ 小程序卡片（点开进只读页，带"我也排一套"） |
| 命名与入口 | 新功能接管「战术板」名字；洞察页"战术板 ›"与首页主队卡新入口进入排阵；旧观点帖页保留代码与数据、不再提供入口 |
| 端 | 只做微信小程序 |

### 1.2 假设（用户已认可）

- 只排首发 11 人，不做替补。
- 编辑不需要登录；保存与分享需要登录（未登录先存本地草稿，登录回来自动恢复并保存）。
- 分享出去的只读页不登录也能看。
- 首批阵型 7 个：4-3-3、4-4-2、4-2-3-1、3-5-2、3-4-3、5-3-2、4-1-4-1。
- 审核态不隐藏本功能（只有球员选择与站位，没有用户自由文本）。

### 1.3 现状（已读代码与数据核实）

- `f_i_players` 只有出现在排行榜上的球员（每队 14–19 人），没有号码和位置，不能用来排阵容。
- 新浪 `https://goal.sports.sina.cn/op/api/team/player?tid=<team_id>[&season=<season>]` 返回完整名单：成都蓉城 2026 赛季 33 人（守门员 4 / 后卫 10 / 中场 10 / 前锋 9），每人都有 `pid`、`name_cn`、`number`、`position`、`logo`、`country`、`u23`；`season=2024` 返回 31 人，`season=2025` 返回 41 人。
- 现有"战术板"（`pages/team-board`，后端 `team_board`）是球队观点帖：生产共 2 帖、7 条评论、4 个点赞。
- 可复用：抓取器头像上传器（MinIO `players/{player_id}`）、`utils/canvasPoster.ts`（绘制、导出、保存相册、授权引导、小程序码落地）、后端 `share_code` 模块（小程序码，成功缓存 24 小时、失败 30 秒）、`utils/shareSource.ts`（分享来源）。
- 小程序主包当前约 1.54MB（上限 2MB），已有 `pages/ticket-watch` 分包。

## 2. 名单数据（抓取器 → 数据库）

### 2.1 表 `f_i_squad_players`

迁移放在 `football_insight_service_backend_rs/migrations/`，文件名时间戳大于现有最新一条；按现有约定在 peiqian 手动执行 `cargo run --bin run_migrations`。

| 字段 | 类型 | 说明 |
|---|---|---|
| `team_id` | BIGINT | 主键之一 |
| `player_id` | BIGINT | 主键之一，即新浪 `pid` |
| `player_name` | TEXT NOT NULL | `name_cn`，为空时用 `name` |
| `shirt_number` | INTEGER NULL | 取最后在队赛季的号码；非数字时为空 |
| `position` | TEXT NULL | `GK`/`DF`/`MF`/`FW`，映射：守门员→GK、后卫→DF、中场→MF、前锋→FW；其他为空 |
| `country` | TEXT NULL | |
| `avatar_source_url` | TEXT NULL | 新浪原图 |
| `avatar_storage_url` | TEXT NULL | MinIO 公开地址；转存失败为空 |
| `last_season` | INTEGER NOT NULL | 最后一次出现在该队名单的赛季 |
| `active` | BOOLEAN NOT NULL | 是否在当前赛季名单里 |
| `synced_at` | TIMESTAMPTZ NOT NULL | 最近同步时间 |

主键 `(team_id, player_id)`；索引 `(team_id, active)`。同一球员在不同球队各有一行（例如转会去了另一支中超队），互不影响。

### 2.2 同步命令

- 新命令 `uv run sina-csl-scraper sync-squads --season <当前赛季> --backfill-seasons 3 --write-db`。
- 球队列表取自 `f_i_teams`（16 支）。逐队请求，**每队独立成败**：
  - 请求失败或返回结构异常：跳过该队，旧数据原样保留，不标记任何人离队。
  - 当前赛季成功：本次名单内的球员 upsert 为 `active=true`、`last_season=当前赛季`，号码、位置、头像以本次为准；该队原 `active=true` 但本次不在名单的，置 `active=false`（其 `last_season` 不变）。
- 回填：`--backfill-seasons N` 依次请求往前 N 个赛季（默认 3，即 2023–2025），只**新增**不在表里的球员或把 `last_season` 更新为更晚的赛季；回填的球员一律 `active=false`，除非当前赛季也在名单里。回填用于首次运行，之后的每日同步传 `--backfill-seasons 0`。
- 头像：沿用现有上传器，对象名 `players/{player_id}.<ext>`；MinIO 已有同名对象或表里已有 `avatar_storage_url` 时不重复上传。
- 调度：在 `sina_csl_scraper/deploy/auto-sync.cron.example` 增加每日一次的 `sync-squads`（`--backfill-seasons 0`）；首次在 local233 手动跑一次带回填的命令。
- 纯逻辑（位置映射、active/last_season 合并规则）有 pytest 单测；`uv run pytest` 通过。

## 3. 后端：`lineup` 模块

按后端约定纵向切分：`src/lineup/{domain,ports,application,adapters}` + `bootstrap.rs` 装配；业务规则在 `application`，不写进 handler 或 SQL。

### 3.1 表 `f_i_lineups`

| 字段 | 类型 | 说明 |
|---|---|---|
| `id` | UUID PK | |
| `user_id` | UUID NOT NULL → `f_i_users(id)` ON DELETE CASCADE | |
| `team_id` | BIGINT NOT NULL | |
| `formation` | TEXT NOT NULL | 7 个阵型代码之一 |
| `slots` | JSONB NOT NULL | 11 项，见 3.2 |
| `share_code` | TEXT NOT NULL UNIQUE | 8 位 base62 随机码，首次保存时生成，之后不变 |
| `created_at` / `updated_at` | TIMESTAMPTZ | |

唯一约束 `(user_id, team_id)`：每人每队一套，保存为 upsert。

### 3.2 `slots` 结构与校验

```json
[{ "slot": 0, "player_id": 156047, "x": 0.5, "y": 0.9 }, …]
```

- `slot`：0–10，与阵型定义中的位置序号一一对应（见 5.2），必须正好 11 项、不重复。
- `player_id`：可为空（允许保存不完整阵容）；非空时必须属于该 `team_id` 的 `f_i_squad_players`（现役或前队员均可），同一阵容中不得重复。
- `x`、`y`：0–1 的比例坐标，球场竖向、己方球门在下（`y` 越大越靠近己方球门）；超出范围拒绝。
- `formation` 不在 7 个支持值内拒绝。

错误用领域错误枚举表达，按现有模块风格返回 400 与中文提示。

### 3.3 接口

| 方法与路径 | 登录 | 说明 |
|---|---|---|
| `GET /api/v1/lineups/squads/{team_id}` | 否 | 名单：`{ team: {team_id, team_name, avatar_url}, active: [...], former: [...] }`；每名球员 `{player_id, name, shirt_number, position, avatar_url, last_season}`；`active` 按位置（GK、DF、MF、FW、空）再按号码排序，`former` 按 `last_season` 倒序再按位置 |
| `GET /api/v1/lineups/me/{team_id}` | 是 | `{ lineup: null }` 或 `{ lineup: {formation, slots, share_code, updated_at} }` |
| `PUT /api/v1/lineups/me/{team_id}` | 是 | body `{formation, slots}`；upsert 后返回同上结构 |
| `GET /api/v1/lineups/shared/{share_code}` | 否 | 只读：`{ author: {display_name, avatar_url}, team, formation, slots, players: {player_id → 名单字段}, updated_at }`；不返回用户 id 或其他个人信息；不存在返回 404 |
| `GET /api/v1/lineups/shared/{share_code}/wxacode` | 是 | 分享图用小程序码图片 |

### 3.4 `share_code` 模块扩展

现有实现把 scene `s=share&c=ticket_poster`、page `pages/ticket-watch/index` 写死。扩展为由调用方传入 scene 与 page 的通用能力，保留 ticket_poster 现有行为与缓存语义（成功 24 小时、失败 30 秒，不强刷 stable token），并加回归测试。

阵容码：scene `s=share&c=lineup&l=<share_code>`（27 字符，小于 32 上限），page `pages/lineup/view`，`env_version` 保持 `release`。

**已知限制**：`release` 小程序码只能打开正式版。上线前在体验版扫分享图上的码进不了只读页；体验阶段用小程序卡片验证，扫码在正式发布后验证。

### 3.5 测试

- 领域校验与 upsert/share_code 生成的单测。
- `tests/*_postgres.rs` 覆盖：每人每队唯一与覆盖、前队员可保存、他队球员拒绝、重复球员拒绝、公开读取不泄露 user_id、share_code 唯一。
- HTTP 层：未登录 401、阵型/坐标非法 400、不存在的 share_code 404。
- 后端代码改动后，本地验证前重启本地后端；汇报区分本地已重启 / 生产未部署 / 生产已部署。

## 4. 小程序：编辑页

### 4.1 页面与分包

- 新分包 `pages/lineup`，含 `index`（编辑，标题"战术板"）与 `view`（只读）。
- 入口：洞察页 `InsightsTeamPanel` 的"战术板 ›"改为 `pages/lineup/index?teamId=<id>`；首页主队卡新增"排我的最佳阵容"入口（无主队时进入后默认第一支球队并显示球队选择）。
- 旧 `pages/team-board` 页面与后端 `team_board` 模块保留不动，已分享的旧链接仍可打开；只是不再有入口。
- 编译到 H5 时页面只显示"请在微信小程序中使用"（`#ifdef` 隔离），不实现 H5 交互。

### 4.2 页面结构

```
FiSecondaryNav（战术板）
球队胶囊 [成都蓉城 ▾]                         保存状态：已保存 / 未保存
阵型条   [4-3-3][4-4-2][4-2-3-1][3-5-2][3-4-3][5-3-2][4-1-4-1]（横滑）
球场     竖向，纯色草坪 + 白色线条（边线、中线、中圈、两个禁区），己方球门在下
         11 个球员圆点：头像 + 右下号码 + 下方姓名；前队员左上"前"角标；
         无头像显示号码；空位显示"+"与位置名（门将/中卫/边卫/后腰/中场/前腰/边锋/前锋）
底部操作  重置站位    保存（主按钮）    分享
```

视觉遵循 `docs/mini-design-system.md`：颜色、字号、间距、圆角、阴影、动效引用 token；新增草坪/线条色先加 primitive 再建 semantic；`src/pages/lineup` 纳入与换座页同类的样式审计测试（无渐变、无散落 hex、无数字字重）。

### 4.3 交互

- **点圆点**：弹出 `LineupPlayerSheet`（`FiBottomSheet`）：顶部是位置名与（已有人时）"移出"按钮；"现役"分组在前，按位置分小组，与该位置类别对口的小组排最上；"前队员"分组默认折叠，点开展开，每人带"某赛季在队"标签；已在场上的球员置灰（点选它则与当前位置互换）。选中即放入并关闭弹层。
- **拖动圆点**：`movable-area` / `movable-view` 原生拖动，`out-of-bounds=false`、无惯性；松手时：
  - 位移 < 10rpx 视为点击（打开选人弹层），不改坐标；
  - 落点与另一名球员中心距离小于圆点直径的 0.6 倍 → 两人互换位置（各自回到对方原坐标，220ms 过渡）；
  - 否则停在落点，坐标写回比例值（钳制在 0.04–0.96）。
- **切换阵型**：已排球员按位置类别重新分配到新阵型同类别空位（GK→GK，DF→DF…），同类别空位不足的按 GK、DF、MF、FW 顺序放入剩余空位；所有坐标重置为新阵型默认值。
- **重置站位**：坐标恢复当前阵型默认值，球员不动。
- 动效只动 transform/opacity，引用 `--fi-motion-*`；遵守 reduced-motion 约定。

### 4.4 保存、登录与状态

- 进入：有已保存阵容则加载；否则默认 4-3-3 空阵；若有本地草稿（见下）优先恢复草稿并标"未保存"。
- 保存：已登录直接 `PUT`；未登录先把 `{teamId, formation, slots}` 存入本地草稿（`uni.setStorageSync`，按球队分键），记住登录后回跳，跳转登录；回到页面后自动恢复草稿并保存，成功后清草稿。
- 有改动时显示"未保存"；离开页面前若未保存，弹一次确认。
- 名单里某球员变成前队员不影响已保存阵容；名单里已不存在的 `player_id`（异常数据）在编辑页显示为空位并提示。

### 4.5 分层

| 文件 | 职责 |
|---|---|
| `pages/lineup/formations.ts` | 7 个阵型定义（位置序号、类别、位置名、默认坐标），纯数据 |
| `pages/lineup/helpers.ts` | 纯函数：换阵型重分配、松手判定（点击/互换/移动）、坐标钳制、选人弹层排序与分组、分享条件判定 |
| `pages/lineup/useLineupPage.ts` | 数据层：加载名单与我的阵容、保存、草稿读写；唯一调用 `api/lineup.ts` 的地方 |
| `pages/lineup/useLineupEditor.ts` | 编辑状态：formation、slots、选中位置、dirty |
| `pages/lineup/components/*` | 纯展示：`LineupPitch`（含只读模式）、`LineupPlayerDot`、`LineupPlayerSheet`、`LineupFormationBar`、`LineupTeamPicker` |
| `api/lineup.ts`、`types/lineup.ts` | 接口与类型 |

组件不 import `api/**`；页面之间不互相 import（编辑页与只读页共用的组件放在 `pages/lineup/components`，同分包内）；不加 ALLOWLIST 例外。纯函数先写失败测试再实现。

## 5. 阵型定义

### 5.1 坐标约定

`x` 从左到右 0–1，`y` 从对方球门（0）到己方球门（1）。门将固定 `(0.50, 0.90)`。

### 5.2 默认站位

| 阵型 | 后卫（y≈0.74） | 中场 | 前锋 |
|---|---|---|---|
| 4-3-3 | x 0.12 / 0.37 / 0.63 / 0.88 | y 0.52：x 0.25 / 0.50 / 0.75 | 边锋 y 0.24：x 0.18 / 0.82；中锋 y 0.18：x 0.50 |
| 4-4-2 | x 0.12 / 0.37 / 0.63 / 0.88 | y 0.50：x 0.12 / 0.37 / 0.63 / 0.88 | y 0.20：x 0.35 / 0.65 |
| 4-2-3-1 | x 0.12 / 0.37 / 0.63 / 0.88 | 后腰 y 0.60：x 0.35 / 0.65；前腰 y 0.38：x 0.18 / 0.50 / 0.82 | y 0.18：x 0.50 |
| 3-5-2 | x 0.25 / 0.50 / 0.75 | y 0.52：x 0.08 / 0.92；y 0.56：x 0.30 / 0.70；y 0.44：x 0.50 | y 0.20：x 0.35 / 0.65 |
| 3-4-3 | x 0.25 / 0.50 / 0.75 | y 0.50：x 0.12 / 0.37 / 0.63 / 0.88 | 边锋 y 0.24：x 0.18 / 0.82；中锋 y 0.18：x 0.50 |
| 5-3-2 | 边翼卫 y 0.70：x 0.06 / 0.94；中卫 y 0.76：x 0.27 / 0.50 / 0.73 | y 0.50：x 0.25 / 0.50 / 0.75 | y 0.20：x 0.35 / 0.65 |
| 4-1-4-1 | x 0.12 / 0.37 / 0.63 / 0.88 | 后腰 y 0.62：x 0.50；y 0.44：x 0.12 / 0.37 / 0.63 / 0.88 | y 0.18：x 0.50 |

位置序号：0 = 门将，其后按后卫、中场、前锋，各类别内从左到右编号。每个位置带类别（GK/DF/MF/FW）与位置名，用于空位提示与选人排序。实现时用单测保证：每个阵型正好 11 个位置、恰好 1 个 GK、坐标在 0–1 内、任意两点间距不小于圆点直径。

## 6. 分享

### 6.1 只读页 `pages/lineup/view?code=<share_code>`

- 来自小程序码时从 `scene` 解析 `l`；来自卡片时读 `code` 参数。
- 内容：作者头像与昵称、"成都蓉城 · 4-3-3"、更新日期；`LineupPitch` 只读模式（不可拖、不可点）；前队员带"前"角标。
- 底部固定主按钮"我也排一套" → `pages/lineup/index?teamId=<同队>`；若当前用户已有该队阵容，按钮为"看看我的阵容"。
- 不需要登录；share_code 不存在显示"这套阵容已不存在"。

### 6.2 小程序卡片

编辑页（已保存且满 11 人时）与只读页都实现 `onShareAppMessage`：
- 标题：`我心中的{球队}最佳阵容（{阵型}）`；
- `path`：`buildSharePath('/pages/lineup/view?code=<share_code>', 'lineup')`；
- `imageUrl`：用同一绘制逻辑导出的 5:4 球场阵容图（失败时退回默认截图）。

### 6.3 分享图

- 编辑页"分享"按钮：未登录走保存登录流程；有未保存改动先保存；未满 11 人提示"还差 N 人"。
- 750×1200 竖图：顶部队徽、球队名、"我的最佳阵容"、阵型；中间球场与 11 名球员（圆形头像、号码、姓名、前队员"前"角标）；底部作者昵称、小程序码与"长按识别，排你的最佳阵容"。
- 绘制前用 `getImageInfo` 下载头像（MinIO 域名需在 `downloadFile` 合法域名内，实现时核对）；单个头像失败用号码圆占位，不阻断出图；小程序码失败提示重试，不导出无码图片。
- 预览后保存相册，复用 `savePosterWithPermission`。
- 绘制布局函数（坐标换算、文字截断）为纯函数并单测。

## 7. 交付阶段

| 阶段 | 内容 | 可独立验证 |
|---|---|---|
| 一 | 迁移 `f_i_squad_players`；抓取器 `sync-squads`（含回填）；后端 `GET /lineups/squads/{team_id}` | pytest；本地后端读到 16 队名单 |
| 二 | 迁移 `f_i_lineups`；后端我的阵容读写；小程序编辑页、入口改造、草稿与登录 | 后端测试；小程序可排、可保存、可再次打开 |
| 三 | `share_code` 扩展；公开读取与小程序码接口；只读页、卡片、分享图 | 卡片分享进只读页；分享图生成与保存（扫码待正式版） |

每阶段：提交并推送 main；后端改动后重启本地后端；生产部署、迁移执行与小程序上传按负责人指令进行。

## 8. 约束

1. 小程序：只做 mp-weixin；`bun test`、`bun run type-check`、`bun run check:boundaries`、`bun run verify:mp-components`、`bun run build:mp-weixin` 通过；不手改 `manifest.json` 版本号。
2. 后端：`cargo test` 通过（postgres 测试用一次性数据库）；不在生产库用真实用户做写入测试。
3. 抓取器：`uv run pytest` 通过；写库前先在本地跑通。
4. 不读取、搜索或修改 `football_insight_admin_android`。
5. 提交信息沿用 `feat(mini)` / `feat(backend)` / `feat(scraper)` / `docs:`，subject 英文；`CHANGELOG.md`、`FEATURES.md` 各补一条并写明原因。

## 9. 范围外

- 替补席、阵容命名或多套阵容；
- 点赞、评论、"多少人排了同样阵容"、阵容排行；
- 与真实首发对比、按比赛预测首发；
- 观点帖战术板的数据清理与下线；
- 独立 H5 与 Web 后台的阵容管理。

## 10. 审查清单

1. 名单同步：每队独立成败，失败不误标离队；回填只新增或推后 `last_season`；头像不重复上传。
2. 名单接口：现役与前队员分组、排序符合 3.3；前队员可被保存进阵容，他队球员被拒。
3. 每人每队唯一，再保存覆盖；share_code 首次生成后不变且唯一；公开读取不含 user_id。
4. `share_code` 扩展后 ticket_poster 行为与缓存不变，有回归测试；阵容码 scene ≤ 32 字符。
5. 编辑页：点选、拖动微调、拖到他人互换、点击与拖动的 10rpx 判定、换阵型重分配、重置站位均符合 4.3；动效引用 token。
6. 未登录草稿 → 登录 → 自动恢复并保存，成功后清草稿；离开未保存有一次确认。
7. 分享条件（已保存、满 11 人）、卡片路径带分享来源、只读页"我也排一套"跳同队编辑页。
8. 分享图：头像失败占位不阻断、小程序码失败不出无码图、保存相册走授权引导。
9. 分包生效，主包体积不显著增长；组件注册检查与边界检查通过；`src/pages/lineup` 无渐变、散落 hex、数字字重。
10. 旧观点帖页面可直接访问但无入口；洞察页与首页入口指向新页面。

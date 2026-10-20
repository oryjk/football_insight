# 转会投票设计

日期：2026-10-20
状态：设计已逐段确认，待负责人审阅本文档

## 1. 目标与边界

**目标**：休赛期没有比赛和抢票时，给球迷一个回来表态、愿意转发的理由，提升粘性。数据背景：注册几乎全部集中在成都蓉城主场开票前 1–2 天，两次主场之间每周只有个位数；月活 299、周活 25（2026-10-20 只读统计）。

**做什么**：球迷对自己主队的转会意向投票，分三类——希望谁离队、希望谁留下、希望买入谁；其他球迷对每条提名赞成或踩；支持好友卡片分享和"帮我投一票"海报。

**不做什么**：
- 不做任何文字留言、评论或自由输入的内容（审查风险）。提名弹层里的搜索框只在本地过滤候选列表，不提交到后端。
- 不提名中超以外的球员（库里没有数据，也不允许自由输入）。
- 不做管理后台页面；新开一期用一条 SQL。
- 审核期间**不隐藏**（负责人确认）。

## 2. 全局约束

- 界面风格与现有首页、洞察页、战术板保持一致。
- 组件化：组件只接收属性、发出事件；调接口和跳转放在页面或组合函数里（`check:boundaries` 机器保证）。
- token 化：颜色、字号、间距、圆角全部引用 `fi-tokens.css`；需要新值先加 primitive 再建 semantic。全局 `style-token-audit` 自动覆盖新文件，新分包另加样式审计（无渐变、无散写 hex、无数字字重）。
- 后端六边形架构：新模块 `transfer`，分 `domain` / `ports` / `application` / `adapters` + `bootstrap.rs`；业务规则只在 domain 与 application。
- 主包距 2MB 上限约 450KB：完整页面放新分包 `pages/transfer`，主包只放洞察页面板与共用组件（预计 15–20KB）。
- 不在生产库做写入测试；SQL 测试用一次性 Docker PostgreSQL。生产迁移、部署、上传均需负责人确认。

## 3. 名词

| 名词 | 含义 |
|---|---|
| 一期 / 转会窗口 | 一段投票周期，如"2026 冬窗"。每期榜单独立，往期保留只读 |
| 意向 intent | `out` 希望离队、`stay` 希望留下、`in` 希望买入（本队旧将显示为"希望回归"） |
| 提名卡 | "一期 + 球队 + 球员 + 意向"唯一的一张卡 |
| 投票 | 某用户对某张卡的态度：+1 赞成、−1 踩；取消即删除 |
| 净支持 | 赞成数 − 踩数 |

## 4. 数据模型

迁移文件时间戳须大于现有最新 `20261019160000`。

### 4.1 `f_i_transfer_windows`

| 字段 | 类型 | 说明 |
|---|---|---|
| id | BIGSERIAL PK | |
| code | TEXT UNIQUE NOT NULL | 如 `2026-winter` |
| name | TEXT NOT NULL | 如 `2026 冬窗` |
| opens_at | TIMESTAMPTZ NOT NULL | |
| closes_at | TIMESTAMPTZ NOT NULL | CHECK `closes_at > opens_at` |

第一期由迁移写入：`2026-winter` / `2026 冬窗` / `2026-10-20T00:00:00+08:00` 至 `2027-03-01T00:00:00+08:00`。

**当前一期**：`opens_at <= now() < closes_at` 的那一期（若重叠取 `opens_at` 最新）；没有开放期时取 `closes_at` 最新的一期，只读。一期都没有时接口返回空，前端显示"暂无转会投票"。

### 4.2 `f_i_transfer_nominations`

| 字段 | 类型 | 说明 |
|---|---|---|
| id | UUID PK | |
| window_id | BIGINT NOT NULL FK → windows ON DELETE CASCADE | |
| team_id | BIGINT NOT NULL FK → f_i_teams | 投票所属球队 |
| player_id | BIGINT NOT NULL | 对应 `f_i_squad_players.player_id` |
| intent | TEXT NOT NULL CHECK IN (`out`,`stay`,`in`) | |
| share_code | TEXT UNIQUE NOT NULL | 8 位，字符集与战术板分享码一致 |
| created_by | UUID NOT NULL FK → f_i_users ON DELETE CASCADE | 提名人 |
| created_at | TIMESTAMPTZ NOT NULL DEFAULT now() | |

约束：UNIQUE (`window_id`, `team_id`, `player_id`, `intent`)。索引：(`window_id`, `team_id`, `intent`)；(`window_id`, `created_by`, `intent`) 用于数上限。

### 4.3 `f_i_transfer_votes`

| 字段 | 类型 | 说明 |
|---|---|---|
| nomination_id | UUID NOT NULL FK → nominations ON DELETE CASCADE | |
| user_id | UUID NOT NULL FK → f_i_users ON DELETE CASCADE | |
| value | SMALLINT NOT NULL CHECK IN (1, −1) | |
| created_at / updated_at | TIMESTAMPTZ NOT NULL DEFAULT now() | |

PRIMARY KEY (`nomination_id`, `user_id`)：每人每卡一行，赞踩天然互斥，不需要触发器。索引：(`user_id`)。

## 5. 业务规则（domain + application）

1. **资格**（提名与投票共用）：已登录、账号 `status = 'active'`、请求的球队等于用户 `favorite_team_id`、该期处于开放中。改主队后已投的票保留，不追溯。
2. **候选资格**（后端按名单表校验，不信任前端）：
   - `out` / `stay`：该球队 `active = true` 的球员。
   - `in`：其他球队 `active = true` 的球员，或本队 `active = false` 的旧将（展示为"希望回归"）。"旧将"指名单表里有该球员在本队 `active = false` 的记录；即使他现在在其他中超队现役，也按旧将处理：`returning = true`，显示"希望回归"，并显示他当前所在队。
3. **提名**：卡不存在则创建并为提名人记 +1；卡已存在则为提名人记 +1（若已投 −1 则改为 +1），返回 `created = false`，前端提示"已有球迷提名，已为你加一票"。
4. **提名上限**：每人每期每种意向最多**新建** 3 张卡；投票不限。上限只按 `created_by` 计数，为已存在卡加票不占名额。
5. **投票**：`PUT` 设为 +1 或 −1（覆盖原值），`DELETE` 删除。提名人可以取消或改投自己的卡。
6. **空卡隐藏**：一票都没有（投票行数为 0）的卡不出现在榜单和预览里；单卡页仍可通过分享码打开并显示 0 票。
7. **排序**：净支持降序 → 赞成数降序 → `created_at` 升序。名次按此排序计算。
8. **球员动态信息**：名字、头像、位置、号码、当前所在队均实时读名单表，不做快照。`out` / `stay` 卡的球员若已不在本队现役，返回 `departed = true`，前端显示"已离队"，卡片保留。

## 6. 接口

前缀 `/api/v1/transfers`。浏览接口登录可选（带 token 时返回 `my_vote`）；写接口与小程序码必须登录。响应加 `no-store`（含个人状态，与战术板一致）。

| 方法 | 路径 | 说明 |
|---|---|---|
| GET | `/windows/current` | `{ window: { code, name, opens_at, closes_at, open } \| null }` |
| GET | `/teams/{team_id}/summary?window=` | 三榜各前 3 + 各榜卡片数 + 本期信息 + `can_vote`（已登录且为主队且开放） |
| GET | `/teams/{team_id}/board?intent=&page=&window=` | 单榜分页，每页 20 |
| GET | `/nominations/{share_code}` | 单卡 + 球队 + 期 + 名次 |
| GET | `/teams/{team_id}/candidates?intent=` | `out`/`stay`：本队现役；`in`：其他队现役（按队分组）+ 本队旧将 |
| POST | `/teams/{team_id}/nominations` | body `{ player_id, intent }` → `{ nomination, created }` |
| PUT | `/nominations/{share_code}/vote` | body `{ value: 1 \| -1 }` → `{ up, down, my_vote }` |
| DELETE | `/nominations/{share_code}/vote` | → `{ up, down, my_vote: 0 }` |
| GET | `/nominations/{share_code}/wxacode` | 第二期；返回 base64 PNG |

**卡片对象**：`share_code`、`intent`、`returning`（`in` 且为本队旧将，定义见第 5 节第 2 条）、`player { player_id, name, avatar_url, position, shirt_number, current_team { team_id, team_name } | null, departed }`、`up`、`down`、`my_vote`（−1/0/1）、`rank`、`created_at`。

**错误**（`code` 字段供前端给出明确文案）：

| HTTP | code | 场景 | 前端文案 |
|---|---|---|---|
| 401 | `unauthorized` | 未登录或令牌失效 | 跳登录并带回跳 |
| 403 | `not_favorite_team` | 不是主队 | 只能为你的主队投票 |
| 404 | `not_found` | 卡或球队不存在 | 这张提名不存在 |
| 409 | `window_closed` | 本期不在开放中 | 本期投票已结束 |
| 422 | `invalid_candidate` | 球员不在候选范围 | 这位球员不能这样提名 |
| 429 | `nomination_limit` | 本期这一类已建 3 张 | 本期这一类最多提名 3 位 |

## 7. 后端结构（六边形）

`football_insight_service_backend_rs/src/transfer/`：

- `domain.rs`：`TransferIntent`、`VoteValue`、`TransferWindow`、`NominationCard`、`TransferError`；纯函数 `candidate_allowed(intent, player_membership)`、`nomination_limit_reached(count)`、`rank_cards(...)`、`is_open(window, now)`、分享码校验。
- `ports.rs`：
  - `TransferRepository`：期查询、榜单/预览/单卡读取、事务内提名（含上限计数与加票）、投票设置/删除。
  - `TransferCandidateDirectory`：读名单表，回答"某球员相对某队的身份"和候选列表。
  - `TransferUserAccess`：账号状态与主队。
  - `TransferCodePort`：小程序码（第二期）。
- `application.rs`：`TransferService`，编排"资格 → 候选 → 写入 → 读回"，不含 SQL 和 HTTP。
- `adapters/`：`persistence.rs`（SQLx）、`web.rs`（Axum 路由）、`dto.rs`（请求/响应与错误映射）。小程序码适配器放在现有 `share_code/adapters/wechat_code` 下新增转会场景，复用微信接口与缓存。
- `bootstrap.rs`：装配，挂到 `app.rs`，写法与 `lineup` 一致。

**并发**：
- 提名在单个事务内：`SELECT … FROM f_i_users WHERE id = $1 FOR UPDATE` 锁提名人 → 数本期该意向已建卡数 → `INSERT … ON CONFLICT (window_id, team_id, player_id, intent) DO NOTHING RETURNING` → 无返回则读已存在卡 → upsert 投票 +1。同一人连点不会超过上限；两人同时提名同一球员只产生一张卡、两票都计。
- 分享码冲突（唯一约束）时重新生成，最多重试 5 次。
- 投票：`INSERT … ON CONFLICT (nomination_id, user_id) DO UPDATE SET value, updated_at`；取消为 `DELETE`。写入前在同一事务内确认期仍开放、用户仍有资格。

## 8. 小程序

### 8.1 主包

- `components/FiReactionBar.vue`：由 `FiLineupReactionBar.vue` 改名，新增可选属性 `dislikeCount`（传入时在踩一侧显示数字）。战术板三处引用同步改名，不传 `dislikeCount`，外观不变。
- `components/FiTransferCard.vue`：一行提名卡（名次、头像、名字、位置/当前所在队、意向标签"希望回归"、"已离队"标记、`FiReactionBar` 显示赞踩两个数）；只接收属性、发出 `open` / `vote` 事件。洞察页与分包共用。
- `pages/insights/components/InsightsTransferPanel.vue`：顶部"2026 冬窗 · 投票中/已结束"；离队、留队、引援三个小榜各前 3，带"查看全部 ›"；底部"提名球员"。不是主队时显示"只能为你的主队投票"。
- `pages/insights/useInsightsTransfer.ts`：仅在"转会"胶囊激活时加载，跟随页面顶部选中球队；处理换队、换账号、返回页面刷新。
- `pages/insights/index.vue`：`mainTabs` 增加 `{ key: 'transfer', label: '转会' }`；分享标题在转会胶囊下用榜单标题。
- `composables/useTransferVotes.ts`：赞踩共享逻辑，参照 `useLineupCommunity` 的点赞部分（请求中锁卡、游客跳登录并回跳、换账号/换队后迟到响应不覆盖、失败保留原状态并提示）。
- `api/transfer.ts`、`types/transfer.ts`、`utils/transferLinks.ts`。
- 首页 `HomeSupportPanel`：卡片底部"排阵容"入口旁增加"转会投票"入口，发出事件由首页跳转。

### 8.2 分包 `pages/transfer`

- `board.vue`：`FiSegmented` 切换离队 / 留队 / 引援（可由链接参数指定）；分页列表，上拉加载；底部固定"提名球员"。
- `view.vue`：单卡落地页，大号卡片 + 中号赞踩胶囊；"分享拉票"（第一期为好友卡片，第二期加海报）、"看完整榜单"、"我也提名"。
- `components/TransferNominateSheet.vue`：基于 `FiBottomSheet`；先选意向，再选球员；`in` 候选按球队分组，旧将单列"本队旧将"；本地名字搜索框只过滤列表。
- `transferPoster.ts`（第二期）：复用 `utils/canvasPoster.ts`。

### 8.3 状态

| 状态 | 表现 |
|---|---|
| 游客 | 可浏览；赞踩/提名跳登录，登录后回到原页 |
| 未设主队 / 非主队 | 胶囊与提名按钮置灰，提示"只能为你的主队投票" |
| 本期已结束 | 只读，胶囊置灰，顶部"本期投票已结束" |
| 加载 / 出错 | 骨架屏；出错显示"重试"，与洞察页一致 |
| 空榜 | "还没有人提名，来做第一个" + 提名按钮（可投票时） |

## 9. 分享

- 所有分享路径带现有分享来源参数（`utils/shareSource`），`cid` 分别为 `transfer_board`、`transfer_card`。
- 榜单分享：标题"{球队}球迷最想送走谁？{期名}转会榜"（留队榜"最想留下谁"，引援榜"最想买入谁"），落地 `pages/transfer/board?teamId=&intent=`。
- 单卡分享：标题"我希望 {球员} {离队/留下/加盟/回归}，你同意吗？"，落地 `pages/transfer/view?code=`，对方可直接赞踩。
- 海报（第二期）：750 宽；队徽 + 期名；球员大头像与名字；"{球队}球迷希望他{离队/留下/加盟/回归}"；赞数、踩数、"{榜名}第 N"；分享人头像昵称"邀你表态"；小程序码"扫码投一票"。小程序码场景值为分享码，成功缓存 24 小时、失败 30 秒；失败明确提示重试，不导出无码海报。头像失败用昵称首字占位，保存相册与授权引导复用现有工具。

## 10. 分期

- **第一期**：后端模块、三张表与第一期数据；洞察页转会胶囊、完整榜单页、单卡页、提名弹层；赞踩；好友卡片分享；首页入口；`FiReactionBar` 改名与踩数显示。
- **第二期**：小程序码接口与"帮我投一票"海报。

## 11. 测试

- **domain / application**：纯单元测试 + 假实现，覆盖资格、候选三类、旧将回归、上限、重复提名加票、赞踩切换与取消、排序、空卡隐藏、期开放判断。
- **SQL**：一次性 Docker PostgreSQL 集成测试（`#[ignore]`，参照 `tests/lineup_postgres.rs`），覆盖并发提名同一球员只一张卡、同一人并发提名不超上限、投票 upsert 与删除、级联删除。
- **Web**：路由鉴权（写接口 401）、错误码映射、`no-store`。
- **小程序**：组合函数的请求隔离与登录回跳；`FiReactionBar` 改名后战术板行为不变；分包样式审计；`type-check`、`check:boundaries`、`build:mp-weixin`、`verify:mp-components`。

## 12. 风险与待观察

- 球员名单每日同步，球员在一期中途转会后卡片信息随之变化（如"希望买入"的球员已经加盟本队）。第一期不做特殊处理，展示实时所在队即可。
- 提名上限 3 张是否合适，上线后看数据调整（常量在 domain）。
- 审核风险：内容只有球员与表态，无自由文本；负责人确认审核期间不隐藏。

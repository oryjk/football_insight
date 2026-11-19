# 「我来当总监」休赛期建队选人功能设计文档

- 日期：2026-11-19
- 模块名：后端 `director`（路由前缀 `/api/v1/director`），小程序分包 `pages/director`
- 关联既有能力：复用 `f_i_transfer_windows`（转会窗口）、`f_i_squad_players`（球队现役名单）、`f_i_teams`（球队基础信息）、`ContentSecurityPort`（内容安全检测）

---

## 1. 产品目标与核心规则

在休赛期为球迷提供 1 分钟可完成的「主队休赛期建队模拟 + 微信群熟人分歧 PK」轻社交玩法：

1. **窗口与球队**
   - 复用 `f_i_transfer_windows` 当前开启窗口（无开启窗口则回退最近已关闭窗口，逻辑与 `transfer` 模块一致）。
   - 支持中超全部 16 队切换；默认选中用户主队（`favorite_team_id`），未设置主队时默认 `77680`（成都蓉城）。
   - 每位登录用户在「同一窗口 + 同一球队」保留 **1 套方案**，可随时覆盖更新，保留原有 `share_code` 和 `circle_code`。
2. **选人规则（核心约束）**
   - **现役外援留/走**：从 `f_i_squad_players`（`active = true`）按 `country` 判定外援，用户对现役外援逐个表态 `stay`（留队）或 `leave`（离队）。
   - **5 外援名额上限**：`留队现役外援数 + 新签外援数 <= 5`（至少留/签 1 名外援才允许发布）。
   - **外援补强来源（3 层）**：
     - 精选海外/绯闻候选池（`f_i_director_candidates`，重点含 **K 联赛**、J 联赛、中超热门外援与绯闻目标）；
     - 中超其他球队现役外援（来自 `f_i_squad_players`）；
     - 用户自由提名（填姓名 + 位置 + 联赛/俱乐部，用于兜底冷门 K 联赛/海外球员）。
   - **国内重点补强 / 离队建议（可选）**：
     - 最多选 3 名国内引援（来自候选池、其他中超球队或自由提名）；
     - 最多选 3 名本队现役国内球员标为离队建议（`domestic_outs`）。
   - **总监宣言（可选）**：最多 40 个 Unicode 字符，提交前走 `ContentSecurityPort`（`ModerationScene::Comment`）检测。
3. **风格标签（纯函数自动计算）**
   - 优先级从高到低：
     1. 新签外援中存在 `league_tag` 含 `K`（如 `K1`、`K2`、`K联赛`）且数量 >= 1 → **`K联赛淘宝流`**（如同时离队现役外援 >= 3 且新签外援 >= 3，优先 **`推倒重建派`**，否则 **`K联赛淘宝流`**）；
     2. 离队现役外援 >= 3 → **`推倒重建派`**；
     3. 留队现役外援 >= 4 且新签外援 <= 1 → **`稳健微调派`**；
     4. 新签球员中前锋（`FW`）占比 >= 50% 且总数 >= 2 → **`锋线狂魔`**；
     5. 新签球员中后卫/门将（`DF`/`GK`）占比 >= 50% 且总数 >= 2 → **`铁血防线派`**；
     6. 兜底 → **`务实均衡派`**。
4. **外援判定规则（`f_i_squad_players.country`）**
   - `country` 为空（`NULL` 或空串），或 trim 后属于国内/港澳台集合（`"中国"`, `"China"`, `"CHN"`, `"中国香港"`, `"香港"`, `"Hong Kong"`, `"中国澳门"`, `"澳门"`, `"Macau"`, `"中国台湾"`, `"台湾"`, `"中华台北"`, `"Taiwan"`, `"Chinese Taipei"`，大小写不敏感）→ 判定为 **国内球员（`is_foreign = false`）**；
   - 其余非空国籍 → 判定为 **外援（`is_foreign = true`）**。
5. **隐式好友圈（`circle_code`）与 1v1 分歧报告**
   - 用户独立新建方案时，自动生成 8 位大写字母数字 `share_code` 与独立 `circle_code`（初始 `circle_code = share_code`）。
   - 用户 B 通过用户 A 的分享链接（携带 `from_code = A.share_code` 或 `circle_code = A.circle_code`）进入并提交自己的方案时：
     - 若用户 B 是首次为该窗口+该球队建方案且传入了合法的 `circle_code`（对应同窗口同球队存在方案），则继承该 `circle_code`，自动加入同一好友圈；
     - 保存后前端自动展示 **「你和 @A 的 1v1 分歧报告」**，并可在详情页切换查看 **「本圈子共识」**（满 2 人即展示：最受欢迎 5 外援、最想送走球员离队率、圈子内所有好友方案列表）。
   - 游客可先在编辑页完整试选；点击“保存并生成方案”时引导登录并回跳。

---

## 2. 数据库表结构（迁移契约）

### 2.1 结构迁移：`migrations/20261119120000_add_director_plans.sql`

```sql
-- 1. 精选引援候选池（重点覆盖 K 联赛、J 联赛、绯闻外援及国内重点目标）
CREATE TABLE IF NOT EXISTS f_i_director_candidates (
    id BIGSERIAL PRIMARY KEY,
    code TEXT NOT NULL UNIQUE,
    player_name TEXT NOT NULL,
    position TEXT NOT NULL CHECK (position IN ('GK', 'DF', 'MF', 'FW')),
    country TEXT NOT NULL,
    is_foreign BOOLEAN NOT NULL DEFAULT TRUE,
    league_tag TEXT NOT NULL,              -- 如 'K1', 'K2', 'J1', 'CSL', 'FREE', 'BRA'
    current_club TEXT NOT NULL,            -- 如 '蔚山HD', '全北现代', '自由身'
    tags JSONB NOT NULL DEFAULT '[]'::jsonb, -- 字符串数组，如 ["亚外","快马","绯闻"]
    target_team_id BIGINT,                 -- NULL 表示全联赛通用，非空表示重点推荐给某队（如 77680）
    avatar_url TEXT,
    sort_order INTEGER NOT NULL DEFAULT 100,
    active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_f_i_director_candidates_filter
    ON f_i_director_candidates (active, is_foreign, league_tag, sort_order);

-- 2. 用户建队方案（同窗口 + 同球队 + 同用户唯一）
CREATE TABLE IF NOT EXISTS f_i_director_plans (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    window_id BIGINT NOT NULL REFERENCES f_i_transfer_windows(id) ON DELETE CASCADE,
    team_id BIGINT NOT NULL REFERENCES f_i_teams(team_id),
    user_id UUID NOT NULL REFERENCES f_i_users(id) ON DELETE CASCADE,
    share_code TEXT NOT NULL UNIQUE,
    circle_code TEXT NOT NULL,
    parent_share_code TEXT,
    foreign_stays JSONB NOT NULL DEFAULT '[]'::jsonb,   -- 现役留队外援列表
    foreign_leaves JSONB NOT NULL DEFAULT '[]'::jsonb,  -- 现役离队外援列表
    foreign_ins JSONB NOT NULL DEFAULT '[]'::jsonb,     -- 新签外援列表
    domestic_ins JSONB NOT NULL DEFAULT '[]'::jsonb,    -- 国内重点引援列表 (<=3)
    domestic_outs JSONB NOT NULL DEFAULT '[]'::jsonb,   -- 国内建议离队列表 (<=3)
    style_tag TEXT NOT NULL,
    manifesto TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_f_i_director_plans_window_team_user UNIQUE (window_id, team_id, user_id)
);

CREATE INDEX IF NOT EXISTS idx_f_i_director_plans_circle
    ON f_i_director_plans (window_id, team_id, circle_code, updated_at DESC);
CREATE INDEX IF NOT EXISTS idx_f_i_director_plans_team
    ON f_i_director_plans (window_id, team_id, updated_at DESC);

-- 3. 方案赞/踩（互斥，value IN (1, -1)）
CREATE TABLE IF NOT EXISTS f_i_director_plan_reactions (
    plan_id UUID NOT NULL REFERENCES f_i_director_plans(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES f_i_users(id) ON DELETE CASCADE,
    value SMALLINT NOT NULL CHECK (value IN (1, -1)),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (plan_id, user_id)
);

CREATE INDEX IF NOT EXISTS idx_f_i_director_plan_reactions_plan
    ON f_i_director_plan_reactions (plan_id, value);

-- 4. 方案一句话锐评
CREATE TABLE IF NOT EXISTS f_i_director_plan_comments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    plan_id UUID NOT NULL REFERENCES f_i_director_plans(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES f_i_users(id) ON DELETE CASCADE,
    content TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_f_i_director_plan_comments_plan
    ON f_i_director_plan_comments (plan_id, created_at DESC);
```

### 2.2 候选人 JSON 项结构（`PlanPlayerItem`，用于 `foreign_stays` / `foreign_leaves` / `foreign_ins` / `domestic_ins` / `domestic_outs`）

```json
{
  "key": "squad:12345",
  "source": "squad",
  "player_id": 12345,
  "candidate_code": null,
  "player_name": "费利佩",
  "position": "FW",
  "country": "巴西",
  "is_foreign": true,
  "league_tag": "CSL",
  "current_club": "成都蓉城",
  "avatar_url": "https://..."
}
```
- `source` 取值：`"squad"`（库内球员）| `"pool"`（精选候选池）| `"custom"`（自由提名）
- `key` 规则（用于跨方案重合度与共识聚合）：
  - `squad`: `"squad:{player_id}"`
  - `pool`: `"pool:{candidate_code}"`
  - `custom`: `"custom:{normalized_player_name}"`（去首尾空格并转小写）

### 2.3 种子迁移：`migrations/20261119130000_seed_director_candidates.sql`
- 使用 `INSERT INTO f_i_director_candidates (...) VALUES (...) ON CONFLICT (code) DO UPDATE SET ...` 保证幂等。
- 首批涵盖 >= 45 名真实/典型候选人：
  - K 联赛（`league_tag = 'K1'` / `'K2'`）外援与亚外 >= 20 名；
  - 其他海外/自由身/绯闻外援（`J1` / `BRA` / `EUR` / `FREE`）>= 15 名；
  - 国内热门引援（`is_foreign = false`, `league_tag = 'CSL'` / `'CL1'` / `'FREE'`）>= 10 名。

---

## 3. REST API 契约（前缀 `/api/v1/director`）

所有接口均加入 `src/http_cache.rs` 的 `matches_excluded_prefix("/api/v1/director/")` 并挂 `no_store` 响应头。

### 3.1 `GET /api/v1/director/context?team_id={team_id}`（公开 / 可选登录）
返回当前窗口、所选球队、现役外援与现役国内球员列表、当前用户已保存方案（如已登录且存在）、该球队公开方案总数。
```json
{
  "window": { "id": 1, "code": "2026-winter", "name": "2026 冬窗", "opens_at": "...", "closes_at": "..." },
  "team": { "team_id": 77680, "team_name": "成都蓉城", "logo_url": "..." },
  "foreign_quota": 5,
  "max_domestic_ins": 3,
  "max_domestic_outs": 3,
  "squad_foreign": [
    { "player_id": 1001, "player_name": "费利佩", "shirt_number": 21, "position": "FW", "country": "巴西", "avatar_url": "..." }
  ],
  "squad_domestic": [
    { "player_id": 1002, "player_name": "韦世豪", "shirt_number": 7, "position": "FW", "country": "中国", "avatar_url": "..." }
  ],
  "my_plan": null,
  "team_plan_count": 12
}
```

### 3.2 `GET /api/v1/director/candidates?team_id={team_id}&is_foreign={true|false}&league={all|K1|K2|J1|CSL|FREE|OTHER}&q={keyword}`（公开）
返回精选候选池 +（当 `q` 非空或 `league=CSL` 时）其他中超球队的现役球员搜索结果（自动排除当前 `team_id` 已有现役球员）。
```json
{
  "items": [
    {
      "key": "pool:k1-cesinha",
      "source": "pool",
      "player_id": null,
      "candidate_code": "k1-cesinha",
      "player_name": "塞西尼亚",
      "position": "FW",
      "country": "巴西",
      "is_foreign": true,
      "league_tag": "K1",
      "current_club": "大邱FC",
      "tags": ["K联赛核心", "传射兼备"],
      "avatar_url": null
    }
  ]
}
```

### 3.3 `PUT /api/v1/director/plans/me`（必须登录）
创建或更新当前用户在「当前窗口 + `team_id`」的建队方案。
请求体：
```json
{
  "team_id": 77680,
  "circle_code": "AB12CD34",
  "parent_share_code": "AB12CD34",
  "foreign_stay_ids": [1001, 1002],
  "foreign_leave_ids": [1003],
  "foreign_ins": [
    {
      "source": "pool",
      "candidate_code": "k1-cesinha",
      "player_id": null,
      "player_name": null,
      "position": null,
      "league_tag": null,
      "current_club": null
    },
    {
      "source": "custom",
      "candidate_code": null,
      "player_id": null,
      "player_name": "金某某",
      "position": "MF",
      "league_tag": "K1",
      "current_club": "蔚山HD"
    }
  ],
  "domestic_ins": [],
  "domestic_outs": [1009],
  "manifesto": "把预算砸在中场和边路，冲击亚冠！"
}
```
后端校验：
- 校验 `foreign_stay_ids` 与 `foreign_leave_ids` 均属于该队现役外援且无交集；
- 校验 `foreign_stay_ids.len() + foreign_ins.len() <= 5` 且 `>= 1`；
- 校验 `domestic_ins.len() <= 3`、`domestic_outs.len() <= 3`；
- 对 `manifesto` 以及 `custom` 球员的 `player_name`/`current_club` 进行长度校验 + `ContentSecurityPort::check_text` 安全检测；
- 返回完整 `DirectorPlanDetailDto`（与 3.4 同结构）。

### 3.4 `GET /api/v1/director/plans/{share_code}`（公开 / 可选登录）
返回方案完整详情，包含作者昵称头像、外援/国内名单、风格标签、宣言、赞/踩数与当前用户态度、最新锐评列表（最多 30 条）、以及（若当前已登录且在同窗口同球队也有方案）**自动附带 `my_comparison` 1v1 分歧报告**。
```json
{
  "plan": {
    "id": "uuid",
    "window_code": "2026-winter",
    "window_name": "2026 冬窗",
    "team_id": 77680,
    "team_name": "成都蓉城",
    "share_code": "AB12CD34",
    "circle_code": "AB12CD34",
    "author": { "user_id": "uuid", "nickname": "老王", "avatar_url": "..." },
    "is_mine": false,
    "foreign_stays": [],
    "foreign_leaves": [],
    "foreign_ins": [],
    "domestic_ins": [],
    "domestic_outs": [],
    "style_tag": "K联赛淘宝流",
    "manifesto": "...",
    "likes_count": 5,
    "dislikes_count": 2,
    "my_reaction": 1,
    "comments_count": 3,
    "updated_at": "..."
  },
  "comments": [
    {
      "id": "uuid",
      "author": { "user_id": "uuid", "nickname": "小李", "avatar_url": "..." },
      "content": "送走费利佩我第一个不服！",
      "created_at": "..."
    }
  ],
  "comparison": {
    "my_share_code": "EF56GH78",
    "overlap_percent": 60,
    "common_foreign": [],
    "only_in_target": [],
    "only_in_mine": [],
    "leave_disagreements": [
      { "player": {}, "target_action": "leave", "my_action": "stay" }
    ],
    "summary_line": "你们在 3 个外援名额上达成一致，最大分歧在费利佩的去留"
  }
}
```

### 3.5 `GET /api/v1/director/circles/{circle_code}`（公开 / 可选登录）
返回该好友圈（同一 `circle_code`）的聚合共识：
```json
{
  "circle_code": "AB12CD34",
  "window_name": "2026 冬窗",
  "team_id": 77680,
  "team_name": "成都蓉城",
  "member_count": 4,
  "unlocked": true,
  "top_foreign": [
    { "player": {}, "pick_count": 4, "pick_rate": 100 }
  ],
  "top_leaves": [
    { "player": {}, "leave_count": 3, "leave_rate": 75 }
  ],
  "plans": [
    {
      "share_code": "AB12CD34",
      "author": { "user_id": "uuid", "nickname": "老王", "avatar_url": "..." },
      "style_tag": "K联赛淘宝流",
      "manifesto": "...",
      "foreign_count": 5,
      "likes_count": 5,
      "dislikes_count": 2,
      "updated_at": "..."
    }
  ]
}
```
- `unlocked`: `member_count >= 2`。

### 3.6 `PUT /api/v1/director/plans/{share_code}/reaction`（必须登录）
请求体：`{ "value": 1 | -1 | 0 }`（`1` 认同，`-1` 不服，`0` 取消）。返回 `{ "likes_count", "dislikes_count", "my_reaction" }`。

### 3.7 `POST /api/v1/director/plans/{share_code}/comments`（必须登录）
请求体：`{ "content": "..." }`（1..=50 字，走 `ContentSecurityPort::check_text`）。返回新创建的 `DirectorCommentDto`。

---

## 4. 小程序端架构与页面设计（`football_insight_mini`）

1. **分包注册**（`src/pages.json`）：
   - `pages/director/index`：「我来当总监」选人编辑页（接收可选 query：`team_id`, `circle_code`, `from_code`）
   - `pages/director/view`：方案详情 + 1v1 分歧报告 + 好友圈共识 + 赞踩/锐评页（接收 query：`code`）
   - 在 `preloadRule` 中为 `pages/home/index` 和 `pages/insights/index` 预加载 `pages/director`。
2. **入口挂载**：
   - 首页 `HomeFanEntries`（`src/pages/home/`）：增加「我来当总监」入口卡片/胶囊。
   - 洞察页 `InsightsSecondaryEntries`（`src/pages/insights/`）：增加「我来当总监」入口卡片。
3. **严格遵守小程序架构与审计规范**：
   - 单 `.vue` 文件严格 <= 600 行，复杂区块拆为 `pages/director/components/*.vue`；
   - `components/` 不 import `src/api/**` 或认证存储，不调用 `uni.showModal`；
   - 样式全部使用 semantic/component CSS token（禁止 `--fi-primitive-*`、禁止写死 `font-size`/`gap`/`padding`/`border-radius` 数字、禁止 `[disabled]` 选择器、禁止 DOM/Web Storage API）；
   - 分享路径统一通过 `buildSharePath(targetPath, 'director')` 包装；
   - 异步请求使用 `createLatestGuard` 防竞态，未登录写操作走 `rememberPostLoginRedirect` 回跳。

# 分享路径来源参数统一设计

日期：2026-10-19
状态：代码及自动化验证完成；微信真机验收待执行（设计由 Claude 与产品负责人讨论后撰写）

## 背景与目的

用户来源追踪（`2026-10-18-acquisition-source-tracking-design.md`）已上线：小程序启动时从
`query` 或 `scene` 解析 `src` / `cid`，注册时写入 `f_i_user_acquisitions`，只记首次来源。

但当前各页面的 `onShareAppMessage` 返回的是普通页面路径（例如
`/pages/ticket-watch/index`），**不带任何来源参数**。用户转发给朋友后，朋友注册时没有来源，
无法回答：

- 转发带来了多少新用户？
- 哪个页面的转发最有效（回流看板、赛程、话题、赛季海报……）？

这是后续做「回流看板分享图」「成功换座图」等分享功能的前置条件：不先统一参数，
后面所有分享优化都无法度量。

**这是一个小改动，目标是半天内做完。** 不增加任何用户可见的界面。

## 目标

1. 所有分享出口（对话分享、朋友圈分享）的链接统一携带 `src=share&cid=<页面标识>`。
2. 统一用一个工具函数构造，避免各页面各写各的。
3. 可以用现有的来源汇总 SQL，按 `cid` 看出每个分享点带来的新注册。

## 非目标

- 不统计「转发了多少次」「点击了多少次」。微信 `onShareAppMessage` 在菜单打开时即触发，
  不代表真的发送成功；点击打开次数也无从得知。本设计只度量**转发带来的新注册**。
- 不在链接中加入分享者身份（用户 ID、昵称、邀请码）。不做分享者排行，不做裂变奖励。
- 不改变来源追踪的「只记首次来源」规则。已经注册的老用户点开朋友转发的链接，不会产生新记录。
- 不改变各页面现有的分享标题、封面图、目标页面。
- 不新增朋友圈分享入口之外的页面（见下方「朋友圈」）。

## 反目标

- 来源参数导致页面打开失败，或原有 query（如话题详情的 `id`）丢失。
- 把 token、手机号、用户 ID 等放进分享链接。
- 已有的 `src` / `cid`（例如赛季海报的 `recap2026`）被覆盖。

## 方案

### 1. 参数约定

`src` 固定为 `share`。`cid` 表示分享点，约定如下（只允许 `[a-z0-9_-]`，≤64，沿用来源追踪规则）：

| 页面 | `cid` |
|---|---|
| 首页 `pages/home/index` | `home` |
| 赛程 `pages/matches/index` | `matches` |
| 排行 `pages/rankings/index` | `rankings` |
| 洞察 `pages/insights/index` | `insights` |
| 球队看板 `pages/team-board/index` | `team_board` |
| 助威 `pages/support/index` | `support` |
| 话题列表 `pages/topics/index` | `topics` |
| 话题详情 `pages/topic-detail/index` | `topic_detail` |
| 回流看板·当前比赛 `pages/ticket-watch/index` | `ticket_current` |
| 回流看板·历史复盘 `pages/ticket-watch/index` | `ticket_history` |
| 换座位 `pages/seat-swap/index` | `swap` |
| 赛季海报 `pages/season-recap/index` | 保持现状 `recap<赛季>` |

换座位页目前**没有**分享处理函数，需要新增一个，标题和路径不含任何个人信息或座位信息。

### 2. 工具函数

在 `football_insight_mini/src/utils/` 新增 `shareSource.ts`（实现方可调整文件名）：

```ts
buildSharePath(path: string, cid: string): string
buildShareQuery(cid: string): string   // 朋友圈用，返回 "src=share&cid=xxx"
```

要求：

- `path` 可能已经带 query（`/pages/topic-detail/index?id=topic_2026-10-13`）。使用 `?` 或 `&`
  正确拼接，**保留原有参数**。
- 如果 `path` 已经含有 `src` 或 `cid`，**不覆盖**，直接返回原路径。
- `cid` 不符合约定（空、超长、含非法字符）时，返回不带来源参数的原路径，不抛错。
  分享不能因为埋点参数而失败。
- 纯函数，不依赖 `uni`，便于单元测试。

### 3. 各页面改造

把每个页面 `onShareAppMessage` 返回的 `path` 用 `buildSharePath` 包一层，其他字段不动。
话题详情的 `buildTopicSharePayload` 在 `src/pages/topic-detail/share.ts`，
现有测试 `share.test.ts` 断言了精确路径，需要同步更新断言（路径追加来源参数）。

回流看板 `ticket-watch` 目前有三个分支（历史复盘、当前比赛、兜底），历史复盘用
`ticket_history`，其余用 `ticket_current`。

### 4. 朋友圈（`onShareTimeline`）

微信要求页面定义 `onShareTimeline` 才会出现「分享到朋友圈」。目前只有赛季海报定义了。

首版只给**首页、回流看板、话题详情**加朋友圈分享（这三处是内容型页面，适合发朋友圈），
其余不加。朋友圈只能传 `query`，不能指定 `path`，用 `buildShareQuery`；话题详情需要
同时带 `id`。

朋友圈单页模式是只读的，用户点击「前往小程序」后才进入完整版，来源参数在进入完整版时
才会被 `getLaunchOptionsSync` / `onShow` 读到。这是微信行为，实现方验证时注意，
不能保证所有场景都带上参数，不要为此写额外兼容逻辑。

### 5. 度量

不新增表。`docs/sql/user-acquisition-summary.sql` 已按 `src`、`cid` 分组，
`WHERE src = 'share'` 即可看到各分享点的注册、回访和付费。实现方在文档
`docs/acquisition-source-tracking-validation.md` 末尾补一小节，写明如何查看分享来源。

## 边界与已知限制（写进验证报告）

- 已注册的老用户转发给也已注册的朋友：不产生记录，这是预期。
- 未注册但曾打开过小程序、没有来源的人，之后通过朋友的转发打开：`App.vue` 在
  `onLaunch` 和 `onShow(options)` 都会调用 `captureAcquisitionSource`，热启动也会被捕获，
  实现方请在真机验证一次。
- 度量对象是「新用户」。转发的整体传播效果会被低估，这是有意接受的取舍。

## 验收标准

1. `buildSharePath` 单元测试：无 query、已有 query、已含 `src`/`cid`、非法 `cid`、
   空路径、带 hash 的路径。
2. 各页面分享返回值的单元测试或断言：路径含正确的 `cid`，话题详情的 `id` 保留。
3. 换座位页新增分享处理函数，且路径中不含任何用户数据。
4. 用 `parseAcquisitionLaunch` 对生成的路径解析，得到预期的 `src` / `cid`
   （写一个把两者串起来的测试）。
5. 微信开发者工具或真机：从分享卡片打开，`uni.getLaunchOptionsSync().query` 中能看到参数。
6. 通过：`bun run type-check`、`bun test`、`bun run check:boundaries`、
   `bun run verify:mp-components`。

## 交给实现方决定

- 工具函数的文件名和导出细节。
- 朋友圈分享的标题文案（沿用各页面现有标题）。

## 提示

- 页面之间不能互相 import，工具放 `src/utils`。
- 小程序版本号由发版脚本分配，不要手改。
- 这是纯前端改动，不需要后端或迁移。上线只需发一次小程序版本。

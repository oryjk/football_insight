# 公众号自助绑定设计

日期：2026-10-16
状态：用户已确认方案，待交付实现
范围：`football_insight_service_backend_rs`（auth 模块，含一个新迁移）、`football_insight_mini`（「我的」页）
不涉及：`union_id` 打通、`football_insight_admin_android`、其他页面

## 1. 背景

公众号「下一场」比赛 ID 已按会员门控（见 `2026-10-16-wechat-next-match-id-gate-design.md`，已部署）。门控靠**公众号 openid** 找到对应账号，而库里 `f_i_users.official_wechat_open_id` 只有一个来源：用户关注公众号后拿到邀请码，带着邀请码注册时写入。

实测问题（2026-10-16）：

- 913 个注册用户里只有 484 个有公众号 openid；39 个有效 V6+ 会员中 7 个没有。
- 测试账号 `97f5ca05…`（V9，active）回复「下一场」收到「暂未识别到你的会员账号」：它是通过小程序微信登录创建的，只有小程序 openid，没有公众号 openid。日志判定为 `decision=unrecognized`，行为符合设计，但真实会员也会遇到。
- 全部 913 个用户的 `union_id` 为空，所以 unionid 打通现在走不通。
- `f_i_users.has_wechat_binding` 的含义是**小程序 openid 已绑定**（`wx_open_id IS NOT NULL`），与公众号无关，**不能复用**。

## 2. 目标

已有账号**自己补一次**公众号绑定，之后在公众号里回复「下一场」即可被识别。

**为什么一次就够**：openid 对同一个用户、同一个公众号是固定的，取关再关注也不变，所以绑定一次长期有效。只有用户换了微信号，或公众号换了 AppID，才需要重新绑定。

## 3. 流程

```
小程序「我的」页（已登录、未绑定公众号）
  └ 点「绑定公众号」→ 打开弹层 → 点「获取绑定码」→ 显示 6 位绑定码 + 倒计时 + 复制按钮
用户在公众号里发送：绑定 ABC234
  └ 后端校验绑定码 → 把该公众号 openid 写到绑定码对应账号 → 回复绑定成功
之后在公众号里回复「下一场」→ 门控能识别到这个账号
```

**双向证明**：绑定码只有登录了小程序账号的人才拿得到（证明控制该账号），发送消息的公众号 openid 由微信保证（证明控制该微信号）。

## 4. 后端设计

遵循 `domain / ports / application / adapters`，规则放 domain，SQL 放 adapter，不要写进 handler。

### 4.1 迁移（新增一张表，不改现有表）

文件 `YYYYMMDDHHMMSS_add_wechat_official_bind_codes.sql`，时间戳必须大于现有最新迁移（以 `ls migrations | tail -1` 为准）：

```sql
CREATE TABLE f_i_wechat_official_bind_codes (
    id BIGSERIAL PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES f_i_users(id) ON DELETE CASCADE,
    code_hash TEXT NOT NULL,                 -- 绑定码的 SHA-256 十六进制，不存明文
    expires_at TIMESTAMPTZ NOT NULL,
    used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE UNIQUE INDEX uq_wechat_official_bind_codes_hash ON f_i_wechat_official_bind_codes (code_hash);
CREATE INDEX idx_wechat_official_bind_codes_user ON f_i_wechat_official_bind_codes (user_id, created_at DESC);
```

### 4.2 绑定码规则（domain，纯函数 + 单测）

- 6 位，字符集 `ABCDEFGHJKLMNPQRSTUVWXYZ23456789`（去掉易混的 `I O 0 1`），大小写不敏感，**统一转大写后再哈希比较**。
- 有效期 **10 分钟**，**一次性**。
- 同一用户重新获取时，先使该用户所有未使用且未过期的旧码失效（置 `used_at`），再发新码。
- 频率限制：同一用户 1 小时内最多获取 **5** 次；超限返回 429。
- 随机数用密码学安全的随机源。
- 哈希唯一冲突时重新生成，最多重试几次。

### 4.3 获取绑定码接口

`POST /api/v1/auth/wechat-official-bind-code`（需要 JWT，`active` 用户）：

```json
{ "code": "ABC234", "expires_in_seconds": 600 }
```

响应永远不缓存（`Cache-Control: no-store`）。已绑定公众号的账号也允许获取（用于换微信号重新绑定），不报错。

### 4.4 当前用户接口新增只读字段

在返回当前用户的接口（`GET /api/v1/auth/me` 对应的 DTO）里**新增可选字段** `official_wechat_bound: bool`（`official_wechat_open_id IS NOT NULL`）。**只增字段，不改任何现有字段**；旧版小程序忽略它。

### 4.5 公众号指令

在 `handle_wechat_webhook.rs` 现有命令归一化之后（已去掉所有空白并转小写）识别以 `绑定` 开头的文本：`绑定ABC234`、`绑定 ABC234`、`绑定  abc234` 都有效。

| 输入 | 处理 |
|---|---|
| 仅 `绑定`（没有码） | 回复格式提示 |
| `绑定` + 有效且未用的码 | 事务内完成：原子消耗绑定码（`UPDATE … SET used_at=NOW() WHERE code_hash=$1 AND used_at IS NULL AND expires_at>NOW() RETURNING user_id`），再 `UPDATE f_i_users SET official_wechat_open_id=$openid WHERE id=$user_id AND status='active'`；成功回复绑定成功 |
| 码无效、过期、已用、用户非 active | **统一**回复"无效或已过期"，**不得区分原因**（防止探测） |
| 该账号已绑定到同一个 openid | 绑定码照常消耗，回复"已绑定过，无需重复" |
| 该账号绑定的是另一个 openid | **覆盖**为当前 openid（用户换了微信号），回复绑定成功 |
| 同一公众号 openid 连续输错 | 进程内计数：同一 openid 10 分钟内错误超过 5 次，回复"操作过于频繁"，期间不再校验 |

同一个 openid 允许绑定到多个账号（库里已有 27 个这样的共用 openid，门控已按"任一账号满足即授权"处理）。

### 4.6 回复文案（定稿，逐字使用；末尾保留现有 `with_product_intro`）

**绑定成功**：

```
【足球洞察】绑定成功

已将本公众号与你的小程序账号关联。请保持关注本公众号，取关后会员权益会暂停。

现在可以回复“下一场”查询比赛 ID（V6 及以上会员可用）。
```

**绑定码无效或已过期**：

```
【足球洞察】绑定失败

绑定码无效或已过期。请回到小程序「我的」页，点「绑定公众号」重新获取绑定码，再回复“绑定 + 绑定码”。
```

**已绑定过**：

```
【足球洞察】已绑定

你的账号已经绑定过本公众号，无需重复操作。可以直接回复“下一场”查询比赛 ID（V6 及以上会员可用）。
```

**缺少绑定码**：

```
【足球洞察】绑定公众号

请先打开小程序「足球洞察」，在「我的」页点「绑定公众号」获取绑定码，然后回复“绑定 + 绑定码”，例如：绑定 ABC234
```

**操作过于频繁**：

```
【足球洞察】操作过于频繁，请稍后再试。
```

### 4.7 修改既有「未识别」文案（用户要求）

现有 `next_match_unrecognized_message` 改为：

```
【足球洞察】当前比赛信息

下一场比赛 ID 是会员权益，V6 及以上会员可直接查询。

暂未识别到你的会员账号。如果你已经是会员，请打开小程序「足球洞察」，在「我的」页点「绑定公众号」获取绑定码，再回复“绑定 + 绑定码”（例如：绑定 ABC234），只需绑定一次，之后就可以直接在这里回复“下一场”查询。也可以直接在小程序「余票监控」页点「获取比赛 ID」查看。

还不是会员的话，在小程序「我的」页点会员卡上的「升级」即可开通。

常用指令：
- 回复“邀请码”或“1”获取注册邀请码
```

原有的测试里对"未识别文案"的断言要同步更新，并保持"不含比赛 ID"的断言。

### 4.8 已知的副作用（必须保留在文案和 PR 说明里）

绑定后，`membership_benefits_enabled` 会按这个公众号 openid 的**关注状态**计算：**取关后该账号的小程序会员权益暂停**（现有规则，目前只对注册时带邀请码的用户生效）。所以成功文案里提示"请保持关注"。实现者需要全仓搜索 `official_wechat_open_id` 的其他使用点，确认没有别的副作用，并写进最终说明。

### 4.9 日志与隐私

- 每次绑定指令输出一行结构化判定：`wechat_bind decision=success|already_bound|invalid|rate_limited|missing_code|error`。
- **日志里不得出现绑定码、openid 原文、user_id**。
- 失败路径 fail closed：任何错误都不写入绑定。

## 5. 小程序设计（「我的」页）

遵守 `football_insight_mini/AGENTS.md`：页面不互引、组件层不 import `api/**`、HTTP 只走 `utils/request.ts`、不新增 ALLOWLIST 例外；样式全部引用 token，**不新增渐变**；不手改 `versionName`。

- `src/types/auth.ts`：`CurrentUser` 增加可选 `official_wechat_bound?: boolean`。
- `src/api/auth.ts`：新增 `createWechatOfficialBindCode(): Promise<{ code: string; expires_in_seconds: number }>`。
- 「我的」页（`pages/user/`）：在「账户信息」区域的合适位置新增一个入口行"绑定公众号"，**仅在已登录、非审核态、`official_wechat_bound !== true` 时显示**，旧后端没有该字段时按未绑定处理（显示入口，不隐藏）。样式沿用现有行式（图标 40rpx，参考账户信息行）。
- 新增弹层组件 `UserOfficialBindSheet.vue`（参考同目录 `UserNotificationEmailSheet.vue` 与 `FiBottomSheet`）：
  - 初始：说明文字（"绑定一次，之后可在公众号里查询下一场比赛 ID"）+「获取绑定码」按钮。
  - 获取成功：显示 6 位绑定码（大字、等宽）、剩余时间倒计时、「复制」按钮、操作步骤（"打开公众号，回复：绑定 ABC234"），以及一行提示"绑定后请保持关注公众号，取关后会员权益会暂停"。
  - 过期后：提示已过期，按钮变为「重新获取」。
  - 获取失败：429 提示"获取过于频繁，请稍后再试"，其他错误用通用错误文案；错误判断用 `ApiRequestError.statusCode`，**不得按文案猜**。
- 倒计时、绑定码格式化等纯逻辑放 `pages/user/helpers.ts`，必须有单测。
- 获取绑定码后，用户去公众号绑定、再回到小程序，需要刷新当前用户才能让入口消失：`onShow` 已有 `loadUser()`，沿用即可，不新增轮询。

## 6. 向后兼容（硬性）

- 迁移只新增一张表和两个索引。
- 现有接口路径、请求体、已有响应字段、状态码、错误文案不变；`official_wechat_bound` 是新增可选字段。
- 旧版小程序没有绑定入口，行为完全不变；公众号原有指令（邀请码、下一场、你好、订阅事件）行为不变，除 4.7 的未识别文案。
- 门控的 fail-closed 原则不变：任何路径不得因为本改动而在非授权时回复比赛 ID。

## 7. 测试要求

**后端**：
- domain：绑定码字符集与长度、无易混字符、归一化（大小写、空白）、有效期边界、频率限制；
- 指令解析：`绑定ABC234`、`绑定 abc234`、`绑定`、`绑定  ABC234` 等；
- webhook 用例（沿用该文件的 fake）：成功、无效、过期、已使用、已绑定同 openid、覆盖另一个 openid、缺少码、频率限制、非 active 用户；
- 失败回复文案**一致**（不泄露原因），日志中无绑定码与 openid；
- 「未识别」文案新断言，且仍不含比赛 ID；门控原有测试全部通过；
- 获取绑定码接口：需要 JWT（无 token 401）、响应头 `no-store`、1 小时第 6 次返回 429、重新获取使旧码失效。

**SQL 验证**（只有生产这一个数据库）：新增 SQL 必须在生产库的**回滚事务**里真实验证：`BEGIN` → `SET LOCAL lock_timeout='2s'` → 执行迁移 → 用**已存在用户**的 ID 插入模拟绑定码 → 跑原子消耗、重复消耗、过期消耗、覆盖 openid 的语句（**覆盖 openid 只能作用在回滚事务里的这条测试数据上，不得改真实用户**）→ `ROLLBACK`，并确认回滚后表不存在。脚本放会话 scratchpad，用 `uv run python` 的 psycopg，从 `.env` 读 `DATABASE_URL`，不输出连接串。

**小程序**：`bun test`、`bun run type-check`、`bun run check:boundaries`、`bun run verify:mp-components` 全部通过；新增 helpers 的单测。

## 8. 硬性约束

1. 只有生产一个数据库，本地后端连的就是生产 `football_data`：严禁修改真实用户、订单、会员数据；严禁往公众号发消息；严禁发起真实支付。
2. 迁移不自行对生产执行，写好后停下；**不要部署生产、不要上传小程序**，由审查后统一执行。
3. 绑定写入只能发生在"有效绑定码 + 微信保证的 openid"两者都满足时；任何校验失败或出错都不写入。
4. 后端改动后本地验证前先 `./deploy/local-test/up.sh` 重启本地后端；汇报时分开写：本地后端已重启 / 生产未部署 / 生产已部署并重启。
5. 提交风格：`feat(wechat): ...`、`feat(mini): ...`，subject 用英文，末尾加 Co-Authored-By 行；后端与小程序分开提交；直接提交并推送 `main`，不新建分支；同步更新 `CHANGELOG.md`、`FEATURES.md` 并写明原因。

## 9. 范围外

- `union_id` 打通（需要先在小程序登录采集 unionid，且取决于公众号和小程序是否在同一开放平台）；
- 管理后台查看或解绑；
- 对已绑定用户的批量迁移或回填；
- 绑定成功后的主动推送。

## 10. 发布与时机

顺序：迁移 → 后端部署 → 小程序新版本。目标 **2026-10-21 前**全部上线，赶在 10 月 22 日 14:00 开售前，让会员在第一个开票窗口就能自助补绑。

## 11. 审查清单（Review 逐条核对）

1. 绑定码：6 位、无易混字符、大小写不敏感、10 分钟有效、一次性、存哈希不存明文；重新获取使旧码失效；1 小时 5 次限制。
2. 原子消耗：消耗绑定码与写入 openid 在同一事务；并发或重复使用只会成功一次。
3. 所有失败原因回复**同一句**"无效或已过期"，没有可区分的差异；同一 openid 连续输错有限制。
4. 日志无绑定码、无 openid 原文、无 user_id；有结构化 `wechat_bind decision=` 判定。
5. 「未识别」文案与 4.7 逐字一致，仍不含比赛 ID；门控原有测试全部通过；其他文案与 4.6 逐字一致。
6. 迁移只新增表和索引；时间戳足够大；未自行对生产执行；SQL 已在回滚事务中真实验证，覆盖 openid 的验证没有触碰真实用户。
7. `official_wechat_bound` 是新增可选字段，没有改现有字段；没有复用 `has_wechat_binding`。
8. 小程序入口只在"已登录、非审核态、未绑定"时显示；旧后端没有该字段时仍显示入口；错误判断用 `statusCode`；无渐变；无 ALLOWLIST 新增；组件层没有 import `api/**`。
9. 弹层有倒计时、复制、过期后重新获取，并有"取关后会员权益会暂停"提示。
10. 全仓搜索了 `official_wechat_open_id` 的其他使用点，并在最终说明里写出副作用结论。

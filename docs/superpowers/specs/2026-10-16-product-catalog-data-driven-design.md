# 商品目录数据驱动化重构设计（含 V4/V5 日卡）

日期：2026-10-16
状态：待用户确认后交付实现
范围：`football_insight_service_backend_rs`（payment、match_id_unlock、reflux_subscription、admin）、`football_insight_mini`（购买页）、`football_insight_admin_web`（商品管理）
前置：`2026-10-16-payment-observability-design.md` 已实现并合并（`9c81a3e`、`5730bbf`），本设计在其之上进行
不涉及：安卓管理端、定价策略本身

## 1. 为什么要重构

目标原则：**新增一个商品（SKU）只往数据库加一行，或在后台页面点一下，不改任何业务代码。只有新增一种"商品类型"（有全新的发货行为）才允许写代码，且只写一个发货处理器。**

现在做不到。商品的定义散落在多处，且彼此重复、互相不一致：

| 维度 | 现状（均已读代码核实） |
|---|---|
| 档位白名单 | 写死在 4 处：DB CHECK `virtual_product_key_matches_price`、`product_key()`、`order.rs:227`（未知档位还会**静默回落成 V9**）、`get_membership_product.rs` 私有商品分支；后台 `admin_product.rs::validate_tier` 是第 5 处 |
| 价格来源 | 会员价在 `system_config.membership_product` JSON 和商品表各一份（文档要求"三处同步"）；比赛 ID 解锁是代码常量 `MATCH_ID_UNLOCK_PRICE_CENTS=500`；回流订阅价在 `f_i_reflux_subscription_plans`。三套机制 |
| 有效期 | 结算 SQL `make_interval(months => n)`，只能按月 |
| 付费判定 | `resolve_membership_settlement` 用 `rank >= V6` 判断是否付费会员——**V4/V5 的付费订单会被当成非付费，用户付款后不升级** |
| 商品类型分发 | 订单靠解析 `product_type` 字符串前缀分发（`handle_wechat_notify.rs:94-110`），新增类型要改解析函数 |
| 目录形态 | 公开目录每个档位只有一项，由 JSON 生成；没有"排序/展示文案/受众/客户端能力"的数据字段 |
| 运营入口 | 后台商品管理只支持会员，且同样有档位白名单；比赛 ID 解锁和回流订阅**在商品表里没有映射，当前实际买不了** |

由此带来的现实问题：运营上架 `fi_v6_day_9`、`fi_v9_day_19` 两个日卡，需要改至少 6 处代码。

## 2. 目标模型

### 2.1 一个概念：SKU = 数据；商品类型（kind）= 代码

- **SKU**：商品表里的一行，包含价格、展示、受众、发货参数，全部是数据。
- **kind**：一类发货行为，代码里注册的处理器。当前三种：`membership`（升级会员档位）、`match_id_unlock`（解锁单场比赛 ID）、`reflux_subscription`（回流提醒订阅）。
- 同 kind 的任意新 SKU（新档位、新价格、新时长规则、新套餐）零代码。

### 2.2 商品表扩展（沿用 `f_i_virtual_payment_products`，不新建表）

保留全部现有列和 `id` 目录身份，新增：

| 列 | 类型 | 含义 |
|---|---|---|
| `kind` | TEXT NOT NULL | `membership` / `match_id_unlock` / `reflux_subscription`。迁移时由 `product_key` 前缀回填，之后以此为准 |
| `grant_spec` | JSONB NOT NULL DEFAULT '{}' | kind 专属的发货参数，由对应 kind 处理器校验（见 2.3） |
| `pricing_rule` | JSONB NOT NULL DEFAULT '{"type":"fixed"}' | 价格规则：`fixed`，或 `upgrade_credit`（补差价，参数 `fee_cents`、`credit_from_tiers`）。**取代写死的 `V6\|V7\|V8` 与 ¥15 服务费** |
| `eligibility` | JSONB NOT NULL DEFAULT '{}' | 受众条件，P1 支持字段 `max_current_tier`（用户当前生效等级不得高于它）、`min_current_tier`；空对象表示不限。字段集合由 kind 处理器定义并校验，未知字段拒绝保存 |
| `requires_capability` | TEXT NULL | 客户端能力标识，如 `catalog-v2`。请求没有声明该能力时该 SKU 不可见、不可买 |
| `title` / `subtitle` / `description` | TEXT | 展示文案（`name` 保留为后台内部名） |
| `sort_order` | INT NOT NULL DEFAULT 0 | 展示排序 |
| `group_key` | TEXT NULL | 展示分组，如 `pass`（场次体验卡）、`annual`（年度会员），小程序据此分段 |

`membership_duration_months` 保留，仅作为 `expiry.rule=months` 时的值；其余 kind 忽略。

### 2.3 grant_spec（各 kind 的发货参数）

```jsonc
// membership
{ "tier": "V4",
  "expiry": { "rule": "months", "months": 12 } }
// 或 { "tier": "V4", "expiry": { "rule": "end_of_next_day", "tz": "Asia/Shanghai" } }

// match_id_unlock
{}                                   // 具体比赛由订单绑定

// reflux_subscription
{ "plan_code": "season_2026", "scope": "season", "team_code": "chengdurongcheng", "season": 2026 }
```

到期规则 `expiry.rule` 是封闭枚举，由 membership 处理器实现：`months`、`end_of_next_day`。新增一种到期规则属于"新增行为"，允许写代码；但**新增同规则的 SKU 不需要**。

### 2.4 kind 注册（代码，唯一允许写代码的扩展点）

`payment/domain/product_kind.rs` 定义 trait，每个 kind 一个实现：

```rust
pub trait ProductKind: Send + Sync {
    fn name(&self) -> &'static str;
    /// 校验 grant_spec 与行字段，后台保存和迁移回填时调用；非法即拒绝，绝不静默回落
    fn validate(&self, spec: &ProductSpec) -> Result<(), ProductSpecError>;
    /// 受众判断（用户当前等级/来源等），目录展示与下单各调用一次
    fn eligible(&self, spec: &ProductSpec, user: &UserContext) -> bool;
    /// 实付价格；membership 在此实现 fixed 与 upgrade_credit
    fn resolve_price(&self, spec: &ProductSpec, user: &UserContext) -> Result<i32, ProductSpecError>;
    /// 发货。读订单上的 grant 快照，不读商品表
    async fn fulfill(&self, tx: &mut Tx, order: &OrderSnapshot) -> anyhow::Result<()>;
}
```

`ProductKindRegistry` 在 `bootstrap.rs` 装配。未知 `kind` 的行在目录里不展示并记 warn，不 panic。

**去掉的东西**：所有档位枚举白名单（改为单个 `is_valid_membership_tier()`：`V` + 数字，3 到 9，放 `domain`，所有位置引用同一处）；`MATCH_ID_UNLOCK_PRICE_CENTS` 常量；`order.rs:227` 的静默回落 V9（未知档位应报错）；补差价里写死的 `"V6"|"V7"|"V8"` 与 `1500`。

## 3. 订单与发货

### 3.1 下单

统一用例 `CreateProductOrderUseCase(sku, user, context)`：校验 SKU 启用、受众资格、客户端能力 → `resolve_price` → 按"kind 参数 + 实付价"解析出商品表里对应的微信道具行（微信虚拟支付是固定价道具，同一档位的补差价会对应不同行，这一点保持现状）→ 创建订单。现有三个下单入口（会员、比赛 ID、回流订阅）改为委托这个用例，**路径、请求体、响应体不变**。

### 3.2 订单快照

`f_i_virtual_payment_orders` 新增 `grant_snapshot JSONB`（含 `kind`、`grant_spec`、`catalog_product_id`）。下单时写入，**发货只读快照**，所以运营之后修改或停用商品不影响在途订单。

### 3.3 发货分发

`settle_verified_order`：订单有 `grant_snapshot` 则按 `kind` 走注册的处理器；**没有快照的订单（历史订单、重构前已创建尚未支付的在途订单）走现有的 `product_type` 字符串解析旧路径，行为逐字不变**。两条路径并存到旧订单自然清空（至少 30 天，对账窗口为 30 天）。

### 3.4 membership 发货修复（必须）

- 付费判定改为"该订单带时长/到期规则快照"，不再依赖 `rank >= V6`。V6–V9 年卡的结算结果必须与改动前逐例一致。
- 到期计算按 `grant_spec.expiry`：
  - `months`：`NOW() + make_interval(months => n)`，与现在相同。
  - `end_of_next_day`：购买次日 23:59:59 北京时间，即 `(date_trunc('day', NOW() AT TIME ZONE 'Asia/Shanghai') + INTERVAL '2 days' - INTERVAL '1 second') AT TIME ZONE 'Asia/Shanghai'`。
- 日卡永不缩短已有的到期时间，也不降级（现有 `should_apply_paid_tier` 的等级比较已覆盖，实现者补测）。

### 3.5 与现有机制的交互（实现者必查并补测）

- `resolve_effective_membership_tier`（`auth/domain/user.rs:85`）：任何到期后统一回落 **V3**。含义：邀请获得的 V4+ 用户若被写入日卡到期时间，到期后会掉回 V3，**损失已赚得的邀请等级**。因此日卡的 `eligibility` 设为 `{"max_current_tier":"V3"}`：当前生效等级高于 V3 的用户（含邀请得来的 V4+、持有日卡或年卡的用户）一律不展示、不可买。这同时保证日卡不会降级或缩短任何人的会员。下单时服务端再校验一次，目录隐藏不等于安全。
- V1/V2 用户买过日卡后，过期会停在 V3（比原来更快）。本期接受并在 PR 写明，不改回落逻辑。
- `resolve_referral_membership_upgrade`（`postgres_auth_repository.rs`）：持日卡期间邀请人数达标，不得清掉日卡到期时间、不得错降级，需补测。
- `membership_origin` 触发器：结算事务内沿用 `set_config('football.membership_origin','purchase')`，不另造。

## 4. 目录与接口

### 4.1 单一数据源与兼容

目录由商品表生成，`system_config.membership_product` JSON 逐步退役。为了在**只有一个生产库**的前提下安全切换，引入开关：

- `system_config` 新键 `catalog_source`：`legacy`（默认，现状）或 `table`。
- `table` 模式下，旧接口 `GET /api/v1/payment/membership-product` 由商品表重新生成，**响应结构与现有字段字节兼容**，只追加可选字段（`expiry_rule`、`duration_label`、`group_key`、`is_pass`）。
- 上线前用"影子比对"：实现者提供一个只读的比对命令/测试，输入当前生产 `membership_product` JSON 和商品表，断言 `table` 模式生成的 V6–V9 年卡与 `legacy` 输出一致（价格、标题、副标题、描述、有效期、补差价）。比对通过才允许切换，回滚 = 把开关改回 `legacy`，无需发版。
- 年卡的 `subtitle`/`description` 目前只在 JSON 里，需要一次数据回填写进商品表对应行。**该回填是对生产数据的写操作，由用户确认后执行**，实现者提供幂等 SQL，不自行执行。

### 4.2 客户端能力门控

新版小程序在 `GET /payment/membership-product` 与下单请求带头 `X-Client-Capabilities: catalog-v2`。

- 没有该头：`requires_capability='catalog-v2'` 的 SKU 不出现在目录里，下单返回 400 `请更新小程序后购买`。原因：旧版小程序 `formatMembershipDuration(null)` 会把日卡显示成"一年"，协议文案也写死"有效期为一年"，必须避免误导购买。
- 年卡对新旧客户端返回完全一致。

### 4.3 新增接口（可选字段为主）

- `GET /api/v1/payment/catalog?kind=`：统一目录，按用户资格过滤，返回分组后的 SKU。供新版小程序使用；旧接口保持。
- 不删除任何旧接口。

## 5. 后台（Web 管理端 + Admin API）

现有 `admin_product.rs`、`product_dto.rs`、`postgres_admin_product_repository.rs`、`football_insight_admin_web/src/features/products/*` 只支持会员。重构为：

- Admin API 按 kind 校验：保存时调用对应 `ProductKind::validate`，档位白名单整体移除；新增读取 `kinds` 元数据的接口，返回各 kind 的 `grant_spec` 字段定义（字段名、类型、枚举、必填），**Web 表单据此渲染，新增字段不改前端**。
- 后台能直接创建/编辑/启停三类商品，包括日卡（`expiry.rule=end_of_next_day`）、受众条件、展示分组、排序。
- 保持现有会员商品页面的行为、权限、审计不变；已有行编辑不受影响。
- 后台保存价格时校验 `product_key` 的 `@价格` 与 `price_cents` 一致（沿用现有 CHECK），并提示"该价格需与微信后台道具一致"。

## 6. 分期实施（每期独立可部署、可验收）

| 期 | 内容 | 零代码验收 | 目标时间 |
|---|---|---|---|
| **P1** | 迁移（新增列、放宽 CHECK、订单快照列）；membership kind 数据驱动（去白名单、到期规则、受众、能力门控、结算修复）；日卡 SKU 通过商品表公开展示；小程序购买页分组、日卡文案；年卡仍走 `legacy` JSON | 部署后 **只 INSERT 两行** 即上线 V4/V5 日卡 | 10 月 20 日前上线，赶在 10 月 24 日主场前 |
| **P2** | 年卡回填并切到 `table` 目录（影子比对 + 开关）；后台按 kind 通用化 | 在后台页面新增一个会员 SKU，无需改代码或发版 | P1 后一周内 |
| **P3** | `match_id_unlock`、`reflux_subscription` 接入商品目录：价格以商品行为准，移除常量与套餐表价格依赖，补齐微信道具映射 | 在后台新增一个回流订阅套餐并可购买 | P2 后 |

**P1 为什么先行**：下一个购买高峰约在 10 月 21–24 日，P1 解决日卡上线所需的全部阻塞点；P2、P3 是消除重复来源的收尾，风险更高（动已稳定的年卡链路），放到高峰之后。

## 7. P1 详细要求

- 新增迁移（时间戳大于现有最新 `20261016120000`）：
  - 商品表：增加第 2.2 节的列；`kind` 用 `product_key` 前缀回填并设 NOT NULL；`grant_spec` 对现有行回填（membership 行：`{"tier":..., "expiry":{"rule":"months","months":<duration>}}`）；
  - `virtual_product_key_matches_price` 改为 `membership:V[3-9]` 档位范围，保持 V3 必须限定账号的约束；
  - 订单表：`grant_snapshot JSONB NULL`。
  - 全部为加列或放宽约束，现有数据语义不变，PostgreSQL 版本需支持加默认值列不重写表（≥ 11），实现者先确认生产版本。
- 日卡 SKU 示例（上线时执行，不属于迁移；`enabled=false` 先验证）：

```sql
INSERT INTO f_i_virtual_payment_products
 (product_key,name,product_id,price_cents,enabled,kind,grant_spec,eligibility,requires_capability,
  title,subtitle,description,group_key,sort_order,membership_duration_months)
VALUES
 ('membership:V4@900','V4 场次体验卡','fi_v6_day_9',900,false,'membership',
  '{"tier":"V4","expiry":{"rule":"end_of_next_day","tz":"Asia/Shanghai"}}',
  '{"max_current_tier":"V3"}','catalog-v2',
  'V4 场次体验卡','次日 23:59 前有效','支付后立即升级为 V4，余票监控刷新提速至 60 秒。','pass',10,1),
 ('membership:V5@1900','V5 场次体验卡','fi_v9_day_19',1900,false,'membership',
  '{"tier":"V5","expiry":{"rule":"end_of_next_day","tz":"Asia/Shanghai"}}',
  '{"max_current_tier":"V3"}','catalog-v2',
  'V5 场次体验卡','次日 23:59 前有效','支付后立即升级为 V5，余票监控刷新提速至 30 秒。','pass',20,1);
```

- 小程序：
  - `src/api/payment.ts` 请求头 `X-Client-Capabilities: catalog-v2`；`src/types/payment.ts` 新增可选字段。
  - `helpers.ts`：`formatMembershipDuration` 优先用 `duration_label`；协议第一条的有效期措辞由 `duration_label` 生成，日卡文案"有效期至购买次日 23:59（北京时间）"，**不得出现"一年"**；年卡文案逐字不变；新增按 `group_key` 分组的函数。
  - `MembershipTierList.vue` 支持分组渲染（"场次体验卡"、"年度会员"）；选中状态仍以 `target_tier` 为键（V4/V5 与 V6–V9 不重名）。
  - 遵守 `football_insight_mini/AGENTS.md`：不加 ALLOWLIST 例外，颜色引用 token，不手改 `versionName`。
- 微信后台：商品名称带 V6、V9 的两个道具建议改名为"V4 场次体验卡"、"V5 场次体验卡"（ID 不变）；联调用的私有测试行 `membership:V6@900`、`membership:V9@1900` 在日卡上线后删除或停用。

## 8. 向后兼容（硬性要求，小程序已上线）

- 迁移只加列、放宽约束；不删列，不改现有数据语义。
- 现有接口路径、请求体、已有响应字段、状态码、错误文案不变；新增字段全为可选。
- V6–V9 年卡价格、有效期、补差价、结算结果与改动前逐例一致，由现有测试与新增回归用例证明；`catalog_source` 默认 `legacy`，P1 不切换。
- 旧版小程序：看不到带 `requires_capability` 的 SKU，下单不能买；其余购买流程不受影响。
- 新版小程序遇到旧后端响应（无新字段）：渲染与现在一致。
- 没有快照的在途/历史订单走旧发货路径，行为不变。
- 观测（`f_i_payment_attempt_events`）不受影响，新路径同样要产生 `order_created`、`pay_params_issued`、`delivered` 等服务端事件。
- 只有生产一个数据库：写库测试必须事务回滚；不得创建真实订单；手工验证只用运营测试号；迁移与数据回填不自行对生产执行，等用户指令。

## 9. 测试要求

- 后端单元测试：每个 kind 的 `validate / eligible / resolve_price / fulfill`；membership 结算表驱动测试覆盖 V3–V9、年卡与日卡、同级续费、日卡不缩短到期、持日卡期间邀请达标；`end_of_next_day` 在北京时间 0 点前后、月末、跨年。
- 回归：现有 `virtual_payment_tests`、`tests/virtual_payment_domain.rs`、`tests/admin_products_postgres.rs` 全部保持通过。
- 门控测试：无 `X-Client-Capabilities` 时目录无日卡、下单 400；邀请得来的 V4+ 用户、年会员下单被拒。
- 小程序：`bun test`、`type-check`、`check:boundaries`、`verify:mp-components`；新增 `helpers` 分支的测试，含"旧响应渲染不变"。

## 10. 范围外

- 修改年卡价格或权益；
- 日卡到期提醒、续购引导；
- 收紧"到期回落 V3"的逻辑；
- 回流订阅、比赛 ID 解锁的具体定价（P3 只负责接入目录，价格由运营在商品行里设定）；
- 安卓管理端。

## 11. 开放问题

1. V4/V5（60/30 秒刷新，无回流速览、不含免费看比赛 ID）对 ¥9/¥19 的吸引力，上线后用可观测化数据复盘。
2. V1/V2 买日卡后停在 V3，本期接受。

## 12. 审查清单（Review 逐条核对）

1. **零代码验收**：P1 合并后仅靠 INSERT 能上线日卡；P2 后仅靠后台页面能新增会员 SKU。
2. 结算：V4/V5 付费订单会升级用户；V6–V9 年卡结算与改动前逐例一致；日卡不缩短到期、不降级。
3. 到期：`end_of_next_day` 的各边界取值正确（0 点、月末、跨年）。
4. 服务端下单二次校验：年会员、邀请得来的 V4+ 用户、持日卡用户、无能力头的请求都不能买日卡。
5. 旧客户端：目录里没有日卡，年卡响应字节不变。
6. 没有残留档位写死：`product_key()`、订单类型归一化、私有商品分支、后台 `validate_tier` 均走 `is_valid_membership_tier()`/kind 校验；未知档位不再回落成 V9。
7. 发货只读订单快照，不读商品表；无快照订单仍走旧路径。
8. 影子比对与回滚开关存在，且 P1 未切换 `catalog_source`。
9. 迁移时间戳大于 `20261016120000`；未对生产自行执行迁移或数据回填；测试未污染生产库。
10. 小程序：无 ALLOWLIST 新增、无散落 hex、无手改 `versionName`；日卡协议文案不含"一年"。
11. 观测事件在新下单/发货路径上保持完整。

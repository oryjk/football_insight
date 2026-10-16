# 商品目录数据驱动化 P1 交付与验证

日期：2026-10-16。主分支基线 `d478ae8`。设计：`docs/superpowers/specs/2026-10-16-product-catalog-data-driven-design.md`。
仅 P1；后端、小程序分两个英文 subject 提交，均带 Co-Authored-By；不新建分支。

## 结果与发布状态

- 会员 SKU 行包含价格、发货规则、受众、能力门控和展示；精确两条运营 INSERT 经事务验证可以生成 V4 ¥9 / V5 ¥19 日卡，启用后目录可见。到期为购买次日 23:59:59（北京时间）。
- V6–V9 年卡继续 legacy JSON 与原补差价，P1 没有切换 catalog_source。新会员订单冻结快照，历史无快照订单保留旧结算。
- **本地后端已重启**：`./deploy/local-test/up.sh` 最后北京时间 12:18:44 返回 active / health status=ok，18092。
- **生产后端未部署、未重启**；没有上传小程序。迁移/种子只在验证事务内执行并回滚，**未持久应用**，没有保留任何测试商品或订单。

## 真实验证命令与输出结论

| 命令 | 结果 |
|---|---|
| `./deploy/local-test/up.sh` | exit 0；Finished dev profile、active、health ok；既有 compose orphan/no-services-to-build 提示，未清理无关容器 |
| 后端 `cargo test` | exit 0；29 suites，**464 passed、0 failed、65 ignored**；完整输出在 `.tmp/product-catalog-p1/cargo-test-final.log` |
| 后端 `cargo check` | exit 0；Finished dev profile |
| 小程序 `bun run type-check` | exit 0；vue-tsc --noEmit 无错误 |
| 小程序 `bun test` | exit 0；**680 pass、0 fail、60 files**，主 agent 复验 12.89 秒 |
| 小程序 `bun run check:boundaries` | exit 0；依赖边界检查通过 |
| 小程序 `bun run verify:mp-components` | exit 0；All mini-program custom components are registered. |
| `MINI_REVIEW_SKIP=1 bun run build:mp-weixin` | exit 0；Build complete、组件注册与产物 API 检查通过，无登记、无上传；既有 uni-app/Browserslist 更新提示 |
| 改动的 Rust 文件 `rustfmt --check --edition 2024 --config skip_children=true` | exit 0；25 个 Rust 文件 |
| `git diff --check` | exit 0 |
| `cargo test --test admin_products_postgres --test virtual_payment_postgres --no-run` | exit 0；两套 PostgreSQL fixture 和新增忽略的 V4/V5 CRUD 用例编译通过，没有执行 ignored 测试 |
| scratch SQL 验证脚本 `uv run python …/verify_sql_rollback.py --execute` | exit 0；以下事务证据全部成立 |

首次全量测试曾有两个旧断言失败：`rejects_unsupported_tiers_and_nonpositive_prices`、`rejects_blank_fields_and_nonmembership_or_noncanonical_keys` 仍把 V4 当成无效档位。已改用 V2 作为无效输入并新增 V3–V9 正向校验；最终全量为上表结果。

## 生产 SQL 事务验证（实际执行，不是静态推断）

先用只读查询确认 PostgreSQL `server_version_num=160014`（16.14，满足 ≥11）。连接串从后端 `.env` 静默读取，未输出或提交。

脚本和证据仅放 scratch：

- `.tmp/product-catalog-p1/verify_sql_rollback.py`
- `.tmp/product-catalog-p1/final-operator-seed-rollback-report.json`
- `.tmp/product-catalog-p1/legacy-admin-red-report.json`

验证按 `BEGIN` → `SET LOCAL lock_timeout='2s'` / statement_timeout=10s → 迁移 → 全新模拟商品/订单 → 精确源码 SQL → **finally 无条件 ROLLBACK** 执行。使用一个已有用户 ID 满足模拟订单 FK，真实用户不更新；结算 UPDATE 被同名 TEMP f_i_users 隔离。

已覆盖：

1. 实际迁移全部 DDL 与新字段回填，CHECK 放宽可以在 PostgreSQL 事务里完整验证和回滚；期间其他约束保持。
2. 4 条实际商品查询语句共 13 次运行，覆盖 public/private、指定用户/其他用户、目录、按 key 选行与旧私有目录隐藏能力 SKU。
3. 实际订单快照 INSERT；实际 duration/grant_snapshot/NOW 元数据 SELECT；新快照和原 legacy 两条结算 UPDATE（只更新 TEMP 用户）。
4. 9 类实际 Admin SQL：省略 kind 的原创建、get、lock、list、count、会员/购买身份查询、update、delete；只操作测试新建行。V4 和 V5 的 scope 都实际核验。
5. 4 个北京时间参考 SQL 边界：0 点前、0 点后、月末、跨年；与 Rust 纯函数测试配套。
6. **原样执行** `docs/sql/2026-10-16-membership-pass-skus.sql` 两条 INSERT：验证微信 ID/价格、发货、maxV3、catalog-v2、pass 分组和初始禁用；在事务内仅启用这两条新建 UUID 后，实际目录 SQL 返回两张日卡。

最终证据：rollback_executed=true、schema_reverted=true，所有测试商品/订单/虚拟订单残留=0，operator_seed_keys_remaining=0；回滚前后列及全部约束逐项一致。迁移 SHA256：`36c5494e8d23dc338dab946eb518bfeac8f7314b873e7cc17acaca8fc9c0d624`。

TDD SQL 还证明：修复前精确旧 Admin INSERT 报 23502；补上 kind 的兼容默认值后通过。最初 scratch 参数转换遇到 psycopg 对 LIKE 字面量百分号的转义错误，也已无条件回滚清理后修复重跑，不涉及应用 SQL 修改。

## 设计第 12 节审查清单

| # | 结果 | 依据 |
|---|---|---|
| 1 零代码验收 | **满足 P1**；P2 部分未做 | 原样两条 INSERT + 事务内启用后，实际目录查询返回 V4/V5；backend 无 V4/V5 SKU/价格分支。后台新增任意商品属于 P2 |
| 2 结算 | **满足** | 快照付费 V3–V9 表驱动测试；V6–V9 对旧结算 324 组合；日卡不缩短、不降级，永久邀请权益竞态回归通过 |
| 3 到期边界 | **满足** | Rust expiry 纯函数及生产事务参考 SQL 均验证北京时间 0 点前后、月末、跨年，次日最后一秒正确 |
| 4 下单二次校验 | **满足** | listing、checkout、虚拟参数签发均校验能力/受众；缓存目录后用户升到 V4+ 的 fake 回归拒绝且不创建订单 |
| 5 旧客户端 | **满足** | 无能力头目录只有原年卡，日卡下单 400 固定更新文案；新元数据均 skip None，不增加旧年卡字段；价格/补差价和旧渲染回归通过 |
| 6 单一档位校验 | **满足** | product_key、membership_tier_from_product_type、私有目录、Admin validate_tier 共同调用 is_valid_membership_tier；V3–V9 正向、未知档位错误测试；SQL scope 同步范围 |
| 7 快照与历史兼容 | **满足** | 新 virtual order 同事务写快照；结算只读虚拟订单的 grant_snapshot，不查商品；NULL 分支保留原 paid-tier 判断、时长与 SQL |
| 8 影子比对/回滚开关 | **不满足整项（P2 部分有意未做）；满足 P1 legacy 要求** | 影子比对、table 目录切换和开关操作属于设计分期 P2，按用户仅做 P1 不实施；本期始终 legacy，不写 catalog_source |
| 9 迁移/数据库 | **满足** | 20261016130000 > 20261016120000；仅新列/新列回填/约束放宽，V3 限定账号保留；未持久执行，事务验证恢复与零残留有证据 |
| 10 小程序约定 | **满足** | 边界检查通过，无 ALLOWLIST/版本改动，无新增 hex；真实 SFC 渲染与协议测试验证日卡不含“一年”，旧响应原文案保留 |
| 11 观测完整 | **满足** | 新创建测试检查 order_created、pay_params_issued / pay_params_failed 及原 Provider 类型；结算复用同一 delivered 调用点，现有 nonblocking/重复发货测试继续通过；独立观测 pool、spawn/超时/吞错未改 |

## TDD、审查修正和设计取舍

- domain、目录门控、checkout 及快照签发分别先运行缺失方法/行为的失败测试，再实现；快照缺失曾使测试解包失败，接通后验证保存 V4 grant。
- 私有/公开的新目录 SKU 同档位时，目录按 sort_order、price、key 选择；checkout 重新核对选中 identity/key/price，避免 ¥9 展示却收 ¥19。回归先实际 1900 / 预期 900 失败，再通过。无资格的兄弟 SKU 不再否决已选中的合法 SKU。
- 新 fixed SKU 不套用 legacy 年卡补差价；新增测试先得到 -1900 / 预期 500，再通过。原年卡没有 spec 元数据，继续原分支。
- 等待发货期间用户已取得永久 V4+ 邀请权益的日卡，保留已有等级/无限期，避免付款后把永久权益改成会到期的权益；测试先 V5 / 预期 V4 失败再通过。没有改 auth 到期回落规则。
- kind 注册只包含 P1 membership，bootstrap 统一装配同一个 Arc registry 并注入目录、下单、参数签发。ProductKind 的校验、受众和定价保持纯 domain，事务 fulfill 仍通过既有 settlement port / persistence 实现，避免把 SQLx Tx 放入 domain。这是对设计示意 trait 的分层适配。
- pricing_rule 的 P1 数据 SKU 支持 fixed；既有年卡 upgrade-credit 计算仍在 legacy 分支（含原年度档位和费用），未提前做 P2/P3 的迁移。新的到期规则限定 months / end_of_next_day。
- kind 默认 membership 兼容既有 Admin 与 SQL INSERT；现有行仍按原 product_key 前缀回填，不冒充其他类型。新 kind 必须显式填 kind。旧后台月数/档位编辑在 P1 仍从原列重建 legacy checkout 快照；显式新 SKU 的 grant_spec 不被覆盖。
- V1/V2 用户买日卡后到期停在 V3，按设计接受，不修改回落逻辑。
- 没有业务路径缺失。设计 line 号已漂移，按实际文件定位；快照列的实际表为 f_i_virtual_payment_orders。新增 kind 纯函数与快照结算拆为 product_kind.rs / membership_settlement.rs；小程序真实 SFC 测试继续放 scripts，避免给应用引入 Node 类型。
- 新增 /payment/catalog 是设计可选接口，本期复用既有 membership-product。没有 P2 影子比对/开关/table 模式、年卡展示文案回填、通用后台表单、P3 商品接入，也未实现此前可观测化 review 的非阻塞后续建议。
- 独立 review 提出的四项 Important（永久权益竞态、Admin kind 默认、Admin scope、同档位选价）均已修正并复查；最终无剩余阻塞。

## 未做的测试与限制

- 65 项 ignored 数据库集成测试没有对生产执行；其中新增 V4/V5 CRUD 用例与两套 fixture 已编译，实际 SQL 通过上述独立事务验证。没有运行 Rust adapter 完整 commit 发货/退款链，因为它们使用 pool 自行提交，无法保证只触及测试自己的数据并统一回滚。新 SQL 的用户 UPDATE 只在临时表验证，既有 purchase-origin 触发器和退款全链仍依赖原回归覆盖。
- 没有真实支付、真实微信参数/发货网关调用，也未创建已提交的测试订单/用户/商品。没有生产部署或小程序上传。
- 没有微信模拟器/真机验收；已做真实 SFC server-render、选择事件、页面 caption/协议、680 项测试及离线微信构建。
- 生产迁移未持久应用，因此没有用重启后的本地服务对新目录做写接口验证；本地连生产。接口编排用 fake 测试，SQL 用事务验证，两者分开报告。
- P2/P3 的验收项目明确未做，不将其标为本期已完成。

## 用户上线操作清单（按顺序，均未代执行）

1. 在 peiqian 通过约定迁移工具应用尚未应用的观测迁移 `20261016120000_add_payment_attempt_events.sql`，再应用 P1 的 `20261016130000_add_product_catalog_specs.sql`。生产事务验证未写 SQLx 迁移账本，不等于已应用。
2. 部署并重启生产 Rust 后端（现有 `./deploy_peiqian_docker.sh`）；验证旧客户端年卡输出、catalog-v2 门控和上报入口。此步由用户指令执行。
3. 用户执行 `docs/sql/2026-10-16-membership-pass-skus.sql` 两条 INSERT：V4@900 / fi_v6_day_9、V5@1900 / fi_v9_day_19。按设计先 enabled=false；本任务没有把种子写入迁移。
4. 用户在微信后台把 `fi_v6_day_9` 名称改为 **V4 场次体验卡**，把 `fi_v9_day_19` 改为 **V5 场次体验卡**；ID 不变，核对已发布价格分别 ¥9 / ¥19。
5. 用户按统一发布入口发布带 catalog-v2 的小程序；不要手改版本。P1 实现过程没有调用发版入口。
6. 用户确认后启用这两个新 SKU，同时停用旧联调映射（保留历史订单，不删除商品身份）：

```sql
BEGIN;
UPDATE f_i_virtual_payment_products
SET enabled = true
WHERE product_key IN ('membership:V4@900','membership:V5@1900');
UPDATE f_i_virtual_payment_products
SET enabled = false
WHERE product_key IN ('membership:V6@900','membership:V9@1900');
COMMIT;
```

这两条 UPDATE 是明确的后续生产操作，未执行。若全部发布前提已就绪，两条 INSERT 使用 enabled=true 即可直接上线，无需再改代码；交付的种子按设计采用 disabled 验证流程。首次启用后再用原有观测接口检查实际漏斗，真实支付验收由用户指示。

## 改动文件清单

### 后端（27 个）

- `football_insight_service_backend_rs/docs/api.md`
- `football_insight_service_backend_rs/migrations/20261016130000_add_product_catalog_specs.sql`
- `football_insight_service_backend_rs/src/admin/adapters/persistence/postgres_admin_product_repository.rs`
- `football_insight_service_backend_rs/src/admin/domain/admin_product.rs`
- `football_insight_service_backend_rs/src/auth/adapters/persistence/postgres_auth_repository.rs`
- `football_insight_service_backend_rs/src/payment/adapters/persistence/postgres_payment_settlement_port.rs`
- `football_insight_service_backend_rs/src/payment/adapters/persistence/postgres_virtual_order_repository.rs`
- `football_insight_service_backend_rs/src/payment/adapters/persistence/postgres_virtual_product_repository.rs`
- `football_insight_service_backend_rs/src/payment/adapters/web/dto.rs`
- `football_insight_service_backend_rs/src/payment/adapters/web/handlers.rs`
- `football_insight_service_backend_rs/src/payment/application/create_membership_order.rs`
- `football_insight_service_backend_rs/src/payment/application/get_membership_product.rs`
- `football_insight_service_backend_rs/src/payment/application/handle_wechat_notify.rs`
- `football_insight_service_backend_rs/src/payment/application/virtual_payment.rs`
- `football_insight_service_backend_rs/src/payment/application/virtual_payment_tests.rs`
- `football_insight_service_backend_rs/src/payment/bootstrap.rs`
- `football_insight_service_backend_rs/src/payment/domain/membership_settlement.rs`
- `football_insight_service_backend_rs/src/payment/domain/mod.rs`
- `football_insight_service_backend_rs/src/payment/domain/order.rs`
- `football_insight_service_backend_rs/src/payment/domain/product_kind.rs`
- `football_insight_service_backend_rs/src/payment/domain/virtual_payment.rs`
- `football_insight_service_backend_rs/src/payment/ports/virtual_payment_port.rs`
- `football_insight_service_backend_rs/src/payment/ports/wechat_pay_port.rs`
- `football_insight_service_backend_rs/tests/admin_products_postgres.rs`
- `football_insight_service_backend_rs/tests/product_catalog_application.rs`
- `football_insight_service_backend_rs/tests/product_catalog_domain.rs`
- `football_insight_service_backend_rs/tests/virtual_payment_postgres.rs`

### 小程序（8 个）

- `football_insight_mini/scripts/membership-catalog-render.test.ts`
- `football_insight_mini/src/api/payment.ts`
- `football_insight_mini/src/api/paymentCatalog.test.ts`
- `football_insight_mini/src/pages/membership-purchase/components/MembershipTierList.vue`
- `football_insight_mini/src/pages/membership-purchase/helpers.test.ts`
- `football_insight_mini/src/pages/membership-purchase/helpers.ts`
- `football_insight_mini/src/pages/membership-purchase/index.vue`
- `football_insight_mini/src/types/payment.ts`

### 共享文档与运营 SQL（7 个）

- `CHANGELOG.md`
- `FEATURES.md`
- `docs/payment-observability-validation.md`
- `docs/sql/2026-10-16-membership-pass-skus.sql`
- `docs/superpowers/plans/2026-10-16-product-catalog-p1.md`
- `docs/virtual-payment.md`
- `docs/product-catalog-p1-validation.md`

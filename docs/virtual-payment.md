# 微信虚拟支付配置与验证

会员、比赛 ID 解锁和回流提醒的新增交易统一使用官方道具直购模式，不包含代币或自动续费。后台审核、签约完成后才配置密钥和商品。代码默认关闭购买，不回退普通微信支付；历史普通支付订单仍由原回调处理。

## 后台开通与商品

1. 在小程序后台「支付与交易 → 虚拟支付」完成支付管理员签约，等待进入商户管理模块。
2. 在基本配置取得 OfferID 和现网 AppKey，仅放在后端部署环境中，不放前端、不发送到聊天、不提交 Git。
3. 配置小程序消息推送的 Token、EncodingAESKey，选择**安全模式**。服务器 URL 为 `https://match.oryjk.cn/api/v1/payment/virtual/notify`；启用道具发货和退款事件。这里是小程序的推送配置，不能替换公众号 `/football/wechat/webhook` 的配置。
4. 在商品管理创建并发布商品，等待发布生效。商品 ID、金额必须与后端映射一致。
5. iOS 需另行配置小程序简称和 Apple 支付；用户端需要满足官方支持条件。未支持的客户端不会回退其他支付渠道。

## 后端环境变量

| 环境变量 | 用途 |
| --- | --- |
| `WECHAT_VIRTUAL_PAY_ENABLED` | 默认 false；全部配置验证完成后设为 true |
| `WECHAT_VIRTUAL_PAY_OFFER_ID` | 后台提供的 OfferID |
| `WECHAT_VIRTUAL_PAY_APP_KEY` | 现网 AppKey，仅服务端 |
| `WECHAT_VIRTUAL_PAY_WEBHOOK_TOKEN` | 与小程序消息推送 Token 一致 |
| `WECHAT_VIRTUAL_PAY_ENCODING_AES_KEY` | 与安全模式 EncodingAESKey 一致 |

同时沿用 `WECHAT_MINI_APP_ID`、`WECHAT_MINI_APP_SECRET`。虚拟支付回调使用新的配置，公众号 Token/AESKey 不需要修改。只支持现网 `env=0`；本地测试用 fake gateway，不使用生产密钥或真实收费。

回调初始化只要求有效的小程序 AppID、Token 和 EncodingAESKey。即使 `WECHAT_VIRTUAL_PAY_ENABLED=false`，或尚未配置商品，服务器地址签名验证仍可正常完成；购买状态继续返回 `available=false`。因此可以先关闭购买，部署后验证消息推送地址，再配置并发布商品。

## 数据库商品配置

商品映射唯一来源是 `f_i_virtual_payment_products`，不再读取 `WECHAT_VIRTUAL_PAY_PRODUCTS`。迁移 `20261013220000_add_virtual_payment_products.sql` 初始化已发布的 V6 年会员：`membership:V6@3900 → fi_v6_year_39`，3900 分，启用。其他商品需先在微信后台发布，再添加数据库记录。

| 字段 | 用途 |
| --- | --- |
| `product_key` | 主键，业务规格 + `@` + 实付金额（分） |
| `name` | 商品名称，供配置管理使用；微信收银台名称/图片仍需在微信后台修改 |
| `product_id` | 微信后台已发布的道具 ID |
| `price_cents` | 道具价格（分），必须与 key、业务结算价及微信后台一致 |
| `enabled` | 是否允许新增购买；默认 false |
| `membership_duration_months` | 会员有效期（月），默认 12，范围 1–1200；12 表示一个自然年，比赛 ID/提醒商品不使用此字段 |
| `allowed_user_ids` | 允许账号 UUID 数组；空数组表示普通公开商品，V3 测试商品必须指定账号 |
| `remark` | 内部备注 |
| `created_at` / `updated_at` | 创建/修改时间，更新时自动刷新 |

映射 key 规则：

- `membership:V3@1`：私有 VIP3 测试商品，1 分，V3 权益，默认一年；仅白名单账号可见并购买。
- `membership:V6@3900`：V6 年会员，3900 分。
- `membership:V7@3500`：V7 补差价升级，3500 分。不同档位/补差价的固定价格道具分别添加；V9 续费复用 V9 完整价格道具。
- `match_id_unlock@500`：单场比赛 ID 解锁，500 分；具体比赛由订单绑定。
- `reflux_subscription:single_match:chengdurongcheng@500`：对应队伍的提醒套餐；具体场次由订单绑定，新增队伍需增加相应 key。

查询与更新示例（用数据库管理工具连接生产库执行，不在聊天中发送连接串）：

```sql
SELECT product_key, name, product_id, price_cents, membership_duration_months, enabled, remark, updated_at
FROM f_i_virtual_payment_products ORDER BY product_key;

-- 修改微信道具 ID：应先确认新道具已发布，价格相同。
UPDATE f_i_virtual_payment_products
SET product_id = '微信后台新的已发布道具ID', name = 'V6 年会员'
WHERE product_key = 'membership:V6@3900';

-- 修改新订单的会员期限，无需部署/重启；旧订单保留购买时快照。
UPDATE f_i_virtual_payment_products SET membership_duration_months = 12
WHERE product_key = 'membership:V3@1';

-- 暂停/恢复新增购买，无需重启后端。
UPDATE f_i_virtual_payment_products SET enabled = FALSE
WHERE product_key = 'membership:V6@3900';
UPDATE f_i_virtual_payment_products SET enabled = TRUE
WHERE product_key = 'membership:V6@3900';
```

添加其他商品参考 `football_insight_service_backend_rs/config/virtual-payment-products.example.sql`；示例中未发布商品保持停用。商品表保存道具映射及会员有效期，不覆盖原有业务定价。会员定价仍在 `f_i_system_configs` 的 `membership_product`，提醒套餐定价仍在原套餐表；比赛 ID 解锁仍按原业务固定 5 元。修改会员/提醒售价时需同步对应数据库业务定价、微信后台道具价格，以及商品表的 `product_key` / `price_cents`；金额不一致或映射缺失时拒绝支付，不回退普通支付。

每次准备支付会话/下单实时读库，无进程内缓存。会员列表和公开购买状态接口都实时读库，支付路径排除 HTTP 响应缓存；数据库读取失败时关闭购买。历史订单保存购买时的微信商品 ID、金额和会员期限，修改或停用商品不会改变其查单、发货或退款。密钥、OfferID 和购买总开关继续使用环境变量。

本次仅增加数据库表和支付读库逻辑；商品管理页面未包含在此次改动中。

## 部署顺序

1. 本地检查通过后提交/推送代码，生产迁移由 `peiqian` 上的 `cargo run --bin run_migrations` 应用，包含 `20261013000500_add_virtual_payment.sql` 和 `20261013220000_add_virtual_payment_products.sql`。
2. 确认商品表记录与当前已发布商品一致；备份环境文件并移除旧 `WECHAT_VIRTUAL_PAY_PRODUCTS`，保留密钥和购买开关，使用 `deploy_peiqian_docker.sh` 部署并重启；现有脚本支持 env-file，无需修改其他 Nginx 路由。
3. 查询 `/api/v1/payment/virtual/status`，配置完成时返回 `available=true`。它只证明本地配置齐备，后台商品已发布、支付限制是否解除仍需真实验证。
4. 上传新的小程序。旧客户端没有虚拟支付会话准备，不能继续使用普通支付发起新增数字服务交易。
5. 用管理员本人测试三条购买链路、会员补差价、重复通知和退款。核对微信订单、本地订单及权益；保存截图供申诉复核。真实付款必须由用户操作。

本次本地验证使用端口 `18093` 和新建的隔离 PostgreSQL，后端已重启；原 `18092` 联调容器及其生产数据库连接未改动。生产迁移与后端部署已完成，小程序上传和真实付款均未执行。

执行生产迁移时需要显式给迁移进程提供运行时环境文件中的 `DATABASE_URL`；Docker 的 `--env-file` 不会自动把配置带入宿主机的 Cargo 进程。不要在终端输出连接串或密钥。

## 2026-10-13 生产发布记录

- 虚拟支付代码和两次回调修正均已合并并推送到 main。
- 生产迁移 `20261013000500` 应用成功，两张虚拟支付表与 `_sqlx_migrations` 成功记录已核对。
- peiqian 生产后端已部署并重启为 `football-insight-service-backend-rs:cac6f89`，容器健康；运行时环境文件与容器均保持 `WECHAT_VIRTUAL_PAY_ENABLED=false`。
- 通过线上域名验证：普通签名和安全模式 URL 校验返回准确回显，错误签名及错误 AppID 返回 403；购买状态为 `available=false`，原有会员商品和公开配置接口返回 200。
- AppKey、OfferID、Token 和 AESKey 已配置；商品映射未配置（当时仍使用环境变量）。小程序上传与真实付款均未执行，管理员现在可以回微信后台重新提交服务器地址验证。

## 2026-10-13 V6 真实支付测试准备

- 已在 peiqian 环境变量配置 `membership:V6@3900`，商品为 `fi_v6_year_39`、3900 分；生产环境文件已备份，`WECHAT_VIRTUAL_PAY_ENABLED=true`，未配置的其他商品仍无法拉起支付。
- 生产后端重建并重启为 `football-insight-service-backend-rs:4061501`，容器健康，线上状态 `available=true`，运行容器已读取商品映射。
- 从干净 main（`4061501`）通过 robot 2 上传小程序 1.0.66；546 项前端测试、类型检查、组件/API 校验通过。Mac 发布连接暂不可用，使用 peiqian 备用发布环境。
- 真机测试选择安卓微信。由用户在体验版用 V1–V5 账号购买 V6 年会员（39 元）；服务端配置就绪不代表真实支付链路已成功，需核对平台支付、订单发货及会员权益后才能确认。
- 用户要求：先完成真实支付链路验证，再将商品映射迁入数据库。迁移后应在下单时读库，数据库修改立即生效，密钥继续保留环境变量，历史订单商品与金额保持购买时快照。

## 2026-10-13 商品表生产发布

- 用户确认安卓真机已能调起收银台后，要求立即迁入独立商品表；实际付款与会员权益结果仍未确认，不把调起收银台作为真实支付成功。
- 代码已提交并推送 main（`3918951`）；生产迁移 `20261013220000` 应用成功，V6 记录为 `membership:V6@3900`、`fi_v6_year_39`、3900 分、启用。
- peiqian 后端已部署并重启为 `football-insight-service-backend-rs:3918951`，容器健康。生产环境文件已备份并仅移除 `WECHAT_VIRTUAL_PAY_PRODUCTS`；密钥和购买总开关保留，运行容器不再包含旧商品变量。
- 线上公开购买状态 `available=true`，不进入 HTTP 缓存；会员商品/公开配置接口 200，回调 URL 正确回显 200、错误签名 403。此次后端接口保持兼容，现有小程序 1.0.67 可继续使用，无需再次上传。
- 377 项后端测试、13 项隔离 PostgreSQL 测试通过；本地隔离后端 18093 已重启，同一进程中启用/停用/恢复状态即时生效。严格 Clippy/all-targets 与全仓架构扫描仍有未修改模块的既有告警，本次商品配置代码无新增命中。

## 私有 VIP3 测试商品

- 2026-10-13：生产已直接写入 `membership:V3@1 → test`（1 分、启用、单账号白名单），商品数据写入前已备份；结构及 SQLx 版本通过直接事务 SQL 记录，未运行 Cargo 迁移入口。peiqian 后端已部署并重启为 `d188e79` 且健康。
- 线上核对：指定账号返回 V3/实付 1 分，其他账号和匿名返回无 V3；其他账号直接提交 `V3` 或带空格形式均为 400 且无新订单。378 项后端、550 项前端及 14 项隔离数据库测试通过，本地后端 18093 已重启。
- 小程序 1.0.68 已从干净 main（`d188e79`）通过 robot 2 上传，包大小 1886490 字节，价格显示 0.01 元；真实付款需用户在安卓体验版操作，当前记录不宣称实际支付成功。

已发布的微信道具 `test` 用于 `membership:V3@1`，价格 0.01 元。生产商品数据由直接 SQL 写入，不作为迁移种子提交；`allowed_user_ids` 只填用户指定测试账号的稳定 UUID，名称大小写或昵称修改不会影响授权。结构定义保留在 `20261013223000_add_restricted_virtual_products.sql` 供新环境重建；生产通过直接事务 SQL 更新结构并记录 SQLx 版本，未调用 Cargo 迁移入口。

列表仅为允许账号追加 VIP3 测试选项，匿名及其他账号均不显示；下单接口从当前账号目录选取商品，底层生成支付参数时再按订单账号检查白名单。数据库约束禁止 V3 商品使用空白名单。旧系统会员配置中的 V3（含空格或大小写变体）不会作为公共付费选项返回。

VIP3 对应项目既有 V3，付款确认后授予 V3 权益，到期日按订单保存的商品期限计算（默认 12 个月）；界面服务声明和卡片读取后台返回的期限。到期后沿用既有会员等级回退策略。真实支付结果由服务端确认后生效，测试程序不会替用户实际付款。

## 订单确认与退款

- 付款前重新获取微信 code，服务端验证微信身份与当前登录账号绑定一致，session_key 只在服务端短时内存保存。当前部署单后端实例；将来扩容多副本时需将会话缓存迁到共享存储或使用会话粘性路由。
- 发货通知必须安全模式解密、验签并检查 AppID；通过查单取得 Android/Apple 的统一平台订单号，再核对商品、金额、数量和正式环境。数据库事务保证重复通知不重复授予权益。
- 前端等待后端付款确认。后台每分钟补偿未确认订单，单批最多 20 条，自动补偿覆盖最近 30 天的订单；用户主动查询不受该时间窗口限制。
- 退款 ID 去重、累计退款不可超过订单金额。已发货订单全额退款时回收对应权益，部分退款保持原权益；未发货订单收到成功退款会阻止后续发货。
- 会员退款使用订单授予前后快照：保留后续购买或管理员调整，并修正后续退款链，避免复活已退款会员。基础档退款后，后续补差价购买的高档会员按已确认方案保留；正式开放退款前可另行定义升级退款政策。
- 比赛 ID 退款只移除该订单的解锁，其他有效付款仍保留；提醒退款只取消该订单生成的订阅。
- 本次不提供自动续费的 iOS 退款问询决策；收到此类事件不虚构退款证据，交由 Apple 按官方未获明确应答机制处理。iOS 用户在 App Store 发起退款，服务端处理成功退款通知。

## 验证命令

```sh
cd football_insight_service_backend_rs
cargo test
# 必须使用专门的隔离测试数据库，数据库名以 _test 结尾。
VIRTUAL_PAYMENT_TEST_DATABASE_URL=... cargo test --test virtual_payment_postgres -- --ignored
cd ../football_insight_mini
bun test
bun run type-check
bun run check:boundaries
bun run build:h5
MINI_REVIEW_SKIP=1 bun run build:mp-weixin
```

## 官方资料

- [虚拟支付接入](https://developers.weixin.qq.com/miniprogram/dev/platform-capabilities/business-capabilities/virtual-payment.html)
- [客户端支付接口](https://developers.weixin.qq.com/miniprogram/dev/api/payment/wx.requestVirtualPayment.html)
- [查询订单](https://developers.weixin.qq.com/miniprogram/dev/server/API/VirtualPayment/api_query_order.html)

## 会员有效期数据库配置

`20261013230000` 为商品与虚拟订单增加 `membership_duration_months`，全部现有商品和历史订单默认为 12。会员发货以订单快照设置到期日，新订单从商品实时读取期限；前端显示对应有效期，升级道具按实际补差价规格读取。会员等级与金额不变，比赛 ID 和提醒套餐沿用各自原有权益范围。

`20261013230100` 为已经确认付款但缺少到期日的旧 V3 订单补齐期限，从原支付时间计算，只有当前权益仍等于订单旧授予快照才调整账号；保留后续人工修改/购买并修复退款链。先完成结构更新，再部署新后端，最后单独事务回填，避免旧后端继续生成无期限 V3。

本地隔离后端 18093 已重启：修改商品 12→1→12 个月立即反映在同一进程的会员接口，指定/其他/匿名账号的访问限制保持有效。18 项隔离 PostgreSQL 测试覆盖订单期限快照、补差价规格、已付款 V3 回填、重复执行与多笔订单退款链；552 项前端测试、类型与边界检查、H5/小程序构建通过。生产更新与上传待执行。

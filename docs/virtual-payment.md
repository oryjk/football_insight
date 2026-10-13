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
| `WECHAT_VIRTUAL_PAY_PRODUCTS` | 单行 JSON 商品映射，见示例文件 |

同时沿用 `WECHAT_MINI_APP_ID`、`WECHAT_MINI_APP_SECRET`。虚拟支付回调使用新的配置，公众号 Token/AESKey 不需要修改。只支持现网 `env=0`；本地测试用 fake gateway，不使用生产密钥或真实收费。

回调初始化只要求有效的小程序 AppID、Token 和 EncodingAESKey。即使 `WECHAT_VIRTUAL_PAY_ENABLED=false`，或尚未配置商品，服务器地址签名验证仍可正常完成；购买状态继续返回 `available=false`。因此可以先关闭购买，部署后验证消息推送地址，再配置并发布商品。

商品映射示例：`football_insight_service_backend_rs/config/virtual-payment-products.example.json`。示例值基于代码默认价格，正式配置必须核对数据库当前会员价格、提醒套餐价格和实际队伍 code；不要直接把例子当作已配置的现网商品。

映射 key 规则：

- `membership:V6@3900`：V6 年会员，3900 分。
- `membership:V7@3500`：按现有补差价规则计算的 V7 升级，3500 分。不同档位/补差价的固定价格商品分别配置；V9 续费复用 V9 完整价格商品。
- `match_id_unlock@500`：单场比赛 ID 解锁，500 分；具体比赛由本地订单绑定。
- `reflux_subscription:single_match:chengdurongcheng@500`：对应队伍的提醒套餐；具体场次由订单绑定。队伍变化时添加对应 key。

value 为 `{ "product_id": "后台商品ID", "price_cents": 3900 }`。JSON 放进 Docker env-file 时不要加额外的外围引号，必须是单行合法 JSON。缺失商品或金额不一致时，下单失败且不会拉起收银台。

## 部署顺序

1. 本地检查通过后提交/推送代码，生产迁移由 `peiqian` 上的 `cargo run --bin run_migrations` 应用，包含 `20261013000500_add_virtual_payment.sql`。
2. 配置生产后端环境变量，使用 `deploy_peiqian_docker.sh` 部署并重启；现有脚本支持 env-file，无需修改其他 Nginx 路由。
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
- AppKey、OfferID、Token 和 AESKey 已配置；商品映射未配置。小程序上传与真实付款均未执行，管理员现在可以回微信后台重新提交服务器地址验证。

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

# 第一阶段静默登录 Codex Review

状态：**代码 review 与本地验证通过**。截至 2026-10-11，本次范围没有已确认而未修复的 P1/P2。微信开发者工具、真机登录/绑定/支付尚未验收。

实现者为本地 Kimi，固定模型 `kimi-code/k3-256k`（实际 `k3-256k`）；Codex 负责计划、独立复审、修复回合协调和最终验证。最后文档阶段 Kimi 再次耗尽额度，交付文档由 Codex 根据实际证据完成；业务代码全部由 Kimi 实施。

目录 `/Users/carlwang/projects/football_insight`，分支 `feat/mini-registration-alignment`。初始基线 `91d11c2`，最终业务提交 `183da4b`。依据为 [第一阶段计划](../superpowers/plans/2026-10-10-mini-silent-login-phase1.md)，实现说明见 [实施报告](2026-10-10-mini-silent-login-phase1-implementation.md)。本轮只修改小程序与相关文档。

## 修复与复审结果

| 问题 | 最终修复及验收 |
| --- | --- |
| 慢审核结束后自动恢复覆盖已完成的显式绑定表单 | `032125e`：显式认证/退出替代本次启动恢复；认证意图从获取 code 前建立，迟到 code 不继续兑换 |
| 首页 profile 401 抢先取消旧 token 的启动恢复 | `4c9fea3` / `991414c`：私有读取等待共享会话，公开内容独立加载；正常 401 允许一次微信恢复，非 401 保留 token |
| 游客 current 在途阻挡恢复后的新请求 | `b440991`：board owner 独立于订阅 generation；reset 后新请求立即发出，旧 success/catch/finally 不影响新 loading |
| history/history-stats 恢复后空白，页内切另一看板不加载 | `b440991` / `d271b42`：恢复刷新可见看板，真实 activeTab watcher 补载未加载目标；已载缓存复用，hidden/review/membership 门控有效 |
| insights onShow 与用户发布重复读取，旧错误遮住成功内容 | `4c9fea3` / `991414c`：同上下文读取合并，旧结果/错误/finally 按 owner 放弃；实际 H5 冷刷新只读取一次 |
| 跨账号旧缓存、loading 或 onShow continuation 污染新账号 | `4c9fea3` / `991414c`：先清私有数据与选择，身份在首个审核 await 前捕获；旧私有链停止，账号独立审核结果仍生效 |
| 主队 PUT 后复用写前 GET，或旧账号 PUT 强制替换新读取 | `991414c`：有效写后强制刷新，旧 GET 失效；旧账号 PUT 成功/失败/finally 不关新弹层、不 toast、不替换新读取 |
| 自然换场只 gen++，订阅 loading 与旧缓存永久残留 | `7727872`：所有实际比赛替换统一同步 reset 订阅/解锁状态；新比赛立即可读，旧 finally 不清新 loading |
| 换场使旧订单失效但解锁 paying 不复位 | `7727872`：同步清 sheet/paying/entitlement；旧订单不拉起支付，新操作正常开始 |
| 解锁 sheet 关闭重开，旧 GET 关闭新弹层或弹旧错误 | `7727872` / `183da4b`：open/close/refresh/pay 共用操作 owner，旧 success/catch 不发布 |
| 历史列表库存延续覆盖后来的选择，同选择两读取互相回退 | `d271b42` / `183da4b`：list 与用户选择每次读取共用单调 owner；旧 cache/display/error/toast/finally 全失效，当前失败正确复位 |
| loading 中开始支付，旧权益 GET 覆盖 paying 或关闭 sheet | `183da4b`：支付开始接管新 owner；实际组件与真实 handler 验证保持 paying/visible，重复点击不再下单 |
| 原生支付 await 后失效页面继续旧确认读取 | `7727872`：两路径返回后重新验证上下文，不再发旧 waitForPaidOrder；已发出的真实支付不取消 |

早期 `e2cbc6f`、`aeb092d` 以及额度冻结草稿的失败均已进入以上修复回合。`b9fabac` 的审核 gate 过度取消也在 `991414c` 修正。旧诊断中对错误行为的断言只作为复现证据，不作为最终通过依据。

## 最终验证

以下由 Codex 在最终业务提交 `183da4b` 的小程序目录执行，实际进程退出码均为 0：

| 命令/验收 | 结果 |
| --- | --- |
| `bun test` | 505 pass / 0 fail，4491 assertions，35 files |
| `bun run type-check` | 通过 |
| `bun run check:boundaries` | 通过 |
| `bun run build:h5` | 通过 |
| `MINI_REVIEW_SKIP=1 bun run build:mp-weixin` | 通过；组件注册完整；产物 API `https://match.oryjk.cn/api/v1` |
| `git diff --check`、版本/文件范围检查 | 通过；manifest/generated 仍为 1.0.64 / 10064；未提交 env/密钥/证书/生成产物 |

固定提交的独立验收直接执行生产 boards/composable/handler；页面恢复场景使用真实 session 与 Vue watcher，订阅/支付场景在外部 API、会话上下文、时钟或轮询 effect 边界注入可控响应：

- 首页/洞察 `991414c`：18 项独立验收，134 assertions；后续没有修改这两页业务。
- 看板 `183da4b`：26 项独立验收，245 assertions，exit 0。覆盖真实 onShow/revision watch/页内 Tab 接线、两方向补载、ABA、cache/trend/reset、正常 poll 和玉昆首次 publish 后续库存。
- 订阅/解锁 `183da4b`：原 23 项和两项 RED 转 GREEN 合计 25 pass / 124 assertions；实际 TicketMatchIdSheet script 接真实支付 handler 另 6 pass / 36 assertions，exit 0。旧请求不污染新状态，合法支付/409/失败重试仍正常。

Codex 在最终 H5 构建上执行实际浏览器页面回归，4 组全部通过，exit 0：密码失败重试/成功/邮箱/退出重载游客；白名单账号显式 A→B；洞察延迟 me 只发一次 team-insights；首页公开内容在 me pending 时已显示、profile 为 0 次，me 完成后 profile 为 1 次。无微信认证请求或脚本错误。所有 API 均拦截为 mock，没有请求生产写接口；临时服务器已关闭。

## 交付边界与未验证项

- 微信开发者工具与真机尚未验证：无 token/过期 token 冷启动、首次未绑定、手动退出重开、显式重新登录、慢 code 后退出、绑定错误重试、跨 Tab 私有数据与轮询、切球队回切，以及真实原生支付/确认和端上视觉。这些不能由单元测试、离线构建或 mock H5 代替。
- 库存/回流仍以 `sale_start_at + 10 minutes` 起算，缺起售时间仍报错或跳过；邀请码、会员、订阅和支付规则保持。
- 后端、数据库、scraper、独立 H5 与报名系统没有修改。无需为本轮重启本地后端；**生产后端未部署、未重启**。
- 本轮没有 push、合并 main、小程序上传、版本登记或生产部署。代码 review 通过是本地交付结论。

# 公众号自助绑定交付与验证

日期：2026-10-16。依据：`docs/superpowers/specs/2026-10-16-wechat-official-bind-design.md`。

## 2026-10-20 补充：缺失关注记录导致的会员误锁

本节更新下文 10 月 16 日记录中的“不会创建关注记录”约定：当旧关注者没有收到过已接入的 subscribe 事件时，成功绑定仅设置用户 openid 会导致权益查询因找不到关注记录而返回 false。

- 绑定成功或同号重复绑定，在消耗绑定码的同一事务内插入缺失关注记录；通过已验证公众号消息获得的身份才可执行。`ON CONFLICT (open_id) DO NOTHING` 保留已有关注、邀请码历史与真实取关事件；不生成邀请码或增加现有关注次数，无效绑定不补记录。
- 独立 disposable localhost PostgreSQL 回归：首次绑定、同号绑定都补齐缺失记录，active/unsubscribe 旧行完整保留，无效码无新增；与现有历史继承/并发用例共 9 项通过。新增用例在修复前明确失败（关注记录为 None）。后端全量 634 项通过，112 项依赖外部环境跳过。
- 共享生产库只修复诊断中已绑定 V9 账号的 1 条缺失记录；使用近期绑定记录限定目标，写前确认候选恰 1 个，既有记录不覆盖，档位和到期时间不变。未输出/保存真实 openid、绑定码或 token。
- 本地 18092 后端于北京时间 20:04 重启；随后 `/auth/me` 返回 200、`official_wechat_bound=true`、`membership_benefits_enabled=true`、V9。生产后端未部署/重启，未执行其他迁移，小程序未改动；前端重进后读取新状态。
- `cargo check` 与两处变更 Rust 文件的格式检查通过。全库格式/边界扫描仍有其他模块既有问题，严格 Clippy 有 17 处既有报错（例如 Admin 用户适配器嵌套 if、AI 适配器缺 Default），未改无关模块。

以下为 10 月 16 日原交付记录，发布状态和副作用描述以对应日期为准。

## 交付状态

本地后端已重启（16:14:57 北京时间，18092，健康检查 `status=ok`）。生产未部署、未重启；小程序未上传。迁移仅在明确要求的验证事务中执行后回滚，未持久应用，也未写迁移登记表。没有真实用户/订单/会员修改，没有公众号发消息或真实支付。

发布需用户审查后依次执行：迁移 `football_insight_service_backend_rs/migrations/20261016150000_add_wechat_official_bind_codes.sql` → 部署后端 → 上传小程序。无数据回填、配置修改或种子 INSERT。

## 设计第 11 节自查

| 项 | 结果 | 依据 |
|---|---|---|
| 1. 六位码、字符集、归一化、十分钟、哈希、旧码失效、每小时五次 | 满足 | domain 纯函数；适配层 OsRng；SHA-256；发码事务锁用户后统计、作废、插入，冲突整笔回滚并重试；domain/HTTP 测试及 SQL 实测 |
| 2. 原子消耗与绑定、仅成功一次 | 满足 | issue/consume 均先锁用户；同一事务执行带 unused/expiry 条件的 UPDATE RETURNING 与 active 用户更新；重复 SQL 返回零行，用户更新失败回滚恢复码，Rust 并发 fake 用例通过 |
| 3. 失败统一与输错限制 | 满足 | 无效/过期/已用/inactive/存储失败共用定稿失败文案；openid 用 SHA-256 内存键，十分钟五次失败含在途预约，第六次不再校验；缺码、限流按定稿单独回复 |
| 4. 私密日志与 decision | 满足 | 绑定文本绕过原有含正文/身份日志，移除该链路原始错误日志；单线程捕获测试覆盖六类 decision，断言无码、openid、user_id |
| 5. 定稿文案、门控与既有指令 | 满足 | 独立字面量断言覆盖 4.6/4.7、明文/加密与多种空白输入；未识别仍断言不含测试比赛 ID；原 next-match 门控测试保留，未修改判定规则 |
| 6. 迁移与 SQL 回滚验证 | 满足 | 仅一表两索引，时间戳 20261016150000 大于原最新 20261016140000；9 条实际 SQL、38 项断言成功；TEMP 用户表隔离覆盖写，回滚后表/索引/序列均不存在，真实用户快照相同 |
| 7. 可选 current-user 字段 | 满足 | DTO `Option<bool>` 且 None 不序列化，只在 `/auth/me` 补充；旧登录响应不增字段；查询失败省略字段并 warn；原 `has_wechat_binding` 不变 |
| 8. 小程序入口和工程边界 | 满足 | 登录、非审核态、未绑定才显示，缺字段仍显示；429 仅按 ApiRequestError.statusCode 判断；组件仅 props/emits，不 import api；不改 ALLOWLIST/manifest，不新增色值/渐变 |
| 9. 弹层完整与关注提示 | 满足 | FiBottomSheet、绝对到期倒计时、复制完整命令、过期重新获取、关注提示；切换账号/会话立即销毁旧码和旧请求；返回绑定流程刷新一次用户，无轮询 |
| 10. official openid 使用点与副作用 | 满足 | 排除明确禁读的 Android 目录后搜索整个仓库；运行时使用点及影响见下节，未发现额外推送、支付或后台联动 |

第 2 项真实多连接并发尚未执行，原因与已有证据边界见“未做测试”。不把 fake 并发测试等同于数据库并发实测。

## official_wechat_open_id 使用点与副作用

搜索使用 `rg -n official_wechat_open_id --glob '!football_insight_admin_android/**' --glob '!bun.lock' .`，没有读取或搜索 Android 目录。原有运行时消费者集中于：

- `auth/adapters/persistence/postgres_auth_repository.rs`：邀请码解析及邀请注册写入；各登录、绑定小程序、按 ID 查用户等路径根据公众号关注状态计算 `membership_benefits_enabled`。
- `auth/domain/user.rs`：会员有效等级沿用已有权益暂停规则。
- `match_id_unlock/adapters/persistence/postgres_official_account_match_id_access.rs`：公众号下一场门控按同一 openid 关联的任意 active 合格账号授权。
- 历史迁移、设计与支付测试 fixture 也有引用，没有其他运行时消费逻辑；本任务新增发码/消耗适配器和 current-user 绑定状态读取。

绑定或更换 openid 后，所关联公众号的关注状态会控制该账号会员权益；取关暂停，重新关注恢复既有规则。一个 openid 可以关联多个账号，取关影响其关联账号；门控继续“任一账号合格即授权”。覆盖后旧微信号失去该账号提供的门控授权，新微信号获得关联，但旧号如有其他合格账号仍可授权。

不会改会员档位/到期时间、订单、推荐关系、小程序 openid 或 JWT 身份，也不会创建关注记录、发主动消息或改写邀请历史。用户需留意取关暂停；成功回复及弹层均明确提示。

## 实现与设计差异

1. 实际 API 文档在 `football_insight_service_backend_rs/docs/api.md`，按实际位置更新。其他设计目标文件均存在；未照抄旧行号。
2. 过期比较用 `statement_timestamp()`，避免等待用户锁后仍用事务起始 NOW() 放行已经过期的码；used_at 仍使用 NOW()。SQL 实测覆盖到期边界。
3. 消耗前先只读定位用户，再锁 active 用户，再原子消耗码，与发码统一锁顺序以防死锁；码和用户更新仍在同一事务。
4. `official_wechat_bound` 通过 current-user 用例新增只读查询补充 DTO，未给通用 AuthUser 加字段，保留其他登录响应；读失败省略可选字段，避免改变旧接口错误行为。
5. 原 `onShow → loadUser → ensureSessionReady` 存在 60 秒缓存，无法保证绑定后立即返回刷新。仅从已获取绑定码流程返回时绕过缓存刷新一次，不新增轮询；账号切换与迟到响应由原会话 revision 及流程所有者隔离。
6. 安全随机源作为 outbound port 注入，OS 随机操作在 security adapter，domain 仅做纯字符映射；随机源失败为 typed error，不进入发码存储。

## TDD 证据

- domain 测试先因缺少 official_bind 模块失败；新增 HTTP 测试先因缺少 handler、DTO 字段失败。
- webhook 绑定测试先因原逻辑不处理绑定指令返回 None 而失败；4.7 独立字面量断言先与旧文案不一致而失败。
- 熵失败测试先因缺少 generator port 编译失败；实现后验证 repository 调用为零。
- 小程序 helpers/流程先补失败测试再实现；包含旧账号迟到成功/失败/finally、过期复制、审核态、429 与绑定后隐藏；缓存返回问题另有真实 appSession 回归：RED 10 pass / 3 fail（预期 /me 两次实际一次），GREEN 流程 14 pass；不改全局 appSession/accountEffects。

## SQL 实际验证

脚本与原始结果留在本会话忽略目录 `.tmp/wechat-official-bind/verify_sql.py`、`verify_sql_result.json`，不提交。运行方式为在 `sina_csl_scraper` 下执行 `uv run python ../.tmp/wechat-official-bind/verify_sql.py`。使用 psycopg，从后端 `.env` 静默读取 DSN，未输出连接串或实际用户标识。

事务执行 `BEGIN` → `SET LOCAL lock_timeout='2s'`（另设 statement_timeout=3s）→ 新迁移 → 已存在用户 ID 的模拟哈希码 → 实际 SQL → 无条件 `ROLLBACK`。新表 FK 仍指向真实 public 用户，但覆盖 UPDATE 的 `f_i_users` 由 TEMP 同名表遮蔽，且脚本在写前断言解析对象为 TEMP。真实用户仅读取并比较绑定值和 updated_at 快照。

实际输出提要（9 条源 SQL 的 SHA-256 与实现逐条一致，38/38 断言为 true）：

```json
{"passed":true,"query_count":9,"case_count":38,"all_cases_passed":true,"rollback":true,"public_objects_absent_after_rollback":true,"real_user_unchanged":true,"no_fixtures_remain":true}
```

覆盖：迁移表/索引/序列/FK；active 锁；每小时零/五次及精确边界；仅存哈希；作废旧码；首次/重复/过期/到期边界消耗；同 openid 消耗；覆盖 TEMP 用户；inactive 不消费且不更新；模拟用户写入失败后回滚恢复码；唯一冲突回滚保留旧码；current-user 新查询的 NULL/已绑定/无用户/inactive 分支；真实用户内外快照及回滚无残留。

迁移 SHA-256：`c58de8215e888931fdbe2ee360fbe9b529bc5f696d5aef88ed982e43811fef54`。

## 未做测试与限制

- 不运行现有 65 项需持久 fixture 的数据库集成测试：本地连接生产，禁止修改真实数据或留下测试数据。新增 SQL 已按上节实际验证，不以未连库单测替代 SQL 验证。
- 未对新迁移做跨连接并发实测：其他连接看不到外层尚未提交的 DDL；为遵守不持久执行迁移，验证了实际条件 UPDATE 的连续重复消费、事务回滚和锁顺序，并以 Rust 并发 fake 测试补充用例。生产数据库并发仍待隔离环境或用户批准迁移后的专项测试。
- 两项进程全局 tracing 捕获测试在普通全量套件中标为 ignored，另用单线程独立执行，避免其他并发测试日志污染；没有漏跑。
- 未携带真实用户 JWT 发码、调用真实绑定 webhook 或往公众号发消息，未真实支付；不使用生产账号验证写入。仅在重启后的本地服务无 JWT 调用新接口，确认 401/no-store（不进入 repository）。
- 未做微信真机/H5 实际点击与屏幕视觉验收。单测与双端编译不能代替真机体验；没有上传小程序或部署生产。

## 验证命令与真实结论

后端命令执行于 `football_insight_service_backend_rs`，小程序命令执行于 `football_insight_mini`。均为真实执行，退出码 0：

| 命令 | 实际输出结论 |
|---|---|
| `./deploy/local-test/up.sh` | 编译成功；systemd `active`；健康 JSON `status=ok`，2026-10-16T08:14:57Z，端口 18092；现有 orphan 容器提示，未清理无关容器 |
| `cargo test` | 33 suites 合计 496 passed / 0 failed / 67 ignored；现有 mini_review_status.rs:245 unused Result warning |
| `cargo check` | `Finished dev profile`，退出 0 |
| `cargo build` | `Finished dev profile`，退出 0；后续最终 up.sh 也重新编译成功 |
| `cargo clippy` | 退出 0；16 条既有 lint 警告仍存在，本任务新增文件无命中 |
| `cargo test --lib official_bind_logs -- --ignored --test-threads=1` | 1 passed / 0 failed |
| `cargo test --lib next_match_decision_logs -- --ignored --test-threads=1` | 1 passed / 0 failed |
| 本地 POST 新接口，无 JWT | `HTTP/1.1 401 Unauthorized`、`cache-control: no-store`、`not logged in`；未触发存储 |
| `bun test` | 733 pass / 0 fail（64 files）；其中流程 14 项含真实 appSession 60 秒缓存与切账号迟到响应回归 |
| `bun run type-check` | vue-tsc --noEmit，退出 0 |
| `bun run check:boundaries` | 退出 0，依赖边界检查通过 |
| `bun run verify:mp-components` | 退出 0，全部小程序组件已注册 |

补充：最终返回刷新接线后的 H5 与 MINI_REVIEW_SKIP=1 离线微信构建均退出 0，组件注册与生产 API 静态检查通过；未申请版本、未上传。Browserslist 旧数据提示保留。

后端完整 stdout/stderr 位于会话目录 `.tmp/wechat-official-bind/` 的 `cargo-test.log`、`cargo-check.log`、`cargo-build.log`、`cargo-clippy.log`、`bind-logs.log`、`gate-logs.log`；SQL 完整逐项布尔输出为同目录 `verify_sql_result.json`。小程序原始输出为同目录 bun-test.log、bun-type-check.log、bun-boundaries.log、bun-components.log。这些 scratch 记录不提交。

## 小程序跨端说明

弹层沿用 FiBottomSheet 的 `@tap`、遮罩关闭与 H5/微信触摸处理；fit-content 模式复用其 view + overflow-y:auto，而不是默认 scroll-view 分支，不引入双绑 click/tap。账户行在 UserAccountInfoPanel 内用 @tap 发出 bind-official，原邮箱 @click 不变；按钮沿用 uni button 默认 hover-class，禁用态阻断重复请求/过期复制。page-meta 在弹层打开时锁背景滚动，并纳入页面已有 hasOpenSheet，避免游客登录浮层覆盖。计时器在 onHide 停止、onShow 按绝对到期时间重算，不依赖微信后台 setInterval；返回仅刷新一次当前用户，无网络轮询。

新弹层全部使用现有 fi-color、fi-space、fi-font、fi-leading、fi-weight、fi-radius、fi-border 现有 token（如 --fi-color-text-secondary、--fi-space-16、--fi-font-26、--fi-leading-relaxed）；账户入口复用 40rpx 的既有行图标。未增加 primitive、hex 或渐变。

## 改动文件

### 后端与共用文档

- 修改：`CHANGELOG.md`
- 修改：`FEATURES.md`
- 修改：`football_insight_service_backend_rs/docs/api.md`
- 修改：`football_insight_service_backend_rs/src/auth/adapters/persistence/mod.rs`
- 修改：`football_insight_service_backend_rs/src/auth/adapters/persistence/postgres_auth_repository.rs`
- 修改：`football_insight_service_backend_rs/src/auth/adapters/security/mod.rs`
- 修改：`football_insight_service_backend_rs/src/auth/adapters/web/dto.rs`
- 修改：`football_insight_service_backend_rs/src/auth/adapters/web/handlers.rs`
- 修改：`football_insight_service_backend_rs/src/auth/adapters/web/mod.rs`
- 修改：`football_insight_service_backend_rs/src/auth/adapters/web/wechat_handlers.rs`
- 修改：`football_insight_service_backend_rs/src/auth/application/get_current_user.rs`
- 修改：`football_insight_service_backend_rs/src/auth/application/handle_wechat_webhook.rs`
- 修改：`football_insight_service_backend_rs/src/auth/application/mod.rs`
- 修改：`football_insight_service_backend_rs/src/auth/bootstrap.rs`
- 修改：`football_insight_service_backend_rs/src/auth/domain/mod.rs`
- 修改：`football_insight_service_backend_rs/src/auth/ports/auth_repository.rs`
- 修改：`football_insight_service_backend_rs/src/auth/ports/mod.rs`
- 新增：`docs/wechat-official-bind-validation.md`
- 新增：`football_insight_service_backend_rs/migrations/20261016150000_add_wechat_official_bind_codes.sql`
- 新增：`football_insight_service_backend_rs/src/auth/adapters/persistence/postgres_official_bind_repository.rs`
- 新增：`football_insight_service_backend_rs/src/auth/adapters/security/official_bind_code_generator.rs`
- 新增：`football_insight_service_backend_rs/src/auth/adapters/web/official_bind_handlers.rs`
- 新增：`football_insight_service_backend_rs/src/auth/application/official_bind.rs`
- 新增：`football_insight_service_backend_rs/src/auth/domain/official_bind.rs`
- 新增：`football_insight_service_backend_rs/src/auth/ports/official_bind_code_generator.rs`
- 新增：`football_insight_service_backend_rs/src/auth/ports/official_bind_repository.rs`
- 新增：`football_insight_service_backend_rs/tests/official_bind.rs`

### 小程序

- 修改：`football_insight_mini/src/api/auth.ts`
- 修改：`football_insight_mini/src/pages/user/components/UserAccountInfoPanel.vue`
- 修改：`football_insight_mini/src/pages/user/helpers.test.ts`
- 修改：`football_insight_mini/src/pages/user/helpers.ts`
- 修改：`football_insight_mini/src/pages/user/index.vue`
- 修改：`football_insight_mini/src/types/auth.ts`
- 新增：`football_insight_mini/src/pages/user/components/UserOfficialBindSheet.vue`
- 新增：`football_insight_mini/src/pages/user/officialBindFlow.test.ts`
- 新增：`football_insight_mini/src/pages/user/officialBindFlow.ts`

无删除文件。

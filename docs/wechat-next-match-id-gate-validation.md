# 公众号「下一场」比赛 ID 门控验证

日期：2026-10-16。设计：`docs/superpowers/specs/2026-10-16-wechat-next-match-id-gate-design.md`。
实现从 `2d4ea10` 开始；会话期间有独立的小程序提交/未提交工作，不纳入本次后端改动。

## 实现与状态

auth/domain 定义三态判定，auth/ports 定义身份端口；match_id_unlock/domain 复用有效等级、等级排序和门槛常量；该模块的 PostgreSQL adapter 只读一次身份、过期时间与解锁，app 构造后注入 auth/bootstrap。auth 的生产代码及测试均不反向引用 match_id_unlock。

门控只有 next_match_reply 一处：开关 → current-standard ID → 身份端口 → 回复。CASE WHEN match_id IS NULL THEN false 跳过解锁子查询。只有 Granted 调用原 next_match_id_message；其他判定使用定稿文案。身份/票务/配置原始错误不记录，避免包含标识符，每次判定只输出一行 next_match_id_reply decision=...。

**本地后端已重启**：最后一次 `./deploy/local-test/up.sh` 于北京时间 15:32:16 返回 active、health status=ok，端口 18092。
**生产后端未部署、未重启**。无迁移、无新增配置键、未修改配置值，无公众号发送或真实支付。本任务没有小程序改动或上传。

## 改动文件清单

后端路径相对 `football_insight_service_backend_rs/`：

新增：

- `src/auth/domain/official_account_match_id_access.rs`
- `src/auth/ports/official_account_match_id_access_port.rs`
- `src/match_id_unlock/domain/official_account_access.rs`
- `src/match_id_unlock/adapters/persistence/postgres_official_account_match_id_access.rs`
- `tests/official_account_match_id_access.rs`

修改：

- `src/auth/domain/mod.rs`
- `src/auth/ports/mod.rs`
- `src/match_id_unlock/domain/mod.rs`
- `src/match_id_unlock/adapters/persistence/mod.rs`
- `src/auth/application/handle_wechat_webhook.rs`
- `src/auth/bootstrap.rs`
- `src/app.rs`
- `docs/api.md`

共享文档：根目录 `CHANGELOG.md`、`FEATURES.md`，以及新增本报告 `docs/wechat-next-match-id-gate-validation.md`。无删除文件。

## 命令归一化实际情况

现有 normalize_text_command 保持原样：先移除所有 Unicode 空白，再对字符转小写，然后映射数字。handle_message 之前的 trim 也保留。

- 下一场 → 下一场；下一场id/下一场ID/下一场 ID/下 一 场 ID 均归一为下一场id。
- 2 → 下一场；3 → 下一场id。带首尾空白或内部空白的对应形式也相同。
- 1 → 邀请码，不经过本门控；其他文本仍按原方式处理，没有增加模糊匹配。

两种文字命令和两个数字都进入同一个条件分支、调用同一个私有门控。明文与加密两种回复使用原 build_text_reply。

## 设计第 8 节逐项自查

| # | 结果 | 依据 |
|---|---|---|
| 1 单点与顺序 | 满足 | next_match_reply 单点；关闭不取比赛，比赛缺失/错误不查身份；别名矩阵覆盖 6 种输入、明文/加密 |
| 2 所有非授权路径无 ID | 满足 | 非会员、未识别、身份 Err 均断言回复 contains(测试ID)==false；关闭/配置错误/无比赛/取比赛失败断言两个候选 ID 不出现；解析失败仅解锁与普通非会员均拒绝并断言原始非数字 ID 不出现 |
| 3 fail closed | 满足 | 只有 Granted 分支拼入比赛 ID，Err 返回“暂时无法确认”定稿文案 |
| 4 缺省路径不查身份 | 满足 | disabled/config_error/missing/fetch_error 均计数断言身份调用为空；关闭/配置错误连比赛端口也不调用；回复逐字等于原 missing 且无“会员” |
| 5 复用会员规则 | 满足 | domain 调用 resolve_effective_membership_tier、membership_tier_rank、MATCH_ID_UNLOCK_MINIMUM_TIER；表驱动 14 条记录及 None，共 15 情形，涵盖 V1/V3/V4/V5、V6–V9、过期、恰好到期、未来到期、解锁及无效等级 |
| 6 无迁移/配置写入，SQL 已验证 | 满足 | 仅新增 SELECT；生产 BEGIN READ ONLY + lock_timeout2s，finally ROLLBACK，报告 writes_executed=0，无迁移/配置文件或表改动 |
| 7 单条身份查询 | 满足 | adapter 只有一个 query_as/fetch_optional；没有 known_match_id/fetch_all_matches 或新的赛程调用；已有开关读取及 current-standard 读取保留 |
| 8 日志不含比赛 ID | 满足 | 不记录 ID 或原始上游错误；日志测试用包含 ID 的错误哨兵，捕获日志无 ID |
| 9 定稿文案 | 满足 | 三段实现直接使用设计 2.1 文案；测试用独立字面量逐字断言并保留 with_product_intro，拒绝文案无 ¥5 |
| 10 无循环与错误分层 | 满足 | auth 只定义/消费自己的枚举和端口，match_id_unlock 单向引用 auth；app 装配 adapter；纯规则在 domain、编排在 application、SQL 在 adapter，无 web handler 改动 |
| 11 每次结构化判定 | 满足 | 一个 info 调用，decision 使用 % 格式；单独日志测试覆盖 granted/not_member/unrecognized/error/disabled/no_match，并额外覆盖配置/比赛读取错误；每次恰好一行判定，全部日志无测试 ID/openid |

独立只读 review 没有剩余阻塞或重要问题。原有 incoming/open_id 日志保留；新增的判定与错误日志没有标识符。

## TDD 与真实验证命令

- 领域测试先因缺少新枚举与函数编译失败，再实现纯规则。
- 接入端口、尚未替换旧回复时，拒绝矩阵实际失败：下一场 not_member 的 contains(ID) 为 true、预期 false，证明旧路径泄漏；随后接入门控修复。
- 结构化日志最初默认格式带引号，改用 %decision，保证 decision=not_member 等格式。并行日志捕获受 tracing 的全局调用点缓存干扰，已分离为显式单线程日志测试；功能矩阵仍参与普通全量测试，没有删除拒绝断言。

| 命令 | 实际结论 |
|---|---|
| `./deploy/local-test/up.sh` | exit 0；编译通过，active / health ok；既有 compose orphan/no-services-to-build 提示未处理无关容器 |
| `cargo test` | exit 0；32 suites，486 passed / 0 failed / 66 ignored；完整输出 `.tmp/wechat-next-match-gate/cargo-test.log` |
| `cargo test --lib next_match_decision_logs -- --ignored --test-threads=1` | exit 0；1 passed，399 filtered out；日志用例实际执行通过 |
| `cargo check` | exit 0；Finished dev profile，10.33s |
| 改动 Rust 文件 rustfmt check | 12 个文件，格式检查通过 |
| `git diff --check` | 通过（最终暂存范围另做 cached 检查） |
| `uv run python ../.tmp/wechat-next-match-gate/verify_sql.py`（scraper 目录） | exit 0；实际回滚事务证据如下 |

## 生产 SQL 真实回滚验证输出

脚本只放会话 scratch：`.tmp/wechat-next-match-gate/verify_sql.py`，结果 `sql-report.json`。用 psycopg 从后端 .env 静默读取 DATABASE_URL，未输出连接串或 openid；从真实 adapter 提取 ACCESS_SQL，转换占位符后执行原 SQL，不重写查询。

顺序 BEGIN READ ONLY → SET LOCAL lock_timeout='2s' → statement_timeout3s → 读取一个已有 active 用户的公众号 openid（不输出）→ 查询 → 不存在 openid 查询 → None 查询与 EXPLAIN ANALYZE → finally 无条件 ROLLBACK。数据库 read-only 状态也断言为 on，回滚后连接为 IDLE。

实际输出（不含用户 ID/openid）：

```json
{
  "query_sha256": "ff8f697f3ca0bf1aefcab7252eddd22458607646ce1256c77fc6cb9a0473c52e",
  "writes_executed": 0,
  "existing_openid_returns": {
    "membership_tier": "V9",
    "membership_expires_at": "2027-04-20T12:26:38.291681+00:00",
    "has_unlock_for_match": false
  },
  "nonexistent_openid_rows": 0,
  "none_match_id_has_unlock": false,
  "none_match_id_unlock_table_scans_executed": 0,
  "rollback_executed": true,
  "transaction_idle_after_rollback": true
}
```

第一次选中的真实账号到期时间为 NULL，也验证成功；最后一次优先选有到期时间的账号，得到上表证据。全过程零 DML、零测试数据，无需清理真实用户/订单/会员记录。

## 偏离、未做测试与限制

- 设计同时给出 domain 枚举位置的两种方案，本期选择 auth/domain 共用枚举，match_id_unlock 返回该类型，避免相互引用。设计文件路径均存在，无业务路径偏离。
- 旧 fetch 错误/开关错误日志改为静态 warn，防止错误文本携带 ID/openid；保留原有 incoming/open_id 日志，原缺省回复不变。这是落实日志隐私要求。
- 日志捕获用例标记 ignored 仅为隔离 tracing 缓存，已按上表显式执行通过；因此 66 ignored 中有 1 项已另行验证。其余 65 项数据库集成测试未对生产运行，避免不受控写入；本任务新增 SQL 已实际只读事务验证。
- 未向真实公众号发送消息，未对线上 webhook 发合成事件，没有真实支付/会员/订单写入；加密/明文编排通过现有 fake 回归，未做微信 5 秒端到端实测。新门控只有一次身份查询，没有额外赛程拉取。
- 未部署生产、未改配置开关。工作区中其他任务的小程序更改没有包含在本提交，未读取或修改 Android 项目。


## 2026-10-20：首次免费机会与存量补记

本节为当前规则，前文保留 10 月 16 日会员门控的历史验证。负责人明确要求“从来没有获取过”才有一次免费机会，且现有存量用户均视为已经获取过。

- 迁移 `20261020160000_add_official_match_id_acquisitions.sql` 给全部现有账号设置 `official_match_id_received=true`，并合并历史关注者（含取关）、绑定账号和邀请码身份为 historical 获取记录。未来账号默认 false；记录不按比赛重置，不写单场购买解锁表。
- 会员、单场购买及首次免费获取均写入历史；公众号身份唯一键与账号标记保证次数持久化。获取、换绑及邀请码注册先取得同一事务 advisory lock，再锁业务行；所有三种邀请码注册/重新激活都继承相关身份历史。
- 首次免费不要求绑定小程序，但绑定已有历史的账号不能再领取；已获取后重新关注、换比赛、自助换绑或重新注册都不返还机会。不同微信身份在脱离绑定后才分别首次获取，不建立永久跨微信身份归属关系。
- 无比赛、来源失败、身份失败、获取记录写入失败不返回免费 ID；写入回滚保留首次机会。ID 非正数或无法解析时不能使用首次机会。传输失败无法由被动回复确认，持久化提供最多一次发放语义。
- 回填仅随迁移执行一次；不能作为启动 SQL 重复运行。生产先运行标准迁移，再更新后端；新代码依赖该 schema，本地连接同一生产库时同样需要迁移才能实际使用新公众号/注册逻辑。

验证结果：

| 验证 | 结果 |
| --- | --- |
| TDD 旧行为 | 首次获取、换绑历史、注册继承、共享事务锁用例均先观察到预期失败，再修正通过 |
| `cargo test` | 634 passed，110 ignored；包含所有命令、明文/加密、同场/换场、来源及记录错误回归 |
| 独立 PostgreSQL 集成测试 | 7 passed，显式 localhost `_test` 数据库、独立随机 schema；覆盖迁移、存量无绑定账号、关注/取关、邀请码身份、16 路并发、写入回滚、会员过期、付费、换绑、三种注册/重新激活与共享事务锁 |
| 单独日志测试 | 1 passed；包含 first_acquisition 和写入错误，判定日志不含 ID/openid/原始错误 |
| 格式与构建 | 本次 Rust 文件 rustfmt 检查、`cargo check` 及后端二进制构建通过 |
| 架构边界 | 本次文件扫描通过；全库扫描仍有原有 Web-to-Web 引用及按错误字符串分支两类问题 |
| Clippy | `cargo clippy -- -D warnings` 因原有 17 项 lint 失败；如 admin_user 的 collapsible_if、DisabledAiChatPort/LogoutUseCase 的 new_without_default、既有路由/回复函数的 too_many_arguments 等，未在本任务扩展修复 |
| 本地服务 | 最新业务代码已编译并重启 `football-insight-local-backend.service`，18092 `/api/health` 返回 ok |
| 只读审查 | 获取/换绑同步和注册继承问题已修复复核，无剩余实质性发现 |

开发验证完成时，生产迁移及补记尚未执行，生产后端未部署/未重启；当时未向真实公众号发消息或发合成 webhook，未改真实用户、会员、订单或支付数据。后续生产迁移和部署结果见下一节。


## 2026-10-20 生产上线记录

- main 已从 `2715515` 快进合并至 `061c46b` 并推送；包含首次获取规则及已验证绑定时补齐缺失关注记录的修复。
- 合并前后 `cargo test` 均为 634 passed / 112 ignored，最新显式独立 PostgreSQL 测试为 9 passed。未在生产调用合成公众号消息或替真实用户消耗免费机会。
- 迁移前核对 72 条已应用迁移，SHA-384 全部一致；相关表（用户、关注者、邀请码、迁移记账）备份在 peiqian `/root/backups/football-insight/official-id-061c46b-20261020T121842Z/pre-migration.dump`，权限 0600，大小 5,065,684 字节。
- 在 local233 构建发布镜像，传至 peiqian。两端镜像配置 ID 存在差异，但主程序、迁移程序及新增迁移 SQL 的 SHA-256 完全相同；使用唯一发布标签 `football-insight-service-backend-rs:061c46b-95ae416b` 防止同名标签混淆。
- 在 peiqian 使用该镜像中的标准 `/usr/local/bin/run_migrations`（同仓库入口，SQLx 记账）执行迁移，输出 `migrations applied successfully`。`_sqlx_migrations` 的 `20261020160000` 为 success=true；969/969 个存量账号已标记，historical 身份 557 个，已知身份未补记数 0，新账号列默认值 false。
- 迁移后切换生产容器：host 网络、原运行环境文件、PORT=8092、原日志卷；容器 healthy、RestartCount=0、启动 ERROR=0。运行中主程序 SHA-256 为 `0472fefe858d78d9f02101c35bcb65727539c857f950dda740ef36e5a18fd421`，与构建端一致。
- 线上 `https://match.oryjk.cn/api/health` 和 `/api/v1/ticket-watch/current-board` 均为 200；公众号查询开关为 true。local233 后端也已重新启动且 18092 健康。
- 原镜像 `football-insight-service-backend-rs:3925206` 和原容器 `football-insight-service-backend-rs-rollback-3925206-20261020T1225` 保留。如需回滚，停止/移除新容器，将原容器重命名回 `football-insight-service-backend-rs` 并启动；新增 schema 和已获取历史保留，不能重跑回填或删除历史记录重置免费机会。
- 本次未修改 jd/Nginx、monitor 容器或生产环境文件，未上传小程序。并行任务中的其他未提交源码未纳入本次生产镜像或上线记录提交。

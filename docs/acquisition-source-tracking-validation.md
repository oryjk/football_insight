# 用户来源追踪验证报告

日期：2026-10-18。设计：`superpowers/specs/2026-10-18-acquisition-source-tracking-design.md`；实施计划：`superpowers/plans/2026-10-18-acquisition-source-tracking.md`。

实现与验证完成，后端/小程序分开提交main。**本地后端已重启；生产后端未部署/未重启；未执行生产迁移、未写生产用户/订单数据、未上传小程序。** 生产不会因本次代码推送立即产生来源记录。

## 已实现

- 冷启动在bootstrapSession前采集，热启动也能首次采集；utils解析/存储，不在页面实现。合法直接src优先，scene支持`s=douyin&c=0418a`及URL编码，解码后最多32 ASCII字符；不拼接两个入口的内容，不接受重复键。
- src必需且合法才锁定首次来源；cid独立可选，非法丢弃；数值landing_scene只接受1–65535整数，不存scene原始文本。src32/cid64上限、限定字符在两端验证，DB另有CHECK。
- 来源存储独立key，后续入口/退出/会话切换不会覆盖。会话层装配可选acquisition，API只序列化并用既有request；无来源仍发原请求体。未改认证意图/revision/权限与错误处理。
- Register/mini-bind/wechat-bind用例仅在实际INSERT新账号时调度来源写入。仓库显式携带newly_created标记，6个INSERT分支true、6个重新激活UPDATE分支false；旧账号绑定/普通登录不补记。客户端user_id无用，写入身份来自服务端新建结果。
- 记录同步返回、任务独立执行，128并发、250ms超时、panic捕获、lazy专用2连接池/100ms获取超时。表缺失/写入失败/超时/饱和只warn，不影响原注册结果，不等待业务连接。
- 新表user_id PK/FK、INSERT ON CONFLICT DO NOTHING，不动f_i_users。统计先给只读SQL，公众号关注来源不做。

## 六项验收自查

|设计验收|结果|证据|
|---|---|---|
|1 douyin/test1首次注册有一条记录|满足分段自动化|真实session→API请求断言元数据；三个注册/绑定用例核对创建结果ID；真实PostgreSQL执行生产INSERT常量，首条INSERT 0 1|
|2 再登录/别的来源不改变|满足|utils首次来源覆盖拒绝；真实session退出后换入口仍原来源；旧账号绑定与reactivation负向测试；真实重复SQL INSERT 0 0，原douyin/test1/1047保持|
|3 缺参数/非法/老客户端正常|满足|纯函数表驱动、DTO缺失/错类型/脚本/超长校验；无来源原API体保持；原认证与会话测试通过|
|4 来源失败注册成功|满足|FailingSource返回table missing，用例仍返回原账号/JWT；writer错误/panic/超时/饱和不向认证传播，容量会释放|
|5 按src/cid统计注册|满足|实际SQL得到douyin/test1=2、douyin/test2=1、xhs/test2=1，内容未混合；多笔支付去重、成熟分母/边界验证|
|6 后端/小程序验证|满足|最终命令和真实结论如下，忽略项单独列明|

验收1没有冒用生产微信或真实注册：用实际应用函数、请求/仓库边界及隔离SQL分段验证；完整真实微信注册/运营码真机扫码需要发布后由用户验收。

## 验证命令与真实结论

所有shell命令均前缀rtk。日志在会话scratch `.tmp/acquisition-source-20261018/`。

|命令|最终实际输出结论|
|---|---|
|`./deploy/local-test/up.sh`|exit0；编译、用户systemd重启active；健康接口status=ok（2026-10-18T04:32:14Z），端口18092；原orphan container提示未擅自清理|
|后端`cargo test`|525 passed / 0 failed / 72 ignored，34 suites|
|后端`cargo check`|Finished dev，exit0，2.93s|
|后端`cargo build`|Finished dev，exit0，0.21s|
|后端`cargo clippy`|exit0，0 errors、17项存量lib warnings；本次三处collapsible_if已修正。测试另有mini_review_status.rs既有unused Result警告|
|7个新增Rust文件`rustfmt --edition 2024 --config skip_children=true --check`|通过；未全仓格式化|
|小程序`bun test`|874 pass / 0 fail，5817 expect calls，77 files，13.36s|
|`bun run type-check`|vue-tsc --noEmit，exit0|
|`bun run check:boundaries`|依赖边界检查通过，未改ALLOWLIST|
|`bun run verify:mp-components`|All mini-program custom components are registered|
|`bun run build:h5`|Build complete，exit0|
|`MINI_REVIEW_SKIP=1 bun run build:mp-weixin`|Build complete，注册/API产物检查通过；保留versionName1.0.98、versionCode10098，不申请版本号/不上传|
|隔离PostgreSQL实际迁移/INSERT/统计|所有断言通过、ROLLBACK、rollback_cleanup_ok=t；容器已停止删除|
|`git diff --check`|通过|

构建仅现有uni-app/Browserslist更新提示。没有运行全仓严格-D warnings或修复无关旧警告。

## TDD与复审记录

- mini parser初始2通过/3失败，随后5通过；请求来源初始16通过/1失败，再转绿。边界检查发现API不能直接导入新增utils，因此装配移到appSession，未增例外；真实session+request用例再次先失败后通过，旧会话用例共73项通过。
- 后端domain初始1失败再转绿；DTO及writer初始1通过/3失败；三个创建用例初始3失败再转绿。当前10项来源相关后端用例覆盖正/负分支。
- writer测试首次等待单个yield不足以让128任务全部开始，改为有界等待实际开始/容量释放，避免调度时序假设；不改变生产超时或容量。
- SQL先用无写入stub复现first acquisition missing，再用空统计stub复现wrong group count；最终真实INSERT/分组SQL通过。每次失败也执行ROLLBACK，未留下数据。
- 格式整理曾误移除新模块声明/导入，被编译拦住；恢复后重新本地重启并跑完整验证。只整理本任务新增代码，不处理旧全仓格式。
- 独立只读全量复审未发现可操作的正确性、安全或兼容性问题，未连接数据库/做生产操作。

## SQL真实隔离验证输出

用已有`postgres:16-alpine`镜像启动一次性容器`football-acquisition-test-20261018`，**network=none**，数据目录tmpfs，未读取生产.env。所有基础表、5个模拟用户及新迁移均在同一个事务内创建；只执行本任务的INSERT常量和真实docs/sql文件。`SET LOCAL lock_timeout='2s'`；SQL遇错也继续至无条件ROLLBACK，验证器检查错误并失败。

最终主要输出：

```text
首次：INSERT 0 1
重复：INSERT 0 0
src     cid    registered d1_eligible d1_plus d7_eligible d7_plus paid
 douyin test1  2          2           2       2           1       1
 douyin test2  1          1           0       1           0       0
 xhs    test2  1          0           0       0           0       0
ROLLBACK
rollback_cleanup_ok = t
isolated PostgreSQL migration/insert/report checks PASS
```

同时验证：源字段CHECK拒绝脚本；来源主键不覆盖；同渠道不同cid分组；24/168小时恰好边界；未成熟用户分母0；没有登录不算回访；两笔已付/已退款订单只算一人；pending和未来paid_at不算。迁移表及其约束/索引随ROLLBACK消失；基础测试表也被回滚，容器最后stop并--rm清除。

## 统计决定与设计偏差

设计把scene编码、统计方式和回访精确定义交给实现方决定，采用：短参数s/c（无硬编码短码表）、SQL先交付、D1+/D7+累计回访。详细定义见[接入与统计](acquisition-source-tracking.md)。现有record_user_login只保存last_login_at，不能恢复严格第1/7自然日是否来过；第8天回来也算D1+与D7+，不能把这些数字标为严格日留存。该限制不通过臆造历史或改登录机制掩盖。

合法src是本期形成来源的必要条件，只有cid/数值scene不会占用首次来源。热启动采集是冷启动的补充，仍只写首次合法值。landing_scene不保存原始文本，避免把第三方内容或身份带入库。

原设计的文件路径与当前实现一致；register_with_invite.rs还承载无邀请码注册，新增处理覆盖两条路径。为避免原API工具白名单例外，元数据在已有会话动作发请求前装配，而非API内读取存储。没有改变原会话意图、登录校验或业务权限。

## 未做与上线前事项

- 72项既有ignored测试未启用：它们需要可丢弃数据库、外部服务或生产写入，不满足当前生产库隔离要求。本任务SQL单独在一次性容器验证，不使用生产真实账号。
- 未进行真实微信登录/注册、真机扫码、真实支付、公众号消息；无微信开发者工具/真机环境。H5和微信编译通过不能替代真机。
- 不做公众号subscribe来源（设计明确第二阶段）、后台页面或新后台接口、历史来源回填、严格逐日留存历史。
- 小程序源码/文档没有找到可确认覆盖来源统计的隐私政策文本。负责人需核对微信后台及现行政策，必要时补充渠道统计用途；本任务未自行改隐私文案。
- 上线顺序：审查→负责人在peiqian执行`20261018120000_add_user_acquisitions.sql`→部署后端→上传小程序→运营带参数入口验收→只读运行统计SQL。**本任务上述生产操作一项未执行。**

## 改动文件

### 后端

新增：
- `football_insight_service_backend_rs/migrations/20261018120000_add_user_acquisitions.sql`
- `football_insight_service_backend_rs/src/auth/adapters/persistence/postgres_acquisition_repository.rs`
- `football_insight_service_backend_rs/src/auth/adapters/web/acquisition_dto.rs`
- `football_insight_service_backend_rs/src/auth/application/acquisition_test_support.rs`
- `football_insight_service_backend_rs/src/auth/application/record_acquisition.rs`
- `football_insight_service_backend_rs/src/auth/domain/acquisition.rs`
- `football_insight_service_backend_rs/src/auth/domain/registration.rs`
- `football_insight_service_backend_rs/src/auth/ports/acquisition_repository.rs`

修改：
- `football_insight_service_backend_rs/docs/api.md`
- `football_insight_service_backend_rs/src/auth/adapters/persistence/mod.rs`
- `football_insight_service_backend_rs/src/auth/adapters/persistence/postgres_auth_repository.rs`
- `football_insight_service_backend_rs/src/auth/adapters/web/handlers.rs`
- `football_insight_service_backend_rs/src/auth/adapters/web/mod.rs`
- `football_insight_service_backend_rs/src/auth/application/bind_wechat_account.rs`
- `football_insight_service_backend_rs/src/auth/application/bind_wechat_mini_program_account.rs`
- `football_insight_service_backend_rs/src/auth/application/mod.rs`
- `football_insight_service_backend_rs/src/auth/application/register_with_invite.rs`
- `football_insight_service_backend_rs/src/auth/bootstrap.rs`
- `football_insight_service_backend_rs/src/auth/domain/mod.rs`
- `football_insight_service_backend_rs/src/auth/ports/auth_repository.rs`
- `football_insight_service_backend_rs/src/auth/ports/mod.rs`

### 小程序

新增：
- `football_insight_mini/src/utils/acquisitionSource.test.ts`
- `football_insight_mini/src/utils/acquisitionSource.ts`

修改：
- `football_insight_mini/src/App.vue`
- `football_insight_mini/src/api/auth.test.ts`
- `football_insight_mini/src/api/auth.ts`
- `football_insight_mini/src/stores/appSession.ts`
- `football_insight_mini/src/types/auth.ts`

### 共享文档

新增：
- `docs/acquisition-source-tracking-validation.md`（本报告）
- `docs/acquisition-source-tracking.md`
- `docs/sql/user-acquisition-summary.sql`
- `docs/superpowers/plans/2026-10-18-acquisition-source-tracking.md`

修改：
- `docs/superpowers/specs/2026-10-18-acquisition-source-tracking-design.md`

另外按各提交补充`FEATURES.md`和`CHANGELOG.md`；删除文件：无。


## 分享来源参数统一（2026-10-19）

普通页面转发统一追加 `src=share&cid=<分享点>`。cid 为 `home`、`matches`、`rankings`、`insights`、`team_board`、`support`、`topics`、`topic_detail`、`ticket_current`、`ticket_history`、`swap`。榜单已打开的积分榜海报同样保留原 `openPoster/table` 参数后追加 `rankings`；赛季海报原有 `recap<赛季>` 来源保持不变。换座位分享仅含公共页面路径及来源，无用户或座位信息。

首页、回流看板、话题详情增加朋友圈分享。朋友圈仅传 query，话题保留合法文章 `id`；标题沿用页面普通转发标题。缺失/非法文章 ID 的普通分享仍回话题列表，来源标识分享出口 `topic_detail`；朋友圈留在原详情页并省略无效 ID。未增加其他页面的朋友圈入口。

查看效果时，在 [来源汇总 SQL](sql/user-acquisition-summary.sql) 最终的 `FROM cohort AS c` 与 `GROUP BY` 之间加入 `WHERE c.src = 'share'`，按 `cid` 比较 `registered_users`、`d1_plus_return_users`、`d7_plus_return_users` 和 `paid_users`。注册时间范围仍在 `params` 设置，不要将累计回访标作严格自然日留存。该 SQL 只读，不新增表或记录转发/点击次数。

边界：度量对象是实际新注册用户；老用户转发给已注册朋友不产生新来源记录，整体传播效果会被有意低估。曾无来源打开且尚未注册的用户，后续从转发热启动时可首次捕获来源；已有首次合法来源始终不覆盖。朋友圈单页模式进入完整版才可能由启动参数采集，微信不保证所有场景透传，不增加额外兼容逻辑。

自动化覆盖工具函数的无/有 query、已有 src/cid（包括空值与编码键）、非法 cid、空路径、hash、64 字符边界；执行真实页面注册的分享回调，校验各分支 cid/原业务参数/赛季海报来源，串联 `parseAcquisitionLaunch` 和首次热启动 capture。微信真机或开发者工具的卡片打开、`getLaunchOptionsSync().query`、朋友圈进入完整版和热启动仍待人工验收，不能以纯函数测试代替。未进行微信上传或生产部署。


本次两项功能集成后的最终验证：`bun test` 968 通过 / 0 失败（90 文件）；`bun run type-check`、`bun run check:boundaries` 均通过；`MINI_REVIEW_SKIP=1 bun run build:mp-weixin` 与 `bun run build:h5` 均构建成功，小程序组件注册和生产 API 产物校验通过，保留登记库版本 1.0.99。独立审查确认来源功能符合设计；海报同场轮询清图问题已修复并回归。Canvas 使用占位二维码与模拟数据做浏览器布局检查，不能当作微信扫码验收。


### 2026-10-19 发布更新

`97b025f` 已合并推送 main；生产 peiqian 后端已部署并重启且 healthy，小程序 1.0.99 已从同一干净 main 经统一入口上传 robot 2。登记库复用审核中版本，未手改版本或审核状态。生产 `f_i_user_acquisitions` 与 `f_i_mini_review_statuses` 已确认存在，本次不运行迁移、不写测试用户。来源解析自动化及合并后全套 968 项测试通过；微信卡片/朋友圈参数透传与实际新注册来源仍待真机验收。部署证据及回滚见 [海报验证记录](reflux-board-share-poster-validation.md)。

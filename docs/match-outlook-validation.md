# 赛前数据倾向验证报告

日期：2026-10-18。设计：`superpowers/specs/2026-10-18-match-outlook-design.md`；计划：`superpowers/plans/2026-10-18-match-outlook.md`；接口：`../football_insight_service_backend_rs/docs/match-outlook-api.md`。

**本地后端已重启；生产后端未部署/未重启；生产迁移、历史回填及预测刷新均未执行；未上传小程序。** 所有数据库写测试在专用一次性 PostgreSQL 容器进行，没有读取项目 `.env`，没有对生产创建测试数据、支付或发送消息。

## 实现范围

Python 用比分、球队 ID、北京时间与中立场地标记预计算 Elo V2；3888 组参数以 τ=1 滚动选择，最终轨迹自适应校准。首次球队（包括 2014）使用 newstart，1500 只作赛季回归锚点，已按负责人更新后的设计实现。

新增历史比分表、预测表与冻结触发器。核心同步提交后才在独立连接生成/计分，附加流程失败不改变核心成功结果。Rust 按 domain/ports/application/adapters 读取存量概率与平均分，不计算概率。小程序提供免费倾向面板、公开计分汇总与首页轻量入口，遵守登录和审核态门控，不改会员或付费行为。

## 设计十项验收

| 项目 | 自查 | 依据 |
|---|---|---|
| 1 复现回测及 2026 参数 | 满足 | 同一研究快照：2014–2025 共 2802 场、2026 已完赛 221 场；非中立样本 1487。生产 RPS **0.19877789036730545**；原参考约 0.19880741475072394，差 **-0.00002952438**，远小于 0.0005。参数 `(10,60,150,.7,1,.8,1450)` 完全一致。可选验收测试实际执行通过，不补造历史公开记录 |
| 2 无前视 | 满足 | 改写后半程比分不改变此前预测；同一开赛时刻先预测整批、再更新，改变同刻另一场结果也不影响预测；选参排除目标与未来赛季，未来待赛比分不进入评分更新 |
| 3 冻结 | 满足 | 数据库墙钟同时校验首次 INSERT、旧记录和新开赛时间；过时生成拒绝，旧冻结记录冲突更新拒绝。直接 SQL 改冻结概率返回 rowcount=0，概率、generated_at、kickoff_at 保持；改期只加标记 |
| 4 一次计分 | 满足 | 隔离库手算 `.5/.25/.25` 主胜 RPS=.15625；行锁及 scored_at 条件保护，只给已冻结、有效完赛记录计分。重复执行、赛后改比分及直接修改已计分记录均不覆盖原值 |
| 5 失败隔离 | 满足 | fake 中 after_sync 在 commit 后抛错，核心仍成功且已提交；真实接线只传入独立连接的附加任务，不改原 team_insights 计算与提交 |
| 6 数据校验 | 满足 | 全年数量、2020/2021/2022 的 160/176/306、重复 ID、缺失/非法比分、未完赛状态、非法时间、中立标记都有校验；完整下载校验后才可连接写库。缓存续跑和校验失败不连写库的测试通过 |
| 7 概率合法 | 满足 | 极端分差输入、归一化和非负断言通过；真实回测全部概率有效；数据库另设上下界与总和 CHECK，模型/基线都适用 |
| 8 两个后端接口 | 满足 | Rust 路由及应用测试含正常单场、暂无、零场战绩、非法/缺失格式参数；实际 Rust SQL 在隔离库验证正常/空/负改善/并列最大值/失效赛程排除。没有模型、Elo 或概率计算 |
| 9 小程序文案与检查 | 满足自动化，真机待验收 | 整数百分比含双余数边界总和=100；τ 空值/阈值、负改善如实显示；文案黑名单扫描通过。所有登录等级可见，游客/审核隐藏；换球队、晚响应、返回页面、换账号请求隔离都有回归 |
| 10 cargo / pytest | 满足 | cargo 545 passed、0 failed；Python 默认 205 passed、13 skipped；显式隔离库和研究快照的全量运行 **213 passed、5 skipped**，详见下节 |

## 命令与真实输出

会话日志：`.superpowers/sdd/2026-10-18-match-outlook/`（忽略目录，不提交研究原始快照或运行凭据）。下列命令实际通过，均使用 rtk 包装。

| 命令 | 真实结论 |
|---|---|
| `./deploy/local-test/up.sh` | exit0，编译后 systemd active，18092 健康 status=ok；最后回执 **2026-10-18T06:58:11Z**（北京时间 14:58:11），不执行迁移 |
| 后端 `cargo test` | **545 passed / 0 failed / 73 ignored**，34 个测试集合；本功能 3 个 Rust 测试通过 |
| `cargo check` | exit0，Finished dev，3.18s |
| `cargo build` | exit0，Finished dev |
| `cargo clippy --lib` | exit0，17 项存量 lib warnings；本功能新模块无告警，没有顺手调整旧模块 |
| scraper `uv run pytest -q` | **205 passed / 13 skipped**，1.70s；默认不自动连接数据库或查找外部快照 |
| 隔离库＋快照下 `uv run pytest -q -s` | **213 passed / 5 skipped**，4.49s；全部 7 项新 SQL 测试及 1 项真实回测测试通过，容器 finally 删除 |
| `MATCH_OUTLOOK_HISTORY_DIR=… uv run pytest tests/test_match_outlook_reproduction.py -q -s` | 1 passed，输出 `production V2 RPS=0.19877789036731; sample=1487; 2026 parameters=Params(K=10,H=60,s=150,c=.7,gexp=1,reg=.8,newstart=1450)` |
| 原研究 `fix.py`（同一份快照，临时 scipy 环境） | exit0，V2 RPS 0.1988、命中率 54.5%；未给生产增加 scipy 依赖 |
| `uv run sina-csl-scraper match-outlook --help` | exit0，正确登记 backfill / refresh 子命令 |
| 小程序 `bun test` | **908 pass / 0 fail**，5941 expect，85 个文件，13.26s |
| `bun run type-check` | vue-tsc --noEmit，exit0，0 错误 |
| `bun run check:boundaries` | 依赖边界检查通过；组件无 api 导入、页面无互引、未改 ALLOWLIST、HTTP 仍收口 request |
| `bun run build:h5` | DONE Build complete，exit0 |
| `MINI_REVIEW_SKIP=1 bun run build:mp-weixin` | DONE Build complete，组件注册及产物生产 API 检查通过；未登记版本、未上传 |
| `bun run verify:mp-components` | All mini-program custom components are registered |
| 本地只读 HTTP | 无记录单场 200 `{"status":"unavailable","outlook":null}`；战绩 200 零场、各统计 null；负 ID 400 `参数无效`。未发起写请求 |
| `git diff --check` | exit0，无空白错误 |

工具链保留 Vite CJS、Browserslist 过期、uni-app 可升级提示；未为本功能升级前端工具链或改版本号。Python 只新增 numpy 生产依赖及锁文件。

## SQL 验证与冻结保护

`tests/test_match_outlook_postgres.py` 必须显式传入 `MATCH_OUTLOOK_TEST_DATABASE_URL`，并断言 host 为 127.0.0.1、database 为 match_outlook_test。scratch 脚本创建 postgres:16-alpine 专用容器、随机本地端口，finally 删除。不会从项目 `.env` 加载地址，也不复用生产 football_data。

每项先在 `BEGIN / SET LOCAL lock_timeout='2s' / ROLLBACK` 中执行完整新迁移，并断言新表不存在；之后只在此一次性库安装用于实际 SQL 测试。测试包括发布、拒绝迟到/改期/失效赛程、直接 UPDATE 冻结拒绝、一次计分、冻结后改期标记与未知时间跳过，以及从 Rust 源码直接提取的两个 SQL 查询。

```text
.......
7 passed in 0.63s
isolated PostgreSQL container removed; production database never connected
```

其中并列最高概率的命中与 RPS 改善单独判断：命中并不等于优于基线，实际测试保留了“命中但 RPS 变差”的样本。未通过改写存储概率美化战绩。

## 审查及偏离说明

- 按负责人最新文档实施 newstart 初始化与调参 τ=1，无此处偏离。原研究按时间排序，生产增加 match_id 作为同刻稳定排序键；同刻比赛批量预测，杜绝原脚本的同刻结果泄漏。两者导致微小 RPS 差异，仍满足设计明确容差；原研究脚本未修改。
- 当前 `/live/matches` 实际只返回一个轮次，并非整个未来赛程。小程序通过已有 rounds 与逐轮 matches 接口加载未完赛轮次，不新增第三个后端接口；默认选择所选球队下一场，历史公开计分由赛季汇总展示。
- 公开计分从功能实际运行后开始，不把研究回测当作公开战绩。赛季初提醒依据 rounds 中真实已完赛总数 `<30`，不按新表暂时为零的公开计分数判断，避免十月份误称“赛季初”。
- 原共享球队选择器允许无排名球队显示名字，不伪造排名、不隐藏升班马；保留既有非空排名显示。贡献资料迟到时保留当前有效球队选择，是为共享选择所需的最小改动；会话、付费度量与 sessionEffects 逻辑未重写，原用例仍全通过。
- 新模块/迁移命名由实现方决定。设计没有失效的具体文件路径；实际源码目录与本文 API 链接一致。没有改榜赛页、抢票统计、比赛 ID 门控或 Android 项目。
- 一次独立全量审查发现球队切换竞态、后续轮次不可见、返回页重置选择、首页审核态入口；全部修复，旧请求也不再拖住新球队的 loading。补测另发现 auth intent 期间旧用户尚在内存，应等待新用户引用发布后再读关注球队，已同步失效旧请求。

TDD 的真实失败包括：原始概率取整总和 99；队伍切换只请求旧比赛；会员资料覆盖已选球队；未完赛历史被接纳；同刻第二场先读取第一场结果；账号 intent 期间读取旧关注资料。每项先复现失败，再实现/修复并通过回归。

## 小程序结构与跨端

- InsightsMatchOutlook：FiPanel / FiSectionHeading、复用 InsightsTeamSelector、比赛按钮列表、主胜/平/客胜等权三段条、把握度、冻结/改期状态、赛季计分及固定说明。沿用洞察页行卡/胶囊与 FiPanel 的视觉；surface/page/primary/text-secondary/divider 色、space8–24、font24/26/28、leading-normal、radius-sm/round、border-width 与 weight-bold 均用已有 token，没有新增色值、渐变或专用 token。
- 把握度：τ `<0.8` 提示联赛较均衡、把握度偏低；`0.8≤τ<1.05` 为球队差距有限、保持谨慎；`τ≥1.05` 为差异相对明显、比赛仍有不确定性；null 不显示。固定说明逐字为“基于历史比赛结果的统计模型，仅供球迷参考，不代表比赛结果。”
- 首页单行入口只跳转洞察，不请求概率；有 token、有下一场且共享审核限制允许才显示。洞察面板位于会员锁定分支上方，取关暂停或低等级不构成免费面板门槛，游客沿用既有登录引导。
- 新比赛按钮和首页入口用 `@tap`、`hover-class`；复用选择器保留原 `@click`，uni-app 两端均支持。球队横向滚动沿用 scroll-view 和组件内 selector query；页面原生纵向滚动，无新嵌套滚动容器或页面计时器。
- CSS 变量继承全局 fi-tokens，新增组件直接 `.vue` 导入，微信组件注册已验证；没有浏览器 DOM / wx 专用调用。跨账号、审核、卸载使 owner 失效，迟到请求不写回。两端编译通过不等于微信真机布局验收。

## 未做与上线步骤

- 未执行 73 个存量 Rust ignored 测试与 5 个存量 Python 外部依赖测试，避免依赖生产或未配置服务；本功能新增 SQL 和回测测试已在显式隔离环境全部运行，默认 skip 不是未验证。
- 未对生产迁移/回填、未真实刷新预测，未执行真实网络下载 12 个赛季（已有同源研究快照作数值核验）；断点下载逻辑用 fake 客户端验证。未执行 H5 浏览器交互或微信真机视觉验收，只有纯函数/状态/接口自动化和双端构建证据。
- 已用纯离线模拟完成 2027 目标年重选、新球队 newstart、未来结果不进入轨迹的演练；缺完整 2026 时拒绝提前生成 2027。2026 完整赛季结束后、2027 上线前仍需用真实完整数据再演练，不伪造剩余赛果。
- 上线须负责人审查并另行指令：① peiqian 手动执行 `20261018140000_add_match_outlooks.sql`；② 更新 scraper 依赖并按 README 先 dry-run 下载校验，再显式 backfill --write-db；③ 发布后端及 scraper、显式 refresh 或正常 scrape --write-db，核对真实赛前记录；④ 上传/审核小程序并真机检查登录、审核隐藏、球队切换、百分比和负战绩。禁止用已开赛比赛补造公开记录。

## 改动文件

全部为新增或修改，无删除。

**后端新增：** `migrations/20261018140000_add_match_outlooks.sql`；`src/match_outlook/{domain,ports,application,bootstrap,mod}.rs`；`src/match_outlook/adapters/{persistence,web,mod}.rs`；`docs/match-outlook-api.md`。

**后端修改：** `src/{app,lib}.rs`、`README.md`。

**scraper 新增：** `src/sina_csl_scraper/match_outlook/{__init__,model,history,repository,service,cli}.py`；`tests/test_match_outlook_{model,history,cli,sync,service,postgres,reproduction}.py`。

**scraper 修改：** `src/sina_csl_scraper/{cli,sync}.py`、`pyproject.toml`、`uv.lock`、`README.md`。

**小程序新增：** `src/{api,types}/matchOutlook.ts`；`src/pages/insights/{matchOutlook,matchOutlook.test,useMatchOutlook,useMatchOutlook.test}.ts`；`src/pages/insights/components/InsightsMatchOutlook.vue`。

**小程序修改：** `src/pages/home/index.vue`；`src/pages/insights/{index.vue,useInsightsData.ts,sessionEffects.test.ts}`；`src/pages/insights/components/InsightsTeamSelector.vue`。

**公共文档：** 本报告、实施计划、根 `CHANGELOG.md` / `FEATURES.md`（数据后端与小程序条目分开提交）。设计和研究脚本保持负责人定稿内容。

数据/后端提交：`13a793c`（先推送 main）；小程序展示层独立后续提交。同步 main 时保留已提交的首页深色卡片改动，未纳入本任务提交。

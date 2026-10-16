# 最佳盯盘时段实施计划（按用户澄清修订）

日期：2026-10-16。设计：`docs/superpowers/specs/2026-10-16-best-watch-windows-design.md`。
状态：具体座位目标已明确；实际座位标识与数据映射尚未核实，原区域算法的未提交代码不算交付。

## 目标与固定边界

- 以可稳定识别的具体座位为统计对象，不能仅把区域余票总数改称座位结果。
- ticket monitor拥有peiqian MySQL数据，独立离线计算并通过HTTP提供快照；football运行时只读接口，小程序只读football接口。
- 比赛结束后计算，在线不重算，不把统计挂进抢票循环。快照带口径/版本标识，旧区域快照不可混用。
- monitor提交master；football提交main；不新建分支、不混入他人改动。代码推送不代表自动迁移、部署或上传。
- 验证只读；生产库不写入、不DDL；不碰Android、价格、支付、现有轮询、历史筛选与旧付费度量。

## Task 0：完成数据身份核验（后续统计实现的前置条件）

- [x] 核对生产monitor实际连接与表：peiqian MySQL / ticket_system.rs_block_info。
- [x] 沿实际写入追踪：BlockInfoMonitor::check_and_save_block_info → TicketApiClient::get_ticket_regions → save_block_info → MySqlBlockInfoRepository::create。
- [x] 确认字段来源：block_name=item.name，inventory=item.usable_count；id为采样主键。
- [ ] 核实用户所说具体座位与这些编号的业务映射；证明同一业务座位跨采样可稳定识别。
- [ ] 若现有记录仅为区域总数，明确历史数据缺口，补齐座位数据来源方案，不从数量反推座位。
- [ ] 定稿seat_key映射、事件规则及真实只读基准；未完成前不提交/发布区域方案冒充本任务。

## Task 1：monitor座位级离线快照与接口

- [ ] 先写失败测试：座位身份、同座位重复采样、多座位独立事件、起算边界与24桶合计。
- [ ] 根据核实后的真实字段实现独立计算端口，离线作业与在线读取端口分离。
- [ ] 先失败再实现：口径版本隔离、原子发布/重算、在线缺失不计算、旧区域快照拒绝、售票窗口避让。
- [ ] 只读EXPLAIN/对账并回滚；更新OpenAPI和报告；cargo test/check/build/clippy，独立审查后提交推送master。

## Task 2：football接口读取、后台聚合与分级

- [ ] TDD验证座位粒度及快照版本、HTTP编码和已有ID回退；不直连monitor MySQL。
- [ ] TDD覆盖8/20/3选场边界、统计单位、平均/占比/稳定前三、单场缓存与后台10分钟聚合。
- [ ] TDD验证前台不访问上游，会员失败preview，full缓存不泄漏，新付费位白名单；薄handler。
- [ ] 本地up.sh重启后cargo test/check；本地只读接口联调，报告与记忆文件更新，独立提交推送main。

## Task 3：小程序展示

- [ ] helpers/composable/面板先RED后GREEN：请求缓存、会话owner、24柱/单时段预览、静默降级及组件分钟时钟。
- [ ] index只接入新面板和新付费位，既有历史/会话/轮询/两个付费入口不变；token与依赖边界遵守既有约定。
- [ ] bun test/type-check/check:boundaries/verify:mp-components与双端构建；窄屏验证；报告逐条对照设计第9节。
- [ ] 独立审查并提交推送main；部署/上传另按用户具体指令执行。

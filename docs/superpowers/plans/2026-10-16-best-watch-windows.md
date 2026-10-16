# 最佳盯盘时段实施与验收记录

日期：2026-10-16。设计：`../specs/2026-10-16-best-watch-windows-design.md`；评审：`../reviews/2026-10-16-best-watch-windows-review-1.md`。

## 口径与边界

设计第00节已明确：座位就是`rs_block_info.block_name`号码，区域是price价位组，不需要更细排号/座号。以(schedule_id,block_name)识别序列，按time,id定序，整场LAG后过滤开售+10分钟，按北京时间原值分桶。

monitor比赛结束后独立离线计算，在线只读版本化文件；football后台读取HTTP快照聚合，前台只读缓存并按有效会员裁剪。生产不写入、不DDL、不迁移、不部署、不上传，不改变抢票循环、历史筛选、现有付费位。

## Task 1：monitor（完成实现与验证，提交master）

- [x] 事件纯函数及边界先RED后GREEN；SQL只在离线端口，单连接只读。
- [x] v2/stat_unit隔离、24对象桶、原子发布、在线缺失不重算。
- [x] B1：60秒命名预算、新到旧、失败继续且整轮最终失败；售票窗口避让。
- [x] 551/556/573/最新583真实只读对账且回滚；最大551的267279行6.890秒。
- [x] cargo test/check/build/clippy及本次文件格式检查；报告在monitor docs/best-watch-windows-validation.md。
- [ ] 生产部署、首次离线生成和启用定时器：待用户另行指令。

## Task 2：football后端（完成实现，最终验证见报告）

- [x] TDD：选场8/20/3、平均/占比/前三、版本/桶合计、HTTP编码与ID回退。
- [x] R1：类型化404在固定8场内跳过，不补老场；非404仍失败。
- [x] R2：失败保留旧结果和generated_at，60秒重试，成功600秒；前台零上游请求。
- [x] 有效会员逐请求裁剪、preview不泄漏完整分布、新付费位白名单与薄handler。
- [x] 本地重启后cargo test/check与真实本地快照HTTP联调；完整结果见docs/best-watch-windows-validation.md。
- [ ] 生产部署：待用户审查后指令。本任务没有迁移。

## Task 3：小程序（完成实现与验证，独立提交main）

- [x] helpers/composable/真实SFC测试先RED后GREEN，完整/预览/隐藏/分钟时钟。
- [x] 原历史、轮询、旧两个付费位与beginPaywallVisit保持原样，仅接入新面板与surface。
- [x] 复审修复revision先更新、token后清除的退出竞态；真实session/request回归证明退出后无旧Bearer。
- [x] 四项验证、H5/离线微信构建与320窄屏夹具；细节见football_insight_mini/docs/best-watch-windows-validation.md。
- [ ] 微信开发者工具/真机视觉验收及上传：待用户另行指令。

## 提交顺序

monitor master → football后端main → 小程序main。每次仅暂存本任务具体文件，带Co-Authored-By，推送与生产发布分开。提交号及推送顺序在最终汇报给出。

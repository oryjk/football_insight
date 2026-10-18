# 赛季总结（里程海报）验证报告

日期：2026-10-18。设计：`superpowers/specs/2026-10-18-season-recap-mileage-design.md`；实施计划：`superpowers/plans/2026-10-18-season-recap-mileage.md`。后端 API 契约：`../football_insight_service_backend_rs/docs/season-recap-api.md`。

**本地后端已重启；生产后端未部署/未重启；生产迁移未执行；未上传小程序。** 未使用真实用户写入测试，未真实支付、未向公众号或用户发送消息。代码提交、推送不等于发布。

## 实现范围

免费、仅成都蓉城一线队。用户勾选已结束的主客场，输入一次主场单边公里数，云端保存后由后端计算；小程序生成带真实微信小程序码的海报。入口为「我的 → 我的赛季总结」，无会员门槛、无定位、无凭票验证、无排名。

产品负责人已确认：采用云端保存；云南玉昆为玉溪，其他13个城市按下面核对表。首版赛季默认2026，明确年份的分享链接在赛季结束后仍可打开；接口允许2026至当前年份，未来新球队需维护城市表，不猜测未知城市。

## 六项验收自查

|设计验收|自查|依据与限制|
|---|---|---|
|1 客场按各城市大圆距离×2累加|满足自动化|半径6371km的Haversine，单场往返四舍五入后求和；赤道1°约111.1949266km、对称/同点、成都北京往返2900–3100km、云南玉溪1400–1500km断言；未知城市显式失败|
|2 主场单边×2×场次|满足|12.25km的一场主场取整25km，0/500边界；主场总计取整后与客场整数相加。无客场标注「本赛季尚未远征」，无距离且无主场显示「未填写主场距离」|
|3 重复勾选不重复计入|满足|应用HashSet、数据库(user_id,match_id)主键和ON CONFLICT；两支上海队里程各计一次、城市只计一个，青岛复用同一城市键|
|4 非法距离拒绝且有反馈|满足|前端拒绝负数、超500、非数值、指数/十六进制、超两位小数；后端拒绝非有限数及越界，数据库CHECK另校验；缺主场距离返回固定中文提示|
|5 海报生成/保存/扫码src=share|分段验证通过，真机待验收|H5 canvas导出、分享路径、短scene经实际来源解析器得到share/recap2026；微信码HTTP请求与缓存使用本机mock服务验证。相册拒绝授权引导设置、失败提示单测通过。未调用真实微信生成API，未执行微信真机保存/扫码，不能据此宣称真机验收完成|
|6 全量测试与静态检查|满足|最终命令见下一节；数据库测试独立运行，存量依赖生产/外部服务的ignored项不贸然启用|

## 验证命令与真实输出结论

所有shell命令前缀rtk；会话scratch为`.superpowers/sdd/2026-10-18-season-recap-mileage/`。

|命令|实际结论|
|---|---|
|`./deploy/local-test/up.sh`|exit0，编译后用户systemd active，18092健康接口status=ok；未自动迁移。最后健康回执2026-10-18T05:29:42Z（北京时间13:29:42）|
|后端`cargo test`|539 passed / 0 failed / 73 ignored，34 suites；本次新增的一次性DB测试另行显式运行通过|
|后端`cargo check`|exit0，Finished dev|
|后端`cargo build`|exit0，Finished dev|
|后端`cargo clippy`|exit0；存量warnings保留，17项存量lib warnings；本次新模块的collapsible_if已修复，无本模块告警|
|小程序`bun run type-check`|vue-tsc --noEmit，exit0，0错误；首次复现index.vue:22 TS2322，修复后通过|
|小程序`bun test`|889 pass / 0 fail，5870 expect calls，81 files，13.26s|
|`bun run check:boundaries`|依赖边界检查通过；未改ALLOWLIST；组件无api导入；HTTP仍走request|
|`bun run build:h5`|DONE Build complete，exit0|
|`MINI_REVIEW_SKIP=1 bun run build:mp-weixin`|DONE Build complete；组件注册及生产API产物检查通过；未申请版本、未上传|
|`bun run verify:mp-components`|All mini-program custom components are registered|
|H5 Playwright本机离线检查|320px视口无横向溢出；游客登录入口、已登录选择两场、输入13.5公里并核对PUT载荷、实际canvas导出、H5保存降级通过；API全部mock，不连接生产|
|本地18092未登录接口检查|GET/PUT总结、POST码接口均401「请先登录」；未携带用户凭据，没有进入仓库写入/微信调用|
|独立PostgreSQL实际SQL测试|1 passed / 0 failed，0.09s；临时容器最终删除，没有读取项目.env或连接生产数据库|

构建中的Vite CJS、Browserslist过期、uni-app可升级提示是工具链提示，未为本任务升级依赖。manifest.json和generatedMiniProgramVersion.ts均未修改。

## SQL真实验证

测试位于`src/season_recap/adapters/persistence.rs::tests::isolated_migration_and_repository`，默认ignored，必须显式提供`SEASON_RECAP_TEST_DATABASE_URL`；断言host为127.0.0.1、database为season_recap_test。执行脚本在上述scratch中，专门创建postgres:16-alpine一次性容器与随机本地端口，finally删除容器。不复用其他业务数据库。

真实执行新迁移、实际仓库查询/事务/约束，全部用户和比赛是该隔离库自己创建的数据：

```text
migration transaction rollback: new tables absent
actual SQL passed: eligible first-team/finished/active/season filter; unique selections; user isolation; per-season replacement; invalid selection rollback; distance constraint; reads do not lock matches; saves hold shared locks; omitted/null distance preserved
test result: ok. 1 passed; 0 failed; 0 ignored
isolated PostgreSQL container removed; production database never connected
```

- 迁移在BEGIN/SET LOCAL lock_timeout='2s'/ROLLBACK里验证，并断言profile表不存在；之后只在一次性库实际安装用于仓库测试。
- 验证真实一线队/完赛/active/赛季过滤、重复去重、两用户隔离、只替换本赛季、不合法比赛/距离失败回滚。
- 并发事务执行FOR UPDATE NOWAIT：普通snapshot读取时能获取更新锁；保存用snapshot持共享锁时得到55P03。读操作是READ ONLY/REPEATABLE READ，SQL中不含FOR SHARE。
- 省略或null距离保留偏好，0仍可明确设置；不会因操作另一赛季清空已有主场里程。
- 应用通过port的ValidateSnapshot回调，在锁内重新计算/校验再提交；规则不在handler/SQL中重写。测试模拟客场被更正成主场且缺距离，断言MissingDistance且未提交。

## 审查与TDD记录

先失败再通过的证据：计算5项、输入/归因2项、会话4项、海报/授权3项；本次收尾另覆盖以下实际问题：

- H5窄屏实测先复现输入区域仅2px高、文字被裁剪，再用已有space-72作为输入框高度，保持数值完整可见。
- 输入事件：先复现TS2322，改为H5 InputEvent边界类型并显式转换uni-app的detail.value载荷；两端仍使用@input。
- pages.json：先失败（赛季总结成为pages[0]），再移到home之后，保留原默认首页。
- 冷启动扫码后静默登录：先失败（调用数0），再监听会话revision/token就绪，下一微任务重载，过期请求继续丢弃。
- 401：先失败（未失效旧会话），再用ApiRequestError.statusCode仅清理发起请求的revision，显示登录恢复入口；旧账号的迟到错误不能清掉新会话。
- 读锁：隔离SQL先因FOR UPDATE NOWAIT失败而红，再拆分只读和保存锁路径。
- 空距离：保留断言先因MissingDistance失败，再用COALESCE保留已有值。
- 事务竞态：先复现“校验报错但已经提交”，再将应用校验回调放在锁内，未提交断言变绿。

独立审查覆盖全部新增/修改文件，发现上述默认启动、登录就绪、事务校验、401恢复和跨赛季清空距离问题；均纳入修复和回归。未发现付费门槛、身份泄漏或来源参数问题。审查不代替真机验证。

## 城市映射与来源

使用[GeoNames](https://www.geonames.org/)城市中心经纬度（WGS84，CC BY 4.0署名），不是球场坐标，也不伪装成实际交通路线。每项链接可以核对原始坐标：

|球队|主场城市|纬度/经度|GeoNames|
|---|---|---|---|
|成都蓉城|成都|30.66667 / 104.06667|[1815286](https://www.geonames.org/1815286/chengdu.html)|
|北京国安|北京|39.90750 / 116.39723|[1816670](https://www.geonames.org/1816670/beijing.html)|
|青岛海牛、青岛西海岸|青岛|36.06488 / 120.38042|[1797929](https://www.geonames.org/1797929/qingdao.html)|
|上海申花、上海海港|上海|31.22222 / 121.45806|[1796236](https://www.geonames.org/1796236/shanghai.html)|
|天津津门虎|天津|39.14222 / 117.17667|[1792947](https://www.geonames.org/1792947/tianjin.html)|
|山东泰山|济南|36.66833 / 116.99722|[1805753](https://www.geonames.org/1805753/jinan.html)|
|河南队|郑州|34.75778 / 113.64861|[1784658](https://www.geonames.org/1784658/zhengzhou.html)|
|浙江队|杭州|30.29365 / 120.16142|[1808926](https://www.geonames.org/1808926/hangzhou.html)|
|云南玉昆|玉溪|24.35500 / 102.54222|[1784185](https://www.geonames.org/1784185/yuxi.html)|
|大连英博|大连|38.91222 / 121.60222|[1814087](https://www.geonames.org/1814087/dalian.html)|
|重庆铜梁龙|重庆|29.56026 / 106.55771|[1814906](https://www.geonames.org/1814906/chongqing.html)|
|辽宁铁人|沈阳|41.79222 / 123.43278|[2034937](https://www.geonames.org/2034937/shenyang.html)|
|深圳新鵬城（兼容鹏）|深圳|22.54554 / 114.06830|[1795565](https://www.geonames.org/1795565/shenzhen.html)|
|武汉三镇|武汉|30.58333 / 114.26667|[1791247](https://www.geonames.org/1791247/wuhan.html)|

主场城市核对：中足联官方[云南玉昆](https://www.cfl-china.cn/zh/content/csl/3n4jrcw6w965v1qvp0azx5kwk.html)、[浙江](https://www.cfl-china.cn/zh/content/csl/a3l87ozfq8bjwpy0nxvs5w28o.html)、[辽宁铁人](https://www.cfl-china.cn/zh/content/csl/7u46yssnarhr1uyxsmv4q5x3q.html)、[深圳新鹏城](https://www.cfl-china.cn/zh/content/csl/a1a0zo3jliehxemg6qpa3lrva.html)、[重庆铜梁龙](https://www.cfl-china.cn/zh/content/csl/cjnb1ayk5mruxqpud9qmy4t1w.html)、[成都蓉城](https://www.cfl-china.cn/zh/content/csl/19g03gt1yj4oo8ak4vyf6sp9l.html)、[大连英博](https://www.cfl-china.cn/zh/content/csl/dicpumaiand6vk7vqbn4tqdxw.html)，以及[河南主场郑州比赛报道](https://www.cfl-china.cn/zh/content/news/Nlqp.html)。完整城市表已获负责人确认。

设计提到的`src/utils/stadiumRegions.ts`实际是座位分区/环层布局，不含球队城市或地理坐标，因此不复用。后端实际赛程字段match_id是BIGINT，完赛status='3'；沿用真实代码/schema。

## 跨端与视觉结构

- 用户入口：UserSeasonRecapEntry是单行卡片、标题/说明/箭头，参考已有FiPanel外观，用surface、text-primary/muted、primary、space24/16/8、font30/24/36、radius-lg、border-card、shadow-card；只emit open，导航在用户页。
- 页面：复用FiSecondaryNav、FiPanel、FiSectionHeading、FiLoading。纵向标题、比赛勾选、距离表单、海报预览；保留原生页面滚动，不嵌套固定高度scroll-view。space-card-gap与页面padding/nav/safe-area，颜色/字体/圆角全用既有token，无新增token/渐变。
- SeasonRecapMatches：按钮胶囊/卡行，勾选记号、主客场对手与日期；参照FiPanel和既有行卡，space12/16/20、font24/28/32、radius-sm、divider/primary；@tap和hover-class均可跨端，禁用时不提交。
- SeasonRecapPoster：只接收payload并emit ready/failed，不import api。复用canvasPoster的圆角、flush、导出原语，750×1080像素离屏绘制；它是导出图片的几何尺寸，CSS像素匹配画布坐标而不是页面布局。decorative海报绘制色使用项目视觉规范允许的位图例外，页面UI无新增散落hex。
- 微信：base64码写入USER_DATA_PATH临时图片，绘制后finally删除；open-type=share与相册保存限定MP-WEIXIN。授权拒绝用getSetting明确判断，提示openSetting，不猜错误文案。
- H5：码图片使用data URL；保存提供图片预览/长按和复制链接，无微信专属open-type；两端都使用uni.createCanvasContext和uni.canvasToTempFilePath。
- 当前无每秒定时器、无定位或第三方DOM运行时代码；页面卸载/换账号清空状态，在途加载/保存/绘图均以operation/revision/key丢弃旧结果。

## 偏离、未做与上线顺序

- 唯一产品口径修正：设计例子“云南玉昆为昆明”改为实际主场玉溪，已获确认并同步设计。云端保存也已确认。
- 省略/null距离保留是本次审阅确认的接口完善，不增加清除功能。未来球队表需要维护；未知城市提示失败，绝不默算0。
- 首版只做我的入口；首页赛季末提醒作为后续项，没有顺手修改首页。
- 未执行生产DDL/生产用户数据测试；隔离PostgreSQL已验证真实SQL。未运行存量73个ignored外部/DB测试的全部集合，防止默认生产连接和环境依赖；本次专用ignored测试已显式通过。
- 未联调真实微信令牌/小程序码、真机海报渲染/相册/扫码；上线后必须验收，当前只有mock HTTP、纯函数、H5和离线编译证据。初次生成微信码失败会有提示，且云端勾选仍保留。

上线须由负责人另行指令，推荐顺序：

1. 审查后手动执行迁移`20261018130000_add_season_recap.sql`（大于已有20261018120000），只新增两张表与索引，无回填。
2. 部署并重启生产后端，核对既有微信小程序appid/secret配置及新增接口；先不推广海报。
3. 上传/提审新小程序，**审核通过并正式发布新页面后**，再对外推广海报入口；需要真机验证生成、保存及扫码后src=share。
4. 赛季末如需首页提醒，另行安排，避免用户不知道我的入口。

## 改动文件清单

清单按后端和小程序分别列出，全部为新增或修改，无删除：

后端新增：迁移`20261018130000_add_season_recap.sql`；`src/season_recap/{domain.rs,ports.rs,application.rs,bootstrap.rs,mod.rs}`；`src/season_recap/adapters/{persistence.rs,wechat_code.rs,web.rs,mod.rs}`；`docs/season-recap-api.md`。

后端修改：`src/app.rs`、`src/lib.rs`、`docs/api.md`。

小程序新增：`src/types/seasonRecap.ts`、`src/api/seasonRecap.ts`；`src/pages/season-recap/{index.vue,helpers.ts,helpers.test.ts,flow.ts,flow.test.ts,poster.ts,poster.test.ts}`；`src/pages/season-recap/components/{SeasonRecapMatches.vue,SeasonRecapPoster.vue}`；`src/pages/user/components/UserSeasonRecapEntry.vue`。

小程序修改：`src/pages.json`、`src/pages/user/index.vue`。

配套文档：设计澄清、实施计划、本报告、根CHANGELOG.md/FEATURES.md（后端和小程序各自条目分两次提交）。

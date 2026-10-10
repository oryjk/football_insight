# 第一阶段静默登录实施报告

2026-10-11。业务实现：本地 Kimi `kimi-code/k3-256k`；计划、独立 review、最终验证与本报告：Codex。Kimi 最后文档阶段触发额度限制，Codex 按实际验证补齐文档，未代写业务代码。

目录 `/Users/carlwang/projects/football_insight`；分支 `feat/mini-registration-alignment`；基线 `91d11c2`；最终业务提交 `183da4b`。代码 review 与本地检查通过，详见 [Codex Review](2026-10-10-mini-silent-login-phase1-review.md)。文档提交在业务提交之后，业务检查针对上述固定源码。

## 实施内容

1. `2a31582`、`e278150`、`032125e`：从微信 code 获取前建立认证意图；MP 后台仅恢复已绑定账号，未知微信保持游客；手动退出持久化；显式操作替代本次启动探测。有效 token 共享 me，H5 不自动探测微信但仍恢复已有 token。
2. `e2cbc6f`、`4c9fea3`、`b9fabac`、`991414c`：App 非阻塞启动恢复；已可见私有页面随会话发布更新。私有读取共享 ensure 与 owner，跨账号清缓存；有效主队写后强制刷新，旧 PUT 放弃；洞察审核 gate 独立于账号，旧私有 await 不继续。
3. `aeb092d`、`b440991`、`d271b42`、`183da4b`：看板 owner 独立订阅 generation；reset 允许新请求，旧结果与清理失效；页内 Tab 补载未加载看板；list 与用户选择每次读取获取共享单调 owner，避免同选择旧请求回退 cache/display。
4. `7727872`、`183da4b`：所有真实比赛替换同步复位订阅/解锁缓存、loading、sheet、paying；同比赛 poll 保持合法操作。解锁 close/open/refresh/pay 按 owner 防护，支付开始使旧权益 GET 失效；原生支付后失效上下文不再继续确认读。

没有新框架、全局业务状态或组件 API 调用；AI 保持移除。邀请码、会员、订阅、支付和 `sale_start_at + 10 minutes` 统计规则保持。

## 实际验证

从 `football_insight_mini` 执行，全部 exit 0：

```bash
bun test                      # 505 pass / 0 fail / 4491 assertions / 35 files
bun run type-check
bun run check:boundaries
bun run build:h5
MINI_REVIEW_SKIP=1 bun run build:mp-weixin
```

离线小程序组件注册与产物 API 校验通过，API 为 `https://match.oryjk.cn/api/v1`；版本保持 1.0.64 / 10064。版本文件未改号，未登记/上传。diff 检查与文件范围检查通过，无 env/密钥/证书/生成产物提交。

独立固定源码验收：首页/洞察 18 项；最终看板 26 项；最终订阅/解锁 25 项，另实际组件支付接线 6 项，均按正确安全行为断言并通过。历史错误诊断的 pass 不计为验收。

最终 H5 实际页面 mock API 回归 4 组通过：密码失败重试/成功/邮箱/退出重载游客；白名单显式账号切换；洞察延迟 me 时一次读取；首页公开内容不阻塞、私有 profile 等 me 完成才读取一次。无微信认证请求/脚本错误，没有请求生产写接口。

## 尚未验证与运行状态

微信开发者工具/真机冷启动、过期 token、未知微信、退出重开、显式绑定重试、慢 code 退出、跨 Tab 私有数据/轮询、切球队回切，以及真实微信支付和端上视觉尚未验收。单元测试、构建和 mock H5 不替代端上证据。

后端/数据库/scraper/独立 H5/报名系统没有修改，无需本轮重启本地后端。生产后端未部署、未重启；本轮没有 push、合并 main、上传小程序或版本登记。业务与文档均只作本地提交。

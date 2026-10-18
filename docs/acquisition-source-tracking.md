# 用户来源追踪：接入与统计

首期只追踪首次注册来源，不改变登录、邀请码、会员或支付。只记录运营自定义渠道/内容标识，不采集第三方用户身份、设备指纹、手机号或IP。公众号关注事件来源暂不接入。

## 入口参数

- 普通入口query：`src=douyin&cid=20261018_reflux_board`。src最长32、cid最长64，只允许小写ASCII字母、数字、下划线、短横线。
- 小程序码query.scene：`s=douyin&c=0418a`，**解码后整个字符串最多32个ASCII字符**。可以原样或URL编码传入。这里只是缩短参数名，c本身就是内容ID，不需要代码内短码表；运营在自己的表格维护0418a对应哪条内容。新增渠道或内容无需发版。
- scene只接受s、c各一次；畸形编码、重复键、超过32字符丢弃。合法直接src优先，cid不从另一个scene混入。
- 必须有合法src才形成来源；没有src或src非法时不锁定首次来源，后续合法入口仍可采集。cid非法仅丢弃cid。
- `landing_scene`是微信启动选项的原始**数值场景码**，不是query.scene文本；只保存1–65535整数，其他值丢弃。

冷启动在认证恢复前调用`uni.getLaunchOptionsSync()`采集；热启动onShow也可采集。首次合法来源保存在`fi_first_acquisition_v1`，后续入口不覆盖，退出登录也不清除。清理小程序本地缓存/卸载会失去此本地信息，这是首次触点的边界；老账号登录不会把这次来源写到服务端。

## 请求与后端

会话调用层向原登录/注册/小程序绑定请求追加可选字段，API层仍只调用既有request：

```json
{"acquisition":{"src":"douyin","cid":"test1","landing_scene":1047}}
```

只有`POST /api/v1/auth/register`、`/api/v1/auth/mini-wechat/bind`、`/api/v1/auth/wechat/bind`首次真正INSERT新账号后才记录。登录接口允许携带但忽略；微信phone绑定旧账号、重新激活旧账号不记录。旧请求体、状态码、错误文案和响应体保持原样，客户端不能指定user_id。身份取自用例成功创建的账号。

非法类型/格式的来源只丢弃，不拒绝认证请求。后端domain验证32/64限制，数据库另有CHECK约束。记录使用独立异步任务、128并发上限、250ms超时、最多2连接的专用lazy pool及100ms获取超时；表缺失、超时、错误、panic或饱和只warn，不等待、不重试登录、不改变认证结果。服务停止前尚未完成的记录可能丢失，属于尽力而为。

新表`f_i_user_acquisitions`以user_id为主键并引用f_i_users，`INSERT ON CONFLICT(user_id) DO NOTHING`使后来的请求不能覆盖首条。没有来源不创建记录；不回填历史账号。

## 统计口径

直接运行只读[统计SQL](sql/user-acquisition-summary.sql)，可修改params里的注册时间范围，按src、cid分组。没有来源的账号不纳入这些分组。

|字段|定义|
|---|---|
|registered_users|来源表中对应的新注册账号数|
|d1_eligible_users|距注册已满24小时，可计算D1+的账号数|
|d1_plus_return_users|现存last_login_at至少晚于注册24小时的账号数|
|d7_eligible_users|距注册已满168小时，可计算D7+的账号数|
|d7_plus_return_users|现存last_login_at至少晚于注册168小时的账号数|
|paid_users|存在status为paid/refunded且paid_at已发生的订单的不同账号数，多笔订单只计一次|

回访按现有`record_user_login`维护的`f_i_user_activity_snapshots.last_login_at`计算，既有`/auth/me`成功恢复也会更新该值。**现有数据只有最后登录，不是登录历史；D1+/D7+是累计回访代理，不是严格第1/7自然日留存。** 比如第8天回来也算D1+和D7+，无法证明他第1天是否来过。比例分别除以对应eligible人数，0分母不计算。若要严格自然日留存，需另行设计逐日历史，首期不扩展表/登录记录机制。

paid_users回答“是否发生过付款”，已退款也算曾经付费，不代表净收入或当前权益；未来paid_at、未付款pending不计。全程不混入抢票余票类统计口径。

## 上线顺序与隐私

1. 审查后由负责人在peiqian手动执行`20261018120000_add_user_acquisitions.sql`；它只新增表/索引，不修改f_i_users。部署脚本不会自动迁移。
2. 部署后端，再按现有流程上传小程序。迁移前即使后端收到来源也只能warn并丢弃，因此应先迁移。
3. 用带参数的新账号入口验收，之后只读执行统计SQL。测试不能修改生产真实账号，也不能真实付款。
4. 在运营表格约定src和cid含义，不携带第三方用户ID或其他个人信息。

仓库小程序源码/文档中未找到可确认涵盖“来源统计”的隐私政策文本。**产品负责人需要核对微信后台隐私声明及现行政策，必要时补充“渠道来源统计”的用途。** 本任务不自行修改隐私文案。

本任务不执行生产迁移、部署或小程序上传。验证和限制见[验证报告](acquisition-source-tracking-validation.md)。

# Football Insight Admin Android

足球洞察原生 Android 管理端，使用 Kotlin、Jetpack Compose、Material 3、Retrofit 和 Android Keystore。

## 功能

- 独立管理员用户名/密码登录
- Admin Bearer JWT 安全存储与服务端会话撤销
- 用户搜索、状态筛选、详情查看
- 用户详情中的邀请下级、活跃、订单、订阅和设备信息
- 用户禁用与恢复，必须填写原因
- 会员等级与到期方式调整，必须填写原因
- 管理操作审计日志
- 运行时验证并切换 API 服务器
- 可选生物识别本地解锁

## 网络配置

默认生产地址：

```text
https://match.oryjk.cn/
```

不要在真机上使用 `localhost`；它指向手机自身。Android 模拟器访问开发机本地 `8080` 端口时使用：

```text
http://10.0.2.2:8080/
```

构建默认值可在 `local.properties` 中覆盖，该文件不会提交：

```properties
FOOTBALL_ADMIN_API_BASE_URL=http://10.0.2.2:8080/
```

也可在 App 的“设置”中输入地址并点击“验证并切换”。切换前会请求 `/api/v1/system/public-config`，并检查 Football Insight JSON 标识；验证失败时不会保存地址。生产默认走 HTTPS，明文 HTTP 只对白名单开发主机 `10.0.2.2`、`localhost` 和 `127.0.0.1` 开放。

## 构建

```bash
ANDROID_HOME=/home/betalpha/Android/Sdk ./gradlew testDebugUnitTest lintDebug assembleDebug
```

Debug APK 生成在 `app/build/outputs/apk/debug/`，构建产物不会进入 Git。

## 内部下载发布

每次发布前必须提交并 push，然后执行：

```bash
ANDROID_HOME=/home/betalpha/Android/Sdk \
  ./scripts/publish-apk.sh --note "本次更新内容"
```

内网下载页：`http://172.16.60.233/football-admin-android/`。发布脚本会运行测试、lint 和构建，验证 APK 签名，并发布版本化 APK、`latest.apk`、SHA256 metadata 和历史版本。该入口由 local233 内部 gateway 托管，不发布到公网。

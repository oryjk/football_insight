# Football Insight App

回流监控 Flutter 客户端，支持 Android + iOS。

## 功能

- License 绑定码登录（在微信小程序内生成）
- 当前比赛回流面板（实时回流次数、最近回流时间）
- 看台区域关注（block_interests 切换）
- 历史回流统计（mock 数据，待接入真实接口）

## 配置

后端 API 地址通过 `--dart-define=API_BASE_URL=...` 注入。默认值 `https://match.oryjk.cn`。

## 开发

```bash
flutter pub get
flutter run                                            # 默认连默认 baseUrl
flutter run --dart-define=API_BASE_URL=http://127.0.0.1:8080
flutter analyze
flutter test
```

## 推送

推送通道暂未启用。如需接入，依赖 `jpush_flutter` 等极光 SDK + 原生侧 AppKey/Channel/权限配置。

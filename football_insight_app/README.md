# Football Insight App

回流监控 Flutter 客户端，支持 Android + iOS。

## 功能

- License 绑定码登录（微信小程序生成）
- 回流监控列表（实时回流状态、张数、区域）
- 看台区域关注
- 历史回流统计
- 推送通知（FCM + APNs）

## 开发

```bash
flutter pub get
flutter run
flutter test
flutter analyze
```

## Firebase 配置

1. Android: 将 `google-services.json` 放到 `android/app/`
2. iOS: 将 `GoogleService-Info.plist` 放到 `ios/Runner/`
3. 不要提交这些文件到 git

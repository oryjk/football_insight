import 'package:jpush_flutter/jpush_flutter.dart';
import 'package:football_insight_app/services/push_notification_service.dart';

class JPushService {
  final PushNotificationService _pushService;
  final JPush _jpush = JPush();

  JPushService({required PushNotificationService pushService})
      : _pushService = pushService;

  Future<void> initialize(String appKey) async {
    _jpush.setup(
      appKey: appKey,
      channel: 'developer-default',
      production: false,
      debug: false,
    );

    _jpush.addEventHandler(
      onReceiveNotification: (message) async {},
      onOpenNotification: (message) async {},
      onConnected: (message) async {
        final rid = await _jpush.getRegistrationID();
        await _pushService.registerToken(rid);
      },
    );
  }
}

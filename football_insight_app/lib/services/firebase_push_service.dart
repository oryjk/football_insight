import 'package:firebase_messaging/firebase_messaging.dart';
import 'package:football_insight_app/services/push_notification_service.dart';

class FirebasePushService {
  final PushNotificationService _pushService;
  final FirebaseMessaging _messaging = FirebaseMessaging.instance;

  FirebasePushService({required PushNotificationService pushService})
      : _pushService = pushService;

  Future<void> initialize() async {
    final settings = await _messaging.requestPermission();
    if (settings.authorizationStatus != AuthorizationStatus.authorized) return;

    final token = await _messaging.getToken();
    if (token != null) await _registerToken(token);

    _messaging.onTokenRefresh.listen(_registerToken);
    FirebaseMessaging.onMessage.listen(_handleForegroundMessage);
    FirebaseMessaging.onMessageOpenedApp.listen(_handleMessageOpenedApp);
  }

  Future<void> _registerToken(String token) async {
    await _pushService.registerToken(token);
  }

  void _handleForegroundMessage(RemoteMessage message) {}

  void _handleMessageOpenedApp(RemoteMessage message) {}
}

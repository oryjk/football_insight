import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:shared_preferences/shared_preferences.dart';

class NotificationToggleNotifier extends StateNotifier<bool> {
  static const _key = 'notification_enabled';

  NotificationToggleNotifier() : super(false) {
    _load();
  }

  Future<void> _load() async {
    final prefs = await SharedPreferences.getInstance();
    state = prefs.getBool(_key) ?? false;
  }

  Future<void> toggle(bool enabled) async {
    final prefs = await SharedPreferences.getInstance();
    await prefs.setBool(_key, enabled);
    state = enabled;
  }
}

final notificationToggleProvider =
    StateNotifierProvider<NotificationToggleNotifier, bool>(
  (ref) => NotificationToggleNotifier(),
);

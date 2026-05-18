import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:football_insight_app/config/api_config.dart';
import 'package:football_insight_app/models/user_profile.dart';
import 'package:football_insight_app/services/api_client.dart';
import 'package:football_insight_app/services/auth_service.dart';
import 'package:football_insight_app/utils/token_storage.dart';

final tokenStorageProvider = Provider<TokenStorage>((ref) => TokenStorage());
final apiClientProvider = Provider<ApiClient>(
  (ref) => ApiClient(baseUrl: ApiConfig.baseUrl),
);
final authServiceProvider = Provider<AuthService>(
  (ref) => AuthService(
    dio: ref.watch(apiClientProvider).dio,
    tokenStorage: ref.watch(tokenStorageProvider),
  ),
);
final authStateProvider = StateNotifierProvider<AuthNotifier, AsyncValue<UserProfile?>>(
  (ref) => AuthNotifier(ref.watch(authServiceProvider)),
);

class AuthNotifier extends StateNotifier<AsyncValue<UserProfile?>> {
  final AuthService _authService;
  AuthNotifier(this._authService) : super(const AsyncValue.loading()) {
    _checkLogin();
  }

  Future<void> _checkLogin() async {
    try {
      final loggedIn = await _authService.isLoggedIn();
      if (!loggedIn) {
        state = const AsyncValue.data(null);
        return;
      }
      state = AsyncValue.data(await _authService.getMe());
    } catch (_) {
      state = const AsyncValue.data(null);
    }
  }

  Future<void> bindLicense(String licenseKey) async {
    state = const AsyncValue.loading();
    try {
      state = AsyncValue.data(
        (await _authService.bindLicense(licenseKey)).user,
      );
    } catch (e, st) {
      state = AsyncValue.error(e, st);
    }
  }

  Future<void> logout() async {
    await _authService.logout();
    state = const AsyncValue.data(null);
  }
}

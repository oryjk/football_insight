import 'package:dio/dio.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:football_insight_app/config/api_config.dart';
import 'package:football_insight_app/models/user_profile.dart';
import 'package:football_insight_app/services/api_client.dart';
import 'package:football_insight_app/services/auth_service.dart';
import 'package:football_insight_app/utils/token_storage.dart';

final tokenStorageProvider = Provider<TokenStorage>((ref) => TokenStorage());

final _authDioProvider = Provider<Dio>((ref) {
  return Dio(
    BaseOptions(
      baseUrl: ApiConfig.baseUrl,
      connectTimeout: const Duration(seconds: 10),
      receiveTimeout: const Duration(seconds: 15),
      sendTimeout: const Duration(seconds: 10),
      headers: {'Accept': 'application/json'},
    ),
  )..interceptors.add(
      InterceptorsWrapper(
        onRequest: (options, handler) async {
          final token = await ref.read(tokenStorageProvider).getToken();
          if (token != null && token.isNotEmpty) {
            options.headers['Authorization'] = 'Bearer $token';
          }
          handler.next(options);
        },
      ),
    );
});

final authServiceProvider = Provider<AuthService>((ref) {
  return AuthService(
    dio: ref.watch(_authDioProvider),
    tokenStorage: ref.watch(tokenStorageProvider),
  );
});

final authStateProvider =
    StateNotifierProvider<AuthNotifier, AsyncValue<UserProfile?>>((ref) {
  return AuthNotifier(ref.watch(authServiceProvider));
});

final apiClientProvider = Provider<ApiClient>((ref) {
  final client = ApiClient(
    baseUrl: ApiConfig.baseUrl,
    tokenStorage: ref.watch(tokenStorageProvider),
  );
  client.setUnauthorizedHandler(() async {
    ref.read(authStateProvider.notifier).markLoggedOut();
  });
  return client;
});

class AuthNotifier extends StateNotifier<AsyncValue<UserProfile?>> {
  final AuthService _authService;

  AuthNotifier(this._authService) : super(const AsyncValue.loading()) {
    _bootstrap();
  }

  Future<void> _bootstrap() async {
    try {
      final loggedIn = await _authService.isLoggedIn();
      if (!loggedIn) {
        state = const AsyncValue.data(null);
        return;
      }
      try {
        final user = await _authService.getMe();
        state = AsyncValue.data(user);
      } catch (_) {
        await _authService.logout();
        state = const AsyncValue.data(null);
      }
    } catch (e, st) {
      state = AsyncValue.error(e, st);
    }
  }

  Future<void> bindLicense(String licenseKey) async {
    state = const AsyncValue.loading();
    try {
      final result = await _authService.bindLicense(licenseKey);
      state = AsyncValue.data(result.user);
    } catch (e, st) {
      state = AsyncValue.error(e, st);
    }
  }

  Future<void> logout() async {
    await _authService.logout();
    state = const AsyncValue.data(null);
  }

  void markLoggedOut() {
    state = const AsyncValue.data(null);
  }
}

import 'package:dio/dio.dart';
import 'package:football_insight_app/utils/token_storage.dart';

typedef UnauthorizedCallback = Future<void> Function();

class ApiClient {
  final Dio dio;
  final TokenStorage _tokenStorage;
  UnauthorizedCallback? _onUnauthorized;

  ApiClient({required String baseUrl, required TokenStorage tokenStorage})
      : dio = Dio(
          BaseOptions(
            baseUrl: baseUrl,
            connectTimeout: const Duration(seconds: 10),
            receiveTimeout: const Duration(seconds: 15),
            sendTimeout: const Duration(seconds: 10),
            headers: {'Accept': 'application/json'},
          ),
        ),
        _tokenStorage = tokenStorage {
    dio.interceptors.add(_buildInterceptor());
  }

  void setUnauthorizedHandler(UnauthorizedCallback callback) {
    _onUnauthorized = callback;
  }

  Interceptor _buildInterceptor() {
    return InterceptorsWrapper(
      onRequest: (options, handler) async {
        final token = await _tokenStorage.getToken();
        if (token != null && token.isNotEmpty) {
          options.headers['Authorization'] = 'Bearer $token';
        }
        handler.next(options);
      },
      onError: (error, handler) async {
        if (error.response?.statusCode == 401) {
          await _tokenStorage.clearToken();
          final cb = _onUnauthorized;
          if (cb != null) {
            await cb();
          }
        }
        handler.next(error);
      },
    );
  }
}

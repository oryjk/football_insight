import 'package:dio/dio.dart';
import 'package:football_insight_app/utils/token_storage.dart';

class ApiClient {
  final Dio dio;
  final TokenStorage _tokenStorage;

  ApiClient({required String baseUrl})
      : dio = Dio(BaseOptions(baseUrl: baseUrl)),
        _tokenStorage = TokenStorage() {
    dio.interceptors.add(_authInterceptor());
  }

  Interceptor _authInterceptor() {
    return InterceptorsWrapper(
      onRequest: (options, handler) async {
        final token = await _tokenStorage.getToken();
        if (token != null) {
          options.headers['Authorization'] = 'Bearer $token';
        }
        handler.next(options);
      },
    );
  }
}

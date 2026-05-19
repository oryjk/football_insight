import 'package:dio/dio.dart';
import 'package:football_insight_app/models/user_profile.dart';
import 'package:football_insight_app/utils/token_storage.dart';

class AuthResult {
  final String accessToken;
  final UserProfile user;

  const AuthResult({required this.accessToken, required this.user});
}

class AuthService {
  final Dio _dio;
  final TokenStorage _tokenStorage;

  AuthService({required Dio dio, required TokenStorage tokenStorage})
      : _dio = dio,
        _tokenStorage = tokenStorage;

  Future<AuthResult> bindLicense(String licenseKey) async {
    final response = await _dio.post(
      '/api/v1/auth/bind-license',
      data: {'license_key': licenseKey},
    );
    final data = response.data as Map<String, dynamic>;
    final token = data['access_token'] as String?;
    if (token == null || token.isEmpty) {
      throw const AuthFailure('未拿到 access_token');
    }
    await _tokenStorage.saveToken(token);
    final rawUser = data['user'];
    final user = rawUser is Map<String, dynamic>
        ? UserProfile.fromMap(rawUser)
        : await getMe();
    return AuthResult(accessToken: token, user: user);
  }

  Future<UserProfile> getMe() async {
    final response = await _dio.get('/api/v1/auth/me');
    final data = response.data;
    if (data is! Map<String, dynamic>) {
      throw const AuthFailure('/auth/me 返回格式异常');
    }
    return UserProfile.fromMap(data);
  }

  Future<void> logout() async {
    await _tokenStorage.clearToken();
  }

  Future<bool> isLoggedIn() => _tokenStorage.hasToken();
}

class AuthFailure implements Exception {
  final String message;
  const AuthFailure(this.message);

  @override
  String toString() => 'AuthFailure: $message';
}

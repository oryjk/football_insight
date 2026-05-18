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
    final data = response.data;
    final token = data['access_token'] as String;
    await _tokenStorage.saveToken(token);
    final user = UserProfile.fromMap(data['user']);
    return AuthResult(accessToken: token, user: user);
  }

  Future<UserProfile> getMe() async {
    final response = await _dio.get('/api/v1/auth/me');
    return UserProfile.fromMap(response.data);
  }

  Future<void> logout() async {
    await _tokenStorage.clearToken();
  }

  Future<bool> isLoggedIn() async {
    return _tokenStorage.hasToken();
  }
}

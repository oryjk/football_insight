import 'package:dio/dio.dart';

class PushNotificationService {
  final Dio _dio;
  PushNotificationService({required Dio dio}) : _dio = dio;

  Future<void> registerToken(String token) async {
    await _dio.post('/api/v1/push/register-token', data: {
      'device_token': token,
      'platform': 'jpush',
    });
  }

  Future<void> unregisterToken(String token) async {
    await _dio.delete('/api/v1/push/unregister-token', data: {
      'device_token': token,
    });
  }
}

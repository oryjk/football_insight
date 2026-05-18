import 'package:dio/dio.dart';

class PushNotificationService {
  final Dio _dio;
  PushNotificationService({required Dio dio}) : _dio = dio;

  Future<void> registerToken(String token) async {
    await _dio.post('/api/v1/push/register', data: {'token': token});
  }

  Future<void> unregisterToken(String token) async {
    await _dio.post('/api/v1/push/unregister', data: {'token': token});
  }
}

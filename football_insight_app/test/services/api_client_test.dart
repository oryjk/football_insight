import 'package:flutter_test/flutter_test.dart';
import 'package:football_insight_app/services/api_client.dart';
import 'package:football_insight_app/utils/token_storage.dart';
import 'package:shared_preferences/shared_preferences.dart';

void main() {
  group('ApiClient', () {
    setUp(() {
      SharedPreferences.setMockInitialValues({});
    });

    test('creates dio instance with correct base url and timeouts', () {
      final client = ApiClient(
        baseUrl: 'https://example.com',
        tokenStorage: TokenStorage(),
      );
      expect(client.dio.options.baseUrl, 'https://example.com');
      expect(client.dio.options.connectTimeout, isNotNull);
      expect(client.dio.options.receiveTimeout, isNotNull);
    });
  });
}

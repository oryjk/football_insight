import 'package:flutter_test/flutter_test.dart';
import 'package:football_insight_app/services/api_client.dart';
import 'package:shared_preferences/shared_preferences.dart';

void main() {
  group('ApiClient', () {
    setUp(() {
      SharedPreferences.setMockInitialValues({});
    });

    test('creates dio instance with correct base url', () {
      final client = ApiClient(baseUrl: 'https://example.com');
      expect(client.dio.options.baseUrl, 'https://example.com');
    });
  });
}

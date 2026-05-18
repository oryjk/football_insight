import 'package:flutter_test/flutter_test.dart';
import 'package:football_insight_app/utils/token_storage.dart';
import 'package:shared_preferences/shared_preferences.dart';

void main() {
  group('TokenStorage', () {
    late TokenStorage storage;

    setUp(() {
      SharedPreferences.setMockInitialValues({});
      storage = TokenStorage();
    });

    test('save and retrieve token', () async {
      await storage.saveToken('test-jwt-token');
      final token = await storage.getToken();
      expect(token, 'test-jwt-token');
    });

    test('returns null when no token saved', () async {
      final token = await storage.getToken();
      expect(token, isNull);
    });

    test('clear removes token', () async {
      await storage.saveToken('test-jwt-token');
      await storage.clearToken();
      final token = await storage.getToken();
      expect(token, isNull);
    });

    test('hasToken returns correct state', () async {
      expect(await storage.hasToken(), false);
      await storage.saveToken('test-jwt-token');
      expect(await storage.hasToken(), true);
    });
  });
}

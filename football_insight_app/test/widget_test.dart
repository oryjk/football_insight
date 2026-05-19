import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:football_insight_app/app.dart';
import 'package:shared_preferences/shared_preferences.dart';

void main() {
  testWidgets('App renders login page when not authenticated', (tester) async {
    SharedPreferences.setMockInitialValues({});
    await tester.pumpWidget(
      const ProviderScope(child: FootballInsightApp()),
    );
    // 等 AuthNotifier._bootstrap 完成
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 50));
    await tester.pumpAndSettle(const Duration(seconds: 1));
    expect(find.text('回流监控'), findsOneWidget);
    expect(find.text('绑定登录'), findsOneWidget);
  });
}

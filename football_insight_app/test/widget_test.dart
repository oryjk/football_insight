import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:football_insight_app/app.dart';
import 'package:football_insight_app/providers/ticket_watch_provider.dart';
import 'package:shared_preferences/shared_preferences.dart';

void main() {
  testWidgets('App renders without error', (WidgetTester tester) async {
    SharedPreferences.setMockInitialValues({});
    await tester.pumpWidget(
      ProviderScope(
        overrides: [
          currentBoardProvider.overrideWith((ref) async => {}),
        ],
        child: const FootballInsightApp(),
      ),
    );
    await tester.pump();
    await tester.pump();
    expect(find.byType(FootballInsightApp), findsOneWidget);
  });
}

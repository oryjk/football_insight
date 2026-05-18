import 'package:flutter_test/flutter_test.dart';

import 'package:football_insight_app/app.dart';

void main() {
  testWidgets('App renders placeholder text', (WidgetTester tester) async {
    await tester.pumpWidget(const FootballInsightApp());
    expect(find.text('Football Insight App'), findsOneWidget);
  });
}

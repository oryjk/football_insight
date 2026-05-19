import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:football_insight_app/models/match_card.dart';
import 'package:football_insight_app/widgets/match_card_widget.dart';

void main() {
  group('MatchCardWidget', () {
    const match = MatchCard(
      matchId: 1,
      externalMatchId: 'csl-2026-12',
      roundNumber: 12,
      matchDate: '2026-05-20',
      matchTime: '19:35',
      kickoffAt: '2026-05-20T19:35:00+08:00',
      homeTeamName: '成都蓉城',
      awayTeamName: '上海海港',
    );

    testWidgets('displays team names and round number', (tester) async {
      await tester.pumpWidget(
        const MaterialApp(
          home: Scaffold(body: MatchCardWidget(match: match)),
        ),
      );

      expect(find.text('成都蓉城'), findsOneWidget);
      expect(find.text('上海海港'), findsOneWidget);
      expect(find.text('第12轮'), findsOneWidget);
      expect(find.text('2026-05-20 19:35'), findsOneWidget);
      expect(find.text('VS'), findsOneWidget);
    });

    testWidgets('shows current badge when isCurrent', (tester) async {
      const currentMatch = MatchCard(
        matchId: 1,
        externalMatchId: 'x',
        roundNumber: 12,
        matchDate: '2026-05-20',
        matchTime: '19:35',
        kickoffAt: '',
        homeTeamName: 'A',
        awayTeamName: 'B',
        isCurrent: true,
      );
      await tester.pumpWidget(
        const MaterialApp(
          home: Scaffold(body: MatchCardWidget(match: currentMatch)),
        ),
      );
      expect(find.text('本轮'), findsOneWidget);
    });

    testWidgets('calls onTap when tapped', (tester) async {
      var tapped = false;
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: MatchCardWidget(
              match: match,
              onTap: () => tapped = true,
            ),
          ),
        ),
      );

      await tester.tap(find.byType(InkWell));
      expect(tapped, isTrue);
    });
  });
}

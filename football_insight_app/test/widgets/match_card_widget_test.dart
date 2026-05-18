import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:football_insight_app/models/match_card.dart';
import 'package:football_insight_app/widgets/match_card_widget.dart';

void main() {
  group('MatchCardWidget', () {
    testWidgets('displays team names and round number', (tester) async {
      const match = MatchCard(
        matchId: 1,
        homeTeamName: '成都蓉城',
        awayTeamName: '上海海港',
        matchDate: '2026-05-20',
        matchTime: '19:35',
        roundNumber: 12,
      );

      await tester.pumpWidget(
        const MaterialApp(
          home: Scaffold(
            body: MatchCardWidget(match: match),
          ),
        ),
      );

      expect(find.text('成都蓉城'), findsOneWidget);
      expect(find.text('上海海港'), findsOneWidget);
      expect(find.text('第12轮'), findsOneWidget);
      expect(find.text('2026-05-20 19:35'), findsOneWidget);
      expect(find.text('VS'), findsOneWidget);
    });

    testWidgets('displays scores when available', (tester) async {
      const match = MatchCard(
        matchId: 1,
        homeTeamName: '成都蓉城',
        awayTeamName: '上海海港',
        matchDate: '2026-05-20',
        matchTime: '19:35',
        roundNumber: 12,
        homeScore: 2,
        awayScore: 1,
      );

      await tester.pumpWidget(
        const MaterialApp(
          home: Scaffold(
            body: MatchCardWidget(match: match),
          ),
        ),
      );

      expect(find.text('2 - 1'), findsOneWidget);
      expect(find.text('VS'), findsNothing);
    });

    testWidgets('calls onTap when tapped', (tester) async {
      var tapped = false;
      const match = MatchCard(
        matchId: 1,
        homeTeamName: 'A',
        awayTeamName: 'B',
        matchDate: '2026-01-01',
        matchTime: '15:00',
        roundNumber: 1,
      );

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

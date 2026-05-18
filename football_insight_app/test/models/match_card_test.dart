import 'package:flutter_test/flutter_test.dart';
import 'package:football_insight_app/models/match_card.dart';

void main() {
  group('MatchCard', () {
    test('fromMap creates correct model', () {
      final data = {
        'match_id': 123,
        'home_team_name': '成都蓉城',
        'away_team_name': '上海海港',
        'match_date': '2026-05-20',
        'match_time': '19:35',
        'round_number': 12,
        'home_score': 2,
        'away_score': 1,
        'status': 'finished',
      };
      final match = MatchCard.fromMap(data);
      expect(match.matchId, 123);
      expect(match.homeTeamName, '成都蓉城');
      expect(match.awayTeamName, '上海海港');
      expect(match.matchDate, '2026-05-20');
      expect(match.matchTime, '19:35');
      expect(match.roundNumber, 12);
      expect(match.homeScore, 2);
      expect(match.awayScore, 1);
      expect(match.status, 'finished');
    });

    test('fromMap handles missing optional fields', () {
      final data = {
        'match_id': 456,
        'home_team_name': '北京国安',
        'away_team_name': '山东泰山',
        'match_date': '2026-05-21',
        'match_time': '15:30',
        'round_number': 12,
      };
      final match = MatchCard.fromMap(data);
      expect(match.matchId, 456);
      expect(match.homeScore, isNull);
      expect(match.awayScore, isNull);
      expect(match.status, 'scheduled');
    });
  });
}

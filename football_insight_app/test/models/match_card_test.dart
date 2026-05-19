import 'package:flutter_test/flutter_test.dart';
import 'package:football_insight_app/models/match_card.dart';

void main() {
  group('MatchCard', () {
    test('fromMap creates correct model', () {
      final data = <String, dynamic>{
        'match_id': 123,
        'external_match_id': 'csl-2026-12-cdry-shhg',
        'round_number': 12,
        'sale_start_at': '2026-05-15T12:00:00+08:00',
        'match_date': '2026-05-20',
        'match_time': '19:35',
        'kickoff_at': '2026-05-20T19:35:00+08:00',
        'home_team_name': '成都蓉城',
        'away_team_name': '上海海港',
        'is_current': true,
        'include_in_reflux_stats': true,
      };
      final match = MatchCard.fromMap(data);
      expect(match.matchId, 123);
      expect(match.externalMatchId, 'csl-2026-12-cdry-shhg');
      expect(match.homeTeamName, '成都蓉城');
      expect(match.awayTeamName, '上海海港');
      expect(match.matchDate, '2026-05-20');
      expect(match.matchTime, '19:35');
      expect(match.roundNumber, 12);
      expect(match.isCurrent, isTrue);
      expect(match.includeInRefluxStats, isTrue);
      expect(match.saleStartAt, '2026-05-15T12:00:00+08:00');
    });

    test('fromMap handles missing optional fields', () {
      final data = <String, dynamic>{
        'match_id': 456,
        'home_team_name': '北京国安',
        'away_team_name': '山东泰山',
        'match_date': '2026-05-21',
        'match_time': '15:30',
        'kickoff_at': '2026-05-21T15:30:00+08:00',
        'round_number': 12,
      };
      final match = MatchCard.fromMap(data);
      expect(match.matchId, 456);
      expect(match.saleStartAt, isNull);
      expect(match.isCurrent, isFalse);
      expect(match.externalMatchId, '');
    });
  });
}

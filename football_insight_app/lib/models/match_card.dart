class MatchCard {
  final int matchId;
  final String externalMatchId;
  final int roundNumber;
  final String? saleStartAt;
  final String matchDate;
  final String matchTime;
  final String kickoffAt;
  final String homeTeamName;
  final String awayTeamName;
  final bool isCurrent;
  final bool includeInRefluxStats;

  const MatchCard({
    required this.matchId,
    required this.externalMatchId,
    required this.roundNumber,
    required this.matchDate,
    required this.matchTime,
    required this.kickoffAt,
    required this.homeTeamName,
    required this.awayTeamName,
    this.saleStartAt,
    this.isCurrent = false,
    this.includeInRefluxStats = false,
  });

  factory MatchCard.fromMap(Map<String, dynamic> map) {
    return MatchCard(
      matchId: (map['match_id'] as num).toInt(),
      externalMatchId: map['external_match_id'] as String? ?? '',
      roundNumber: (map['round_number'] as num?)?.toInt() ?? 0,
      saleStartAt: map['sale_start_at'] as String?,
      matchDate: map['match_date'] as String? ?? '',
      matchTime: map['match_time'] as String? ?? '',
      kickoffAt: map['kickoff_at'] as String? ?? '',
      homeTeamName: map['home_team_name'] as String? ?? '',
      awayTeamName: map['away_team_name'] as String? ?? '',
      isCurrent: map['is_current'] as bool? ?? false,
      includeInRefluxStats: map['include_in_reflux_stats'] as bool? ?? false,
    );
  }
}

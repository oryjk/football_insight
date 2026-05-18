class MatchCard {
  final int matchId;
  final String homeTeamName;
  final String awayTeamName;
  final String matchDate;
  final String matchTime;
  final int roundNumber;
  final int? homeScore;
  final int? awayScore;
  final String status;

  const MatchCard({
    required this.matchId,
    required this.homeTeamName,
    required this.awayTeamName,
    required this.matchDate,
    required this.matchTime,
    required this.roundNumber,
    this.homeScore,
    this.awayScore,
    this.status = 'scheduled',
  });

  factory MatchCard.fromMap(Map<String, dynamic> map) {
    return MatchCard(
      matchId: map['match_id'] as int,
      homeTeamName: map['home_team_name'] as String,
      awayTeamName: map['away_team_name'] as String,
      matchDate: map['match_date'] as String,
      matchTime: map['match_time'] as String,
      roundNumber: map['round_number'] as int,
      homeScore: map['home_score'] as int?,
      awayScore: map['away_score'] as int?,
      status: map['status'] as String? ?? 'scheduled',
    );
  }
}

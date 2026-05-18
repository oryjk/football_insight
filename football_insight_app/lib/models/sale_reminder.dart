class SaleReminder {
  final int matchId;
  final String homeTeamName;
  final String awayTeamName;
  final DateTime saleStartAt;
  final bool reminderEnabled;

  const SaleReminder({
    required this.matchId,
    required this.homeTeamName,
    required this.awayTeamName,
    required this.saleStartAt,
    this.reminderEnabled = true,
  });

  String get matchLabel => '$homeTeamName vs $awayTeamName';
}

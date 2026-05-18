class RefluxEvent {
  final String blockName;
  final int ticketCount;
  final DateTime occurredAt;
  final String? matchId;

  const RefluxEvent({
    required this.blockName,
    required this.ticketCount,
    required this.occurredAt,
    this.matchId,
  });

  factory RefluxEvent.fromMap(Map<String, dynamic> map) {
    return RefluxEvent(
      blockName: map['block_name'] as String,
      ticketCount: map['ticket_count'] as int,
      occurredAt: DateTime.parse(map['occurred_at'] as String),
      matchId: map['match_id']?.toString(),
    );
  }

  String get timeLabel {
    final h = occurredAt.hour.toString().padLeft(2, '0');
    final m = occurredAt.minute.toString().padLeft(2, '0');
    return '$h:$m';
  }
}

class TrackedInterest {
  final String blockName;
  final String startedAt;
  final String? firstInventoryAt;

  const TrackedInterest({
    required this.blockName,
    required this.startedAt,
    this.firstInventoryAt,
  });

  factory TrackedInterest.fromMap(Map<String, dynamic> map) {
    return TrackedInterest(
      blockName: map['block_name'] as String? ?? '',
      startedAt: map['started_at'] as String? ?? '',
      firstInventoryAt: map['first_inventory_at'] as String?,
    );
  }

  bool get hasReflux => firstInventoryAt != null;
}

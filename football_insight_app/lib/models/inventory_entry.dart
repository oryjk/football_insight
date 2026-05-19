class InventoryEntry {
  final String blockKey;
  final String blockName;
  final int occurrences;
  final String latestTime;

  const InventoryEntry({
    required this.blockKey,
    required this.blockName,
    required this.occurrences,
    required this.latestTime,
  });

  factory InventoryEntry.fromMap(Map<String, dynamic> map) {
    return InventoryEntry(
      blockKey: map['block_key'] as String? ?? '',
      blockName: map['block_name'] as String? ?? '',
      occurrences: (map['occurrences'] as num?)?.toInt() ?? 0,
      latestTime: map['latest_time'] as String? ?? '',
    );
  }

  DateTime? get latestDateTime => DateTime.tryParse(latestTime);

  String get latestTimeLabel {
    final dt = latestDateTime;
    if (dt == null) return latestTime;
    final h = dt.hour.toString().padLeft(2, '0');
    final m = dt.minute.toString().padLeft(2, '0');
    return '$h:$m';
  }
}

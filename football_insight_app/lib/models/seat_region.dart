class SeatRegion {
  final String blockName;
  final String? priceLevel;
  final int? availableCount;
  final bool isTracked;

  const SeatRegion({
    required this.blockName,
    this.priceLevel,
    this.availableCount,
    this.isTracked = false,
  });

  factory SeatRegion.fromMap(Map<String, dynamic> map) {
    return SeatRegion(
      blockName: map['block_name'] as String,
      priceLevel: map['price_level'] as String?,
      availableCount: map['available_count'] as int?,
      isTracked: map['is_tracked'] as bool? ?? false,
    );
  }

  SeatRegion copyWith({bool? isTracked, int? availableCount}) {
    return SeatRegion(
      blockName: blockName,
      priceLevel: priceLevel,
      availableCount: availableCount ?? this.availableCount,
      isTracked: isTracked ?? this.isTracked,
    );
  }
}

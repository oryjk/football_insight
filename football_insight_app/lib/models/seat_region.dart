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

  SeatRegion copyWith({bool? isTracked, int? availableCount}) {
    return SeatRegion(
      blockName: blockName,
      priceLevel: priceLevel,
      availableCount: availableCount ?? this.availableCount,
      isTracked: isTracked ?? this.isTracked,
    );
  }
}

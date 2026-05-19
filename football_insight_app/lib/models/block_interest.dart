class BlockInterest {
  final String blockName;
  final int interestedUserCount;
  final bool viewerInterested;

  const BlockInterest({
    required this.blockName,
    required this.interestedUserCount,
    required this.viewerInterested,
  });

  factory BlockInterest.fromMap(Map<String, dynamic> map) {
    return BlockInterest(
      blockName: map['block_name'] as String? ?? '',
      interestedUserCount: (map['interested_user_count'] as num?)?.toInt() ?? 0,
      viewerInterested: map['viewer_interested'] as bool? ?? false,
    );
  }

  BlockInterest copyWith({
    String? blockName,
    int? interestedUserCount,
    bool? viewerInterested,
  }) {
    return BlockInterest(
      blockName: blockName ?? this.blockName,
      interestedUserCount: interestedUserCount ?? this.interestedUserCount,
      viewerInterested: viewerInterested ?? this.viewerInterested,
    );
  }
}

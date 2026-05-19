import 'package:football_insight_app/models/block_interest.dart';
import 'package:football_insight_app/models/inventory_entry.dart';
import 'package:football_insight_app/models/match_card.dart';
import 'package:football_insight_app/models/tracked_interest.dart';

class CurrentBoard {
  final MatchCard? currentMatch;
  final bool groupTicketActive;
  final String message;
  final List<InventoryEntry> inventory;
  final List<BlockInterest> blockInterests;
  final List<TrackedInterest> trackedInterests;

  const CurrentBoard({
    required this.currentMatch,
    required this.groupTicketActive,
    required this.message,
    required this.inventory,
    required this.blockInterests,
    required this.trackedInterests,
  });

  factory CurrentBoard.fromMap(Map<String, dynamic> map) {
    final rawMatch = map['current_match'];
    return CurrentBoard(
      currentMatch: rawMatch is Map<String, dynamic>
          ? MatchCard.fromMap(rawMatch)
          : null,
      groupTicketActive: map['group_ticket_active'] as bool? ?? false,
      message: map['message'] as String? ?? '',
      inventory: (map['inventory'] as List<dynamic>? ?? [])
          .map((e) => InventoryEntry.fromMap(e as Map<String, dynamic>))
          .toList(),
      blockInterests: (map['block_interests'] as List<dynamic>? ?? [])
          .map((e) => BlockInterest.fromMap(e as Map<String, dynamic>))
          .toList(),
      trackedInterests: (map['tracked_interests'] as List<dynamic>? ?? [])
          .map((e) => TrackedInterest.fromMap(e as Map<String, dynamic>))
          .toList(),
    );
  }

  static const empty = CurrentBoard(
    currentMatch: null,
    groupTicketActive: false,
    message: '',
    inventory: [],
    blockInterests: [],
    trackedInterests: [],
  );
}

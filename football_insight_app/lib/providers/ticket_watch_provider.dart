import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:football_insight_app/models/block_interest.dart';
import 'package:football_insight_app/models/current_board.dart';
import 'package:football_insight_app/models/inventory_entry.dart';
import 'package:football_insight_app/providers/auth_provider.dart';
import 'package:football_insight_app/services/ticket_watch_service.dart';

final ticketWatchServiceProvider = Provider<TicketWatchService>(
  (ref) => TicketWatchService(dio: ref.watch(apiClientProvider).dio),
);

final currentBoardProvider =
    FutureProvider.autoDispose<CurrentBoard>((ref) async {
  return ref.watch(ticketWatchServiceProvider).getCurrentBoard();
});

final matchInventoryProvider =
    FutureProvider.autoDispose.family<List<InventoryEntry>, int>(
  (ref, matchId) async {
    return ref.watch(ticketWatchServiceProvider).getInventory(matchId);
  },
);

final matchBlockInterestsProvider =
    FutureProvider.autoDispose.family<List<BlockInterest>, int>(
  (ref, matchId) async {
    return ref.watch(ticketWatchServiceProvider).getBlockInterests(matchId);
  },
);

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:football_insight_app/providers/auth_provider.dart';
import 'package:football_insight_app/services/ticket_watch_service.dart';

final ticketWatchServiceProvider = Provider<TicketWatchService>(
  (ref) => TicketWatchService(dio: ref.watch(apiClientProvider).dio),
);
final currentBoardProvider = FutureProvider<Map<String, dynamic>>(
  (ref) async => ref.watch(ticketWatchServiceProvider).getCurrentBoard(),
);
final matchInventoryProvider =
    FutureProvider.family<Map<String, dynamic>, int>(
  (ref, matchId) async =>
      ref.watch(ticketWatchServiceProvider).getInventory(matchId),
);

import 'package:dio/dio.dart';
import 'package:football_insight_app/models/block_interest.dart';
import 'package:football_insight_app/models/current_board.dart';
import 'package:football_insight_app/models/inventory_entry.dart';
import 'package:football_insight_app/models/tracked_interest.dart';

class TicketWatchService {
  final Dio _dio;
  TicketWatchService({required Dio dio}) : _dio = dio;

  Future<CurrentBoard> getCurrentBoard() async {
    final response = await _dio.get('/api/v1/ticket-watch/current-board');
    final data = response.data;
    if (data is! Map<String, dynamic>) return CurrentBoard.empty;
    return CurrentBoard.fromMap(data);
  }

  Future<List<InventoryEntry>> getInventory(
    int matchId, {
    String? since,
    int? fallbackMatchId,
  }) async {
    final qp = <String, dynamic>{};
    if (since != null && since.isNotEmpty) qp['since'] = since;
    if (fallbackMatchId != null) qp['fallback_match_id'] = fallbackMatchId;
    final response = await _dio.get(
      '/api/v1/ticket-watch/matches/$matchId/inventory',
      queryParameters: qp,
    );
    final data = response.data;
    if (data is! List) return [];
    return data
        .whereType<Map<String, dynamic>>()
        .map(InventoryEntry.fromMap)
        .toList();
  }

  Future<List<BlockInterest>> getBlockInterests(int matchId) async {
    final response = await _dio.get(
      '/api/v1/ticket-watch/matches/$matchId/interests',
    );
    final data = response.data;
    if (data is! List) return [];
    return data
        .whereType<Map<String, dynamic>>()
        .map(BlockInterest.fromMap)
        .toList();
  }

  Future<List<TrackedInterest>> getTrackedInterests(int matchId) async {
    final response = await _dio.get(
      '/api/v1/ticket-watch/matches/$matchId/tracked-interests',
    );
    final data = response.data;
    if (data is! List) return [];
    return data
        .whereType<Map<String, dynamic>>()
        .map(TrackedInterest.fromMap)
        .toList();
  }

  Future<BlockInterest> toggleBlockInterest(int matchId, String blockName) async {
    final response = await _dio.post(
      '/api/v1/ticket-watch/matches/$matchId/interests/toggle',
      data: {'block_name': blockName},
    );
    final data = response.data;
    if (data is! Map<String, dynamic>) {
      throw StateError('toggle interest 返回格式异常');
    }
    return BlockInterest.fromMap(data);
  }
}

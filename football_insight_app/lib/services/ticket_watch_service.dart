import 'package:dio/dio.dart';

class TicketWatchService {
  final Dio _dio;
  TicketWatchService({required Dio dio}) : _dio = dio;

  Future<Map<String, dynamic>> getCurrentBoard() async {
    final response = await _dio.get('/api/v1/ticket-watch/current-board');
    return response.data as Map<String, dynamic>;
  }

  Future<Map<String, dynamic>> getInventory(int matchId, {String? since}) async {
    final queryParams = <String, dynamic>{};
    if (since != null) queryParams['since'] = since;
    final response = await _dio.get(
      '/api/v1/ticket-watch/matches/$matchId/inventory',
      queryParameters: queryParams,
    );
    return response.data as Map<String, dynamic>;
  }

  Future<List<dynamic>> getTrackedInterests(int matchId) async {
    final response = await _dio.get(
      '/api/v1/ticket-watch/matches/$matchId/tracked-interests',
    );
    return response.data as List<dynamic>;
  }

  Future<void> toggleBlockInterest(int matchId, String blockName) async {
    await _dio.post(
      '/api/v1/ticket-watch/matches/$matchId/interests/toggle',
      data: {'block_name': blockName},
    );
  }

  Future<List<dynamic>> getRegions() async {
    final response = await _dio.get('/api/v1/ticket-watch/regions');
    return response.data as List<dynamic>;
  }
}

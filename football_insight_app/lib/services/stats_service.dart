import 'package:football_insight_app/models/reflux_stats.dart';

class StatsService {
  Future<RefluxStats> getRefluxStats() async {
    return RefluxStats.fromMockData();
  }
}

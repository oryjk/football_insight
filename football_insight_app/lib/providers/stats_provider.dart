import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:football_insight_app/models/reflux_stats.dart';
import 'package:football_insight_app/services/stats_service.dart';

final statsServiceProvider = Provider<StatsService>((ref) => StatsService());
final statsProvider = FutureProvider<RefluxStats>(
  (ref) async => ref.watch(statsServiceProvider).getRefluxStats(),
);

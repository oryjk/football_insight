import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:football_insight_app/models/reflux_event.dart';
import 'package:football_insight_app/models/seat_region.dart';
import 'package:football_insight_app/providers/ticket_watch_provider.dart';
import 'package:football_insight_app/widgets/reflux_timeline_widget.dart';
import 'package:football_insight_app/widgets/region_selector_widget.dart';

class MatchDetailPage extends ConsumerStatefulWidget {
  final int matchId;

  const MatchDetailPage({super.key, required this.matchId});

  @override
  ConsumerState<MatchDetailPage> createState() => _MatchDetailPageState();
}

class _MatchDetailPageState extends ConsumerState<MatchDetailPage> {
  List<SeatRegion> _regions = [];
  List<RefluxEvent> _events = [];

  @override
  void initState() {
    super.initState();
    _loadData();
  }

  Future<void> _loadData() async {
    try {
      final service = ref.read(ticketWatchServiceProvider);
      final inventoryFuture = service.getInventory(widget.matchId);
      final regionsFuture = service.getRegions();

      final inventory = await inventoryFuture;
      final regionsData = await regionsFuture;

      if (mounted) {
        setState(() {
          _events =
              (inventory['events'] as List<dynamic>?)
                  ?.map((e) => RefluxEvent.fromMap(e as Map<String, dynamic>))
                  .toList() ??
                  [];
          _regions =
              regionsData
                  .map((r) => SeatRegion.fromMap(r as Map<String, dynamic>))
                  .toList();
        });
      }
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(
          context,
        ).showSnackBar(SnackBar(content: Text('加载失败: $e')));
      }
    }
  }

  Future<void> _toggleRegion(String blockName) async {
    try {
      final service = ref.read(ticketWatchServiceProvider);
      await service.toggleBlockInterest(widget.matchId, blockName);
      setState(() {
        _regions =
            _regions.map((r) {
              if (r.blockName == blockName) {
                return r.copyWith(isTracked: !r.isTracked);
              }
              return r;
            }).toList();
      });
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(
          context,
        ).showSnackBar(SnackBar(content: Text('操作失败: $e')));
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('比赛详情')),
      body: RefreshIndicator(
        onRefresh: _loadData,
        child: ListView(
          padding: const EdgeInsets.all(16),
          children: [
            Text(
              '关注区域',
              style: Theme.of(context).textTheme.titleMedium,
            ),
            const SizedBox(height: 8),
            RegionSelectorWidget(
              regions: _regions,
              onToggle: _toggleRegion,
            ),
            const Divider(height: 32),
            Text(
              '回流时间线',
              style: Theme.of(context).textTheme.titleMedium,
            ),
            const SizedBox(height: 8),
            RefluxTimelineWidget(events: _events),
          ],
        ),
      ),
    );
  }
}

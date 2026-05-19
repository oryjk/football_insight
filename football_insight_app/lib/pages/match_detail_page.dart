import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:football_insight_app/providers/ticket_watch_provider.dart';
import 'package:football_insight_app/widgets/reflux_timeline_widget.dart';
import 'package:football_insight_app/widgets/region_selector_widget.dart';

class MatchDetailPage extends ConsumerWidget {
  final int matchId;

  const MatchDetailPage({super.key, required this.matchId});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final inventoryAsync = ref.watch(matchInventoryProvider(matchId));
    final interestsAsync = ref.watch(matchBlockInterestsProvider(matchId));
    final theme = Theme.of(context);

    return Scaffold(
      appBar: AppBar(title: const Text('比赛详情')),
      body: RefreshIndicator(
        onRefresh: () async {
          ref.invalidate(matchInventoryProvider(matchId));
          ref.invalidate(matchBlockInterestsProvider(matchId));
          await Future.wait([
            ref.read(matchInventoryProvider(matchId).future),
            ref.read(matchBlockInterestsProvider(matchId).future),
          ]);
        },
        child: ListView(
          padding: const EdgeInsets.all(16),
          children: [
            Text('关注区域', style: theme.textTheme.titleMedium),
            const SizedBox(height: 8),
            interestsAsync.when(
              loading: () => const Padding(
                padding: EdgeInsets.all(16),
                child: Center(child: CircularProgressIndicator()),
              ),
              error: (e, _) => Padding(
                padding: const EdgeInsets.all(16),
                child: Text('加载失败: $e'),
              ),
              data: (interests) => RegionSelectorWidget(
                interests: interests,
                onToggle: (blockName) => _toggle(ref, context, blockName),
              ),
            ),
            const Divider(height: 32),
            Text('回流时间线', style: theme.textTheme.titleMedium),
            const SizedBox(height: 8),
            inventoryAsync.when(
              loading: () => const Padding(
                padding: EdgeInsets.all(16),
                child: Center(child: CircularProgressIndicator()),
              ),
              error: (e, _) => Padding(
                padding: const EdgeInsets.all(16),
                child: Text('加载失败: $e'),
              ),
              data: (entries) => RefluxTimelineWidget(entries: entries),
            ),
          ],
        ),
      ),
    );
  }

  Future<void> _toggle(
    WidgetRef ref,
    BuildContext context,
    String blockName,
  ) async {
    final messenger = ScaffoldMessenger.of(context);
    try {
      await ref
          .read(ticketWatchServiceProvider)
          .toggleBlockInterest(matchId, blockName);
      ref.invalidate(matchBlockInterestsProvider(matchId));
    } catch (e) {
      messenger.showSnackBar(SnackBar(content: Text('操作失败: $e')));
    }
  }
}

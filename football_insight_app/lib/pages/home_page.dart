import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:football_insight_app/models/match_card.dart';
import 'package:football_insight_app/models/reflux_event.dart';
import 'package:football_insight_app/providers/ticket_watch_provider.dart';
import 'package:football_insight_app/widgets/match_card_widget.dart';
import 'package:football_insight_app/widgets/notification_banner.dart';
import 'package:go_router/go_router.dart';

class HomePage extends ConsumerWidget {
  const HomePage({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final boardAsync = ref.watch(currentBoardProvider);

    return Scaffold(
      appBar: AppBar(
        title: const Text('回流监控'),
        actions: [
          IconButton(
            icon: const Icon(Icons.settings),
            onPressed: () => context.go('/settings'),
          ),
        ],
      ),
      body: boardAsync.when(
        loading: () => const Center(child: CircularProgressIndicator()),
        error: (e, _) => Center(child: Text('加载失败: $e')),
        data: (board) {
          final matchesData = board['matches'] as List<dynamic>?;
          final notification = board['notification'] as String?;
          final matches = matchesData
                  ?.map((m) => MatchCard.fromMap(m as Map<String, dynamic>))
                  .toList() ??
              [];

          return RefreshIndicator(
            onRefresh: () => ref.refresh(currentBoardProvider.future),
            child: CustomScrollView(
              slivers: [
                if (notification != null)
                  SliverToBoxAdapter(
                    child: NotificationBanner(message: notification),
                  ),
                if (matches.isEmpty)
                  const SliverFillRemaining(
                    child: Center(child: Text('暂无比赛数据')),
                  )
                else
                  SliverList(
                    delegate: SliverChildBuilderDelegate(
                      (context, index) {
                        final match = matches[index];
                        return MatchCardWidget(
                          match: match,
                          onTap: () => context.go(
                            '/match-detail',
                            extra: {'matchId': match.matchId},
                          ),
                        );
                      },
                      childCount: matches.length,
                    ),
                  ),
                SliverToBoxAdapter(
                  child: _buildRefluxSummary(context, board),
                ),
              ],
            ),
          );
        },
      ),
    );
  }

  Widget _buildRefluxSummary(BuildContext context, Map<String, dynamic> board) {
    final items = board['inventory_items'] as List<dynamic>?;
    if (items == null || items.isEmpty) return const SizedBox.shrink();

    return Padding(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            '最近回流',
            style: Theme.of(context).textTheme.titleMedium,
          ),
          const SizedBox(height: 8),
          ...items.take(5).map((item) {
            final event = RefluxEvent.fromMap(item as Map<String, dynamic>);
            return ListTile(
              dense: true,
              leading: const Icon(Icons.confirmation_num, size: 20),
              title: Text(event.blockName),
              subtitle: Text('${event.ticketCount}张 · ${event.timeLabel}'),
            );
          }),
        ],
      ),
    );
  }
}

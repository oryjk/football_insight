import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:football_insight_app/models/current_board.dart';
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
        error: (e, _) => _ErrorView(
          message: '加载失败: $e',
          onRetry: () => ref.invalidate(currentBoardProvider),
        ),
        data: (board) => _BoardView(board: board, ref: ref),
      ),
    );
  }
}

class _BoardView extends StatelessWidget {
  final CurrentBoard board;
  final WidgetRef ref;

  const _BoardView({required this.board, required this.ref});

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final match = board.currentMatch;
    final inventory = board.inventory;

    return RefreshIndicator(
      onRefresh: () async {
        ref.invalidate(currentBoardProvider);
        await ref.read(currentBoardProvider.future);
      },
      child: ListView(
        children: [
          if (board.message.isNotEmpty)
            NotificationBanner(message: board.message),
          if (match == null)
            const Padding(
              padding: EdgeInsets.all(32),
              child: Center(child: Text('当前没有进行中的比赛')),
            )
          else
            MatchCardWidget(
              match: match,
              onTap: () => context.go(
                '/match-detail',
                extra: {'matchId': match.matchId},
              ),
            ),
          if (board.groupTicketActive)
            const Padding(
              padding: EdgeInsets.symmetric(horizontal: 16, vertical: 4),
              child: NotificationBanner(
                message: '团购票正在售卖中',
                icon: Icons.group,
              ),
            ),
          const SizedBox(height: 8),
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16),
            child: Text('最近回流', style: theme.textTheme.titleMedium),
          ),
          if (inventory.isEmpty)
            const Padding(
              padding: EdgeInsets.all(24),
              child: Center(child: Text('暂无回流记录')),
            )
          else
            ...inventory.take(8).map((entry) {
              return ListTile(
                dense: true,
                leading: const Icon(Icons.confirmation_num, size: 20),
                title: Text(entry.blockName),
                subtitle: Text('${entry.occurrences} 次 · ${entry.latestTimeLabel}'),
              );
            }),
          const SizedBox(height: 16),
        ],
      ),
    );
  }
}

class _ErrorView extends StatelessWidget {
  final String message;
  final VoidCallback onRetry;

  const _ErrorView({required this.message, required this.onRetry});

  @override
  Widget build(BuildContext context) {
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(24),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            const Icon(Icons.cloud_off, size: 48),
            const SizedBox(height: 12),
            Text(message, textAlign: TextAlign.center),
            const SizedBox(height: 16),
            FilledButton(onPressed: onRetry, child: const Text('重试')),
          ],
        ),
      ),
    );
  }
}

import 'package:flutter/material.dart';
import 'package:football_insight_app/models/inventory_entry.dart';

class RefluxTimelineWidget extends StatelessWidget {
  final List<InventoryEntry> entries;

  const RefluxTimelineWidget({super.key, required this.entries});

  @override
  Widget build(BuildContext context) {
    if (entries.isEmpty) {
      return const Padding(
        padding: EdgeInsets.all(24),
        child: Center(child: Text('暂无回流记录')),
      );
    }

    return ListView.builder(
      shrinkWrap: true,
      physics: const NeverScrollableScrollPhysics(),
      itemCount: entries.length,
      itemBuilder: (context, index) {
        final entry = entries[index];
        return ListTile(
          dense: true,
          leading: const Icon(Icons.confirmation_num, size: 20),
          title: Text(entry.blockName),
          trailing: Text(
            '${entry.occurrences} 次',
            style: Theme.of(context).textTheme.bodySmall,
          ),
          subtitle: Text(entry.latestTimeLabel),
        );
      },
    );
  }
}

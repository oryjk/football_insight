import 'package:flutter/material.dart';
import 'package:football_insight_app/models/reflux_event.dart';

class RefluxTimelineWidget extends StatelessWidget {
  final List<RefluxEvent> events;

  const RefluxTimelineWidget({super.key, required this.events});

  @override
  Widget build(BuildContext context) {
    if (events.isEmpty) {
      return const Padding(
        padding: EdgeInsets.all(24),
        child: Center(child: Text('暂无回流记录')),
      );
    }

    return ListView.builder(
      shrinkWrap: true,
      physics: const NeverScrollableScrollPhysics(),
      itemCount: events.length,
      itemBuilder: (context, index) {
        final event = events[index];
        return ListTile(
          dense: true,
          leading: const Icon(Icons.confirmation_num, size: 20),
          title: Text(event.blockName),
          trailing: Text(
            '${event.ticketCount}张',
            style: Theme.of(context).textTheme.bodySmall,
          ),
          subtitle: Text(event.timeLabel),
        );
      },
    );
  }
}

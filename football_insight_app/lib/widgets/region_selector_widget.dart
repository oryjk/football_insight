import 'package:flutter/material.dart';
import 'package:football_insight_app/models/block_interest.dart';

class RegionSelectorWidget extends StatelessWidget {
  final List<BlockInterest> interests;
  final ValueChanged<String> onToggle;

  const RegionSelectorWidget({
    super.key,
    required this.interests,
    required this.onToggle,
  });

  @override
  Widget build(BuildContext context) {
    if (interests.isEmpty) {
      return const Padding(
        padding: EdgeInsets.all(16),
        child: Text('暂无可关注区域'),
      );
    }

    return Wrap(
      spacing: 8,
      runSpacing: 4,
      children: interests.map((interest) {
        return FilterChip(
          label: Text(
            interest.interestedUserCount > 0
                ? '${interest.blockName} · ${interest.interestedUserCount}'
                : interest.blockName,
          ),
          selected: interest.viewerInterested,
          onSelected: (_) => onToggle(interest.blockName),
        );
      }).toList(),
    );
  }
}

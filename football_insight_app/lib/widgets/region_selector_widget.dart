import 'package:flutter/material.dart';
import 'package:football_insight_app/models/seat_region.dart';

class RegionSelectorWidget extends StatelessWidget {
  final List<SeatRegion> regions;
  final ValueChanged<String> onToggle;

  const RegionSelectorWidget({
    super.key,
    required this.regions,
    required this.onToggle,
  });

  @override
  Widget build(BuildContext context) {
    if (regions.isEmpty) {
      return const Padding(
        padding: EdgeInsets.all(16),
        child: Text('暂无区域数据'),
      );
    }

    return Wrap(
      spacing: 8,
      runSpacing: 4,
      children: regions.map((region) {
        return FilterChip(
          label: Text(region.blockName),
          selected: region.isTracked,
          onSelected: (_) => onToggle(region.blockName),
        );
      }).toList(),
    );
  }
}

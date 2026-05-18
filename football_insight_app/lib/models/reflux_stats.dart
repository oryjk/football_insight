class HourlyRefluxCount {
  final int hour;
  final int totalCount;

  const HourlyRefluxCount({required this.hour, required this.totalCount});
}

class RefluxStats {
  final List<HourlyRefluxCount> hourlyBreakdown;
  final int totalRefluxEvents;
  final String? peakHourLabel;

  const RefluxStats({
    required this.hourlyBreakdown,
    required this.totalRefluxEvents,
    this.peakHourLabel,
  });

  static RefluxStats fromMockData() {
    final hours = List.generate(
      24,
      (i) => HourlyRefluxCount(
        hour: i,
        totalCount: i >= 9 && i <= 21 ? (i - 8) * 3 : 0,
      ),
    );
    return RefluxStats(
      hourlyBreakdown: hours,
      totalRefluxEvents: hours.fold(0, (sum, h) => sum + h.totalCount),
      peakHourLabel: '20:00-21:00',
    );
  }
}

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:football_insight_app/pages/home_page.dart';
import 'package:football_insight_app/pages/login_page.dart';
import 'package:football_insight_app/pages/match_detail_page.dart';
import 'package:football_insight_app/pages/settings_page.dart';
import 'package:football_insight_app/pages/stats_page.dart';
import 'package:football_insight_app/providers/auth_provider.dart';
import 'package:go_router/go_router.dart';

final routerProvider = Provider<GoRouter>((ref) {
  final authState = ref.watch(authStateProvider);
  final isLoggedIn = authState.valueOrNull != null;
  final isLoading = authState.isLoading;

  return GoRouter(
    initialLocation: '/',
    redirect: (context, state) {
      if (isLoading) {
        return state.matchedLocation == '/login' ? null : '/login';
      }
      final isLoginRoute = state.matchedLocation == '/login';
      if (!isLoggedIn && !isLoginRoute) return '/login';
      if (isLoggedIn && isLoginRoute) return '/';
      return null;
    },
    routes: [
      GoRoute(
        path: '/login',
        builder: (context, state) => const LoginPage(),
      ),
      GoRoute(
        path: '/',
        builder: (context, state) => const HomePage(),
      ),
      GoRoute(
        path: '/match-detail',
        builder: (context, state) {
          final extra = state.extra as Map<String, dynamic>?;
          final matchId = extra?['matchId'] as int? ?? 0;
          return MatchDetailPage(matchId: matchId);
        },
      ),
      GoRoute(
        path: '/stats',
        builder: (context, state) => const StatsPage(),
      ),
      GoRoute(
        path: '/settings',
        builder: (context, state) => const SettingsPage(),
      ),
    ],
  );
});

class FootballInsightApp extends ConsumerWidget {
  const FootballInsightApp({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final router = ref.watch(routerProvider);
    return MaterialApp.router(
      title: '回流监控',
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(seedColor: Colors.deepPurple),
        useMaterial3: true,
      ),
      routerConfig: router,
    );
  }
}

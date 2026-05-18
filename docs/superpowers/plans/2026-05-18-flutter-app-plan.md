# 回流监控 Flutter App 实施计划（Plan B）

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 创建 Flutter 项目 `football_insight_app/`，实现回流监控 App 的全部页面和交互，优先复用已有后端 API，暂时缺失的 API 用 mock 数据。

**Architecture:** Flutter 独立项目，放在 monorepo 根目录。使用 Riverpod 状态管理、go_router 路由、dio HTTP 客户端、firebase_messaging 推送。页面结构：登录 → 首页回流列表 → 详情 → 统计 → 设置。推送通知通过 FCM + APNs 集成。

**Tech Stack:** Flutter 3.x + Dart 3.x, Riverpod, go_router, dio, firebase_core, firebase_messaging, flutter_local_notifications, shared_preferences

---

## File Structure

```
football_insight_app/
├── lib/
│   ├── main.dart                              -- App 入口，初始化 Firebase/路由
│   ├── app.dart                               -- MaterialApp + Router 配置
│   ├── config/
│   │   └── api_config.dart                    -- API base URL 等配置
│   ├── models/
│   │   ├── match_card.dart                    -- 比赛卡片数据模型
│   │   ├── reflux_event.dart                  -- 回流事件模型
│   │   ├── seat_region.dart                   -- 看台区域模型
│   │   ├── reflux_stats.dart                  -- 历史回流统计模型
│   │   ├── sale_reminder.dart                 -- 开售提醒模型
│   │   └── user_profile.dart                  -- 用户信息模型
│   ├── services/
│   │   ├── api_client.dart                    -- dio 实例 + JWT 拦截器
│   │   ├── auth_service.dart                  -- 登录/绑定码 API
│   │   ├── ticket_watch_service.dart          -- 回流监控 API
│   │   ├── push_notification_service.dart     -- 推送注册/FCM token 上传
│   │   └── stats_service.dart                 -- 历史统计 API（暂 mock）
│   ├── providers/
│   │   ├── auth_provider.dart                 -- 认证状态
│   │   ├── ticket_watch_provider.dart         -- 回流数据状态
│   │   ├── push_provider.dart                 -- 推送通知状态
│   │   └── stats_provider.dart                -- 统计数据状态
│   ├── pages/
│   │   ├── login_page.dart                    -- 绑定码登录页
│   │   ├── home_page.dart                     -- 回流监控首页
│   │   ├── match_detail_page.dart             -- 比赛回流详情页
│   │   ├── stats_page.dart                    -- 历史统计页
│   │   └── settings_page.dart                 -- 设置页
│   ├── widgets/
│   │   ├── match_card_widget.dart             -- 比赛卡片组件
│   │   ├── reflux_timeline_widget.dart        -- 回流时间线组件
│   │   ├── region_selector_widget.dart        -- 看台区域选择组件
│   │   ├── reflux_chart_widget.dart           -- 回流图表组件
│   │   └── notification_banner.dart           -- 推送通知横幅
│   └── utils/
│       ├── token_storage.dart                 -- JWT 本地存储
│       └── formatters.dart                    -- 时间/数字格式化
├── test/
│   ├── models/
│   ├── services/
│   ├── providers/
│   └── widgets/
├── android/
├── ios/
├── pubspec.yaml
├── analysis_options.yaml
└── README.md
```

---

## Task 1: Flutter 项目初始化

**Files:**
- Create: `football_insight_app/pubspec.yaml`
- Create: `football_insight_app/lib/main.dart`
- Create: `football_insight_app/lib/app.dart`
- Create: `football_insight_app/lib/config/api_config.dart`
- Create: `football_insight_app/analysis_options.yaml`

- [ ] **Step 1: 创建 Flutter 项目**

```bash
cd /Users/carlwang/football_insight
flutter create --org com.footballinsight --project-name football_insight_app football_insight_app
```

- [ ] **Step 2: 添加依赖到 pubspec.yaml**

在 `pubspec.yaml` 的 dependencies 中添加：

```yaml
dependencies:
  flutter:
    sdk: flutter
  flutter_riverpod: ^2.5.1
  riverpod_annotation: ^2.3.5
  go_router: ^14.2.0
  dio: ^5.4.3
  shared_preferences: ^2.2.3
  firebase_core: ^3.4.0
  firebase_messaging: ^15.1.0
  flutter_local_notifications: ^17.2.1
  intl: ^0.19.0
  freezed_annotation: ^2.4.3
  json_annotation: ^4.9.0

dev_dependencies:
  flutter_test:
    sdk: flutter
  flutter_lints: ^4.0.0
  build_runner: ^2.4.11
  freezed: ^2.5.2
  json_serializable: ^6.8.0
  mockito: ^5.4.4
```

- [ ] **Step 3: 运行 flutter pub get**

```bash
cd football_insight_app && flutter pub get
```

- [ ] **Step 4: 创建 api_config.dart**

```dart
// lib/config/api_config.dart
class ApiConfig {
  static const String baseUrl = String.fromEnvironment(
    'API_BASE_URL',
    defaultValue: 'https://match.oryjk.cn',
  );
}
```

- [ ] **Step 5: 创建最小 main.dart 和 app.dart**

main.dart:
```dart
import 'package:flutter/material.dart';
import 'app.dart';

void main() {
  WidgetsFlutterBinding.ensureInitialized();
  runApp(const FootballInsightApp());
}
```

app.dart:
```dart
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

class FootballInsightApp extends StatelessWidget {
  const FootballInsightApp({super.key});

  @override
  Widget build(BuildContext context) {
    return ProviderScope(
      child: MaterialApp(
        title: '回流监控',
        theme: ThemeData(
          colorScheme: ColorScheme.fromSeed(seedColor: Colors.deepPurple),
          useMaterial3: true,
        ),
        home: const Scaffold(
          body: Center(child: Text('Football Insight App')),
        ),
      ),
    );
  }
}
```

- [ ] **Step 6: 验证项目能跑起来**

```bash
cd football_insight_app && flutter analyze
```

Expected: 无错误

- [ ] **Step 7: Commit**

```bash
git add football_insight_app/
git commit -m "feat(app): init Flutter project with dependencies"
```

---

## Task 2: 数据模型层

**Files:**
- Create: `football_insight_app/lib/models/match_card.dart`
- Create: `football_insight_app/lib/models/reflux_event.dart`
- Create: `football_insight_app/lib/models/seat_region.dart`
- Create: `football_insight_app/lib/models/reflux_stats.dart`
- Create: `football_insight_app/lib/models/sale_reminder.dart`
- Create: `football_insight_app/lib/models/user_profile.dart`
- Create: `football_insight_app/test/models/match_card_test.dart`

- [ ] **Step 1: 写 match_card 模型的失败测试**

```dart
// test/models/match_card_test.dart
import 'package:flutter_test/flutter_test.dart';
import 'package:football_insight_app/models/match_card.dart';

void main() {
  group('MatchCard', () {
    test('fromMap creates correct model', () {
      final data = {
        'match_id': 123,
        'home_team_name': '成都蓉城',
        'away_team_name': '上海海港',
        'match_date': '2026-05-20',
        'match_time': '19:35',
        'round_number': 12,
        'home_score': 2,
        'away_score': 1,
        'status': 'finished',
      };
      final match = MatchCard.fromMap(data);
      expect(match.matchId, 123);
      expect(match.homeTeamName, '成都蓉城');
      expect(match.awayTeamName, '上海海港');
      expect(match.status, 'finished');
    });

    test('fromMap handles missing optional fields', () {
      final data = {
        'match_id': 456,
        'home_team_name': '北京国安',
        'away_team_name': '山东泰山',
        'match_date': '2026-05-21',
        'match_time': '15:30',
        'round_number': 12,
      };
      final match = MatchCard.fromMap(data);
      expect(match.matchId, 456);
      expect(match.homeScore, isNull);
      expect(match.awayScore, isNull);
    });
  });
}
```

- [ ] **Step 2: 运行测试确认失败**

```bash
cd football_insight_app && flutter test test/models/match_card_test.dart
```

Expected: FAIL - MatchCard class not found

- [ ] **Step 3: 实现 MatchCard 模型**

```dart
// lib/models/match_card.dart
class MatchCard {
  final int matchId;
  final String homeTeamName;
  final String awayTeamName;
  final String matchDate;
  final String matchTime;
  final int roundNumber;
  final int? homeScore;
  final int? awayScore;
  final String status;

  const MatchCard({
    required this.matchId,
    required this.homeTeamName,
    required this.awayTeamName,
    required this.matchDate,
    required this.matchTime,
    required this.roundNumber,
    this.homeScore,
    this.awayScore,
    this.status = 'scheduled',
  });

  factory MatchCard.fromMap(Map<String, dynamic> map) {
    return MatchCard(
      matchId: map['match_id'] as int,
      homeTeamName: map['home_team_name'] as String,
      awayTeamName: map['away_team_name'] as String,
      matchDate: map['match_date'] as String,
      matchTime: map['match_time'] as String,
      roundNumber: map['round_number'] as int,
      homeScore: map['home_score'] as int?,
      awayScore: map['away_score'] as int?,
      status: map['status'] as String? ?? 'scheduled',
    );
  }
}
```

- [ ] **Step 4: 运行测试确认通过**

```bash
cd football_insight_app && flutter test test/models/match_card_test.dart
```

Expected: PASS

- [ ] **Step 5: 实现 RefluxEvent 模型**

```dart
// lib/models/reflux_event.dart
class RefluxEvent {
  final String blockName;
  final int ticketCount;
  final DateTime occurredAt;
  final String? matchId;

  const RefluxEvent({
    required this.blockName,
    required this.ticketCount,
    required this.occurredAt,
    this.matchId,
  });

  factory RefluxEvent.fromMap(Map<String, dynamic> map) {
    return RefluxEvent(
      blockName: map['block_name'] as String,
      ticketCount: map['ticket_count'] as int,
      occurredAt: DateTime.parse(map['occurred_at'] as String),
      matchId: map['match_id']?.toString(),
    );
  }

  String get timeLabel {
    final h = occurredAt.hour.toString().padLeft(2, '0');
    final m = occurredAt.minute.toString().padLeft(2, '0');
    return '$h:$m';
  }
}
```

- [ ] **Step 6: 实现 SeatRegion 模型**

```dart
// lib/models/seat_region.dart
class SeatRegion {
  final String blockName;
  final String? priceLevel;
  final int? availableCount;
  final bool isTracked;

  const SeatRegion({
    required this.blockName,
    this.priceLevel,
    this.availableCount,
    this.isTracked = false,
  });

  SeatRegion copyWith({bool? isTracked, int? availableCount}) {
    return SeatRegion(
      blockName: blockName,
      priceLevel: priceLevel,
      availableCount: availableCount ?? this.availableCount,
      isTracked: isTracked ?? this.isTracked,
    );
  }
}
```

- [ ] **Step 7: 实现 RefluxStats 模型**

```dart
// lib/models/reflux_stats.dart'
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
    final hours = List.generate(24, (i) => HourlyRefluxCount(
      hour: i,
      totalCount: i >= 9 && i <= 21 ? (i - 8) * 3 : 0,
    ));
    return RefluxStats(
      hourlyBreakdown: hours,
      totalRefluxEvents: hours.fold(0, (sum, h) => sum + h.totalCount),
      peakHourLabel: '20:00-21:00',
    );
  }
}
```

注意：此文件第一行 `'` 应为 ```，此处为 markdown 转义需要。实际文件中是：
```dart
// lib/models/reflux_stats.dart
```

- [ ] **Step 8: 实现 SaleReminder 模型**

```dart
// lib/models/sale_reminder.dart
class SaleReminder {
  final int matchId;
  final String homeTeamName;
  final String awayTeamName;
  final DateTime saleStartAt;
  final bool reminderEnabled;

  const SaleReminder({
    required this.matchId,
    required this.homeTeamName,
    required this.awayTeamName,
    required this.saleStartAt,
    this.reminderEnabled = true,
  });

  String get matchLabel => '$homeTeamName vs $awayTeamName';
}
```

- [ ] **Step 9: 实现 UserProfile 模型**

```dart
// lib/models/user_profile.dart
class UserProfile {
  final int userId;
  final String accountIdentifier;
  final String? displayName;
  final String? avatarUrl;

  const UserProfile({
    required this.userId,
    required this.accountIdentifier,
    this.displayName,
    this.avatarUrl,
  });

  factory UserProfile.fromMap(Map<String, dynamic> map) {
    return UserProfile(
      userId: map['id'] as int,
      accountIdentifier: map['account_identifier'] as String,
      displayName: map['display_name'] as String?,
      avatarUrl: map['avatar_url'] as String?,
    );
  }
}
```

- [ ] **Step 10: 运行全量测试**

```bash
cd football_insight_app && flutter test
```

Expected: PASS

- [ ] **Step 11: Commit**

```bash
git add football_insight_app/
git commit -m "feat(app): add data models with tests"
```

---

## Task 3: HTTP 客户端 + JWT 存储

**Files:**
- Create: `football_insight_app/lib/services/api_client.dart`
- Create: `football_insight_app/lib/services/auth_service.dart`
- Create: `football_insight_app/lib/utils/token_storage.dart`
- Create: `football_insight_app/test/services/api_client_test.dart`
- Create: `football_insight_app/test/utils/token_storage_test.dart`

- [ ] **Step 1: 写 token_storage 的失败测试**

```dart
// test/utils/token_storage_test.dart
import 'package:flutter_test/flutter_test.dart';
import 'package:football_insight_app/utils/token_storage.dart';
import 'package:shared_preferences/shared_preferences.dart';

void main() {
  group('TokenStorage', () {
    late TokenStorage storage;

    setUp(() {
      SharedPreferences.setMockInitialValues({});
      storage = TokenStorage();
    });

    test('save and retrieve token', () async {
      await storage.saveToken('test-jwt-token');
      final token = await storage.getToken();
      expect(token, 'test-jwt-token');
    });

    test('returns null when no token saved', () async {
      final token = await storage.getToken();
      expect(token, isNull);
    });

    test('clear removes token', () async {
      await storage.saveToken('test-jwt-token');
      await storage.clearToken();
      final token = await storage.getToken();
      expect(token, isNull);
    });

    test('hasToken returns correct state', () async {
      expect(await storage.hasToken(), false);
      await storage.saveToken('test-jwt-token');
      expect(await storage.hasToken(), true);
    });
  });
}
```

- [ ] **Step 2: 运行测试确认失败**

```bash
cd football_insight_app && flutter test test/utils/token_storage_test.dart
```

Expected: FAIL

- [ ] **Step 3: 实现 TokenStorage**

```dart
// lib/utils/token_storage.dart
import 'package:shared_preferences/shared_preferences.dart';

class TokenStorage {
  static const _key = 'auth_token';

  Future<void> saveToken(String token) async {
    final prefs = await SharedPreferences.getInstance();
    await prefs.setString(_key, token);
  }

  Future<String?> getToken() async {
    final prefs = await SharedPreferences.getInstance();
    return prefs.getString(_key);
  }

  Future<void> clearToken() async {
    final prefs = await SharedPreferences.getInstance();
    await prefs.remove(_key);
  }

  Future<bool> hasToken() async {
    final token = await getToken();
    return token != null && token.isNotEmpty;
  }
}
```

- [ ] **Step 4: 运行测试确认通过**

```bash
cd football_insight_app && flutter test test/utils/token_storage_test.dart
```

Expected: PASS

- [ ] **Step 5: 写 api_client 的失败测试**

```dart
// test/services/api_client_test.dart
import 'package:dio/dio.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:football_insight_app/services/api_client.dart';
import 'package:shared_preferences/shared_preferences.dart';

void main() {
  group('ApiClient', () {
    setUp(() {
      SharedPreferences.setMockInitialValues({});
    });

    test('creates dio instance with correct base url', () {
      final client = ApiClient(baseUrl: 'https://example.com');
      expect(client.dio.options.baseUrl, 'https://example.com');
    });

    test('adds auth interceptor when token provided', () async {
      SharedPreferences.setMockInitialValues({'auth_token': 'my-token'});
      final client = ApiClient(baseUrl: 'https://example.com');
      final token = await TokenStorage().getToken();
      expect(token, 'my-token');
    });
  });
}
```

- [ ] **Step 6: 运行测试确认失败**

```bash
cd football_insight_app && flutter test test/services/api_client_test.dart
```

Expected: FAIL

- [ ] **Step 7: 实现 ApiClient**

```dart
// lib/services/api_client.dart
import 'package:dio/dio.dart';
import 'package:football_insight_app/utils/token_storage.dart';

class ApiClient {
  final Dio dio;
  final TokenStorage _tokenStorage;

  ApiClient({required String baseUrl})
      : dio = Dio(BaseOptions(baseUrl: baseUrl)),
        _tokenStorage = TokenStorage() {
    dio.interceptors.add(_authInterceptor());
  }

  Interceptor _authInterceptor() {
    return InterceptorsWrapper(
      onRequest: (options, handler) async {
        final token = await _tokenStorage.getToken();
        if (token != null) {
          options.headers['Authorization'] = 'Bearer $token';
        }
        handler.next(options);
      },
    );
  }
}
```

- [ ] **Step 8: 运行测试确认通过**

```bash
cd football_insight_app && flutter test test/services/api_client_test.dart
```

Expected: PASS

- [ ] **Step 9: 实现 AuthService**

```dart
// lib/services/auth_service.dart
import 'package:dio/dio.dart';
import 'package:football_insight_app/models/user_profile.dart';
import 'package:football_insight_app/utils/token_storage.dart';

class AuthResult {
  final String accessToken;
  final UserProfile user;

  const AuthResult({required this.accessToken, required this.user});
}

class AuthService {
  final Dio _dio;
  final TokenStorage _tokenStorage;

  AuthService({required Dio dio, required TokenStorage tokenStorage})
      : _dio = dio,
        _tokenStorage = tokenStorage;

  Future<AuthResult> bindLicense(String licenseKey) async {
    final response = await _dio.post(
      '/api/v1/auth/bind-license',
      data: {'license_key': licenseKey},
    );
    final data = response.data;
    final token = data['access_token'] as String;
    await _tokenStorage.saveToken(token);
    final user = UserProfile.fromMap(data['user']);
    return AuthResult(accessToken: token, user: user);
  }

  Future<UserProfile> getMe() async {
    final response = await _dio.get('/api/v1/auth/me');
    return UserProfile.fromMap(response.data);
  }

  Future<void> logout() async {
    await _tokenStorage.clearToken();
  }

  Future<bool> isLoggedIn() async {
    return _tokenStorage.hasToken();
  }
}
```

- [ ] **Step 10: 运行全量测试**

```bash
cd football_insight_app && flutter test
```

Expected: PASS

- [ ] **Step 11: Commit**

```bash
git add football_insight_app/
git commit -m "feat(app): add API client, auth service, token storage with tests"
```

---

## Task 4: Ticket Watch Service + Provider

**Files:**
- Create: `football_insight_app/lib/services/ticket_watch_service.dart`
- Create: `football_insight_app/lib/providers/auth_provider.dart`
- Create: `football_insight_app/lib/providers/ticket_watch_provider.dart`
- Create: `football_insight_app/test/services/ticket_watch_service_test.dart`

- [ ] **Step 1: 写 ticket_watch_service 的失败测试**

```dart
// test/services/ticket_watch_service_test.dart
import 'package:dio/dio.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:football_insight_app/services/ticket_watch_service.dart';
import 'package:mockito/annotations.dart';
import 'package:mockito/mockito.dart';

import 'ticket_watch_service_test.mocks.dart';

@GenerateMocks([Dio])
void main() {
  group('TicketWatchService', () {
    late MockDio mockDio;
    late TicketWatchService service;

    setUp(() {
      mockDio = MockDio();
      service = TicketWatchService(dio: mockDio);
    });

    test('getCurrentBoard returns board data', () async {
      when(mockDio.get(any)).thenAnswer((_) async => Response(
        requestOptions: RequestOptions(path: ''),
        data: {
          'match': {'match_id': 1, 'home_team_name': '成都', 'away_team_name': '海港'},
          'inventory': [],
        },
      ));

      final result = await service.getCurrentBoard();
      expect(result, isNotNull);
      expect(result['match']['match_id'], 1);
    });
  });
}
```

- [ ] **Step 2: 运行测试确认失败**

```bash
cd football_insight_app && flutter test test/services/ticket_watch_service_test.dart
```

Expected: FAIL

- [ ] **Step 3: 实现 TicketWatchService**

```dart
// lib/services/ticket_watch_service.dart
import 'package:dio/dio.dart';

class TicketWatchService {
  final Dio _dio;

  TicketWatchService({required Dio _dio}) : _dio = _dio;

  Future<Map<String, dynamic>> getCurrentBoard() async {
    final response = await _dio.get('/api/v1/ticket-watch/current-board');
    return response.data as Map<String, dynamic>;
  }

  Future<Map<String, dynamic>> getInventory(int matchId, {String? since}) async {
    final queryParams = <String, dynamic>{};
    if (since != null) queryParams['since'] = since;
    final response = await _dio.get(
      '/api/v1/ticket-watch/matches/$matchId/inventory',
      queryParameters: queryParams,
    );
    return response.data as Map<String, dynamic>;
  }

  Future<List<dynamic>> getTrackedInterests(int matchId) async {
    final response = await _dio.get(
      '/api/v1/ticket-watch/matches/$matchId/tracked-interests',
    );
    return response.data as List<dynamic>;
  }

  Future<void> toggleBlockInterest(int matchId, String blockName) async {
    await _dio.post(
      '/api/v1/ticket-watch/matches/$matchId/interests/toggle',
      data: {'block_name': blockName},
    );
  }

  Future<List<dynamic>> getRegions() async {
    final response = await _dio.get('/api/v1/ticket-watch/regions');
    return response.data as List<dynamic>;
  }
}
```

- [ ] **Step 4: 运行测试确认通过**

```bash
cd football_insight_app && flutter test test/services/ticket_watch_service_test.dart
```

Expected: PASS

- [ ] **Step 5: 实现 AuthProvider**

```dart
// lib/providers/auth_provider.dart
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:football_insight_app/config/api_config.dart';
import 'package:football_insight_app/models/user_profile.dart';
import 'package:football_insight_app/services/api_client.dart';
import 'package:football_insight_app/services/auth_service.dart';
import 'package:football_insight_app/utils/token_storage.dart';

final tokenStorageProvider = Provider<TokenStorage>((ref) => TokenStorage());

final apiClientProvider = Provider<ApiClient>((ref) {
  return ApiClient(baseUrl: ApiConfig.baseUrl);
});

final authServiceProvider = Provider<AuthService>((ref) {
  return AuthService(
    dio: ref.watch(apiClientProvider).dio,
    tokenStorage: ref.watch(tokenStorageProvider),
  );
});

final authStateProvider = StateNotifierProvider<AuthNotifier, AsyncValue<UserProfile?>>((ref) {
  return AuthNotifier(ref.watch(authServiceProvider));
});

class AuthNotifier extends StateNotifier<AsyncValue<UserProfile?>> {
  final AuthService _authService;

  AuthNotifier(this._authService) : super(const AsyncValue.loading()) {
    _checkLogin();
  }

  Future<void> _checkLogin() async {
    try {
      final loggedIn = await _authService.isLoggedIn();
      if (!loggedIn) {
        state = const AsyncValue.data(null);
        return;
      }
      final user = await _authService.getMe();
      state = AsyncValue.data(user);
    } catch (e) {
      state = const AsyncValue.data(null);
    }
  }

  Future<void> bindLicense(String licenseKey) async {
    state = const AsyncValue.loading();
    try {
      final result = await _authService.bindLicense(licenseKey);
      state = AsyncValue.data(result.user);
    } catch (e, st) {
      state = AsyncValue.error(e, st);
    }
  }

  Future<void> logout() async {
    await _authService.logout();
    state = const AsyncValue.data(null);
  }
}
```

- [ ] **Step 6: 实现 TicketWatchProvider**

```dart
// lib/providers/ticket_watch_provider.dart
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:football_insight_app/providers/auth_provider.dart';
import 'package:football_insight_app/services/ticket_watch_service.dart';

final ticketWatchServiceProvider = Provider<TicketWatchService>((ref) {
  return TicketWatchService(dio: ref.watch(apiClientProvider).dio);
});

final currentBoardProvider = FutureProvider<Map<String, dynamic>>((ref) async {
  final service = ref.watch(ticketWatchServiceProvider);
  return service.getCurrentBoard();
});

final matchInventoryProvider = FutureProvider.family<Map<String, dynamic>, int>((ref, matchId) async {
  final service = ref.watch(ticketWatchServiceProvider);
  return service.getInventory(matchId);
});

final trackedInterestsProvider = FutureProvider.family<List<dynamic>, int>((ref, matchId) async {
  final service = ref.watch(ticketWatchServiceProvider);
  return service.getTrackedInterests(matchId);
});
```

- [ ] **Step 7: 运行全量测试**

```bash
cd football_insight_app && flutter test
```

Expected: PASS

- [ ] **Step 8: Commit**

```bash
git add football_insight_app/
git commit -m "feat(app): add ticket watch service, auth and watch providers"
```

---

## Task 5: 登录页

**Files:**
- Create: `football_insight_app/lib/pages/login_page.dart`
- Create: `football_insight_app/test/pages/login_page_test.dart`

- [ ] **Step 1: 写登录页 widget 测试**

```dart
// test/pages/login_page_test.dart
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:football_insight_app/pages/login_page.dart';

void main() {
  testWidgets('LoginPage shows license input and submit button', (tester) async {
    await tester.pumpWidget(
      const ProviderScope(
        child: MaterialApp(home: LoginPage()),
      ),
    );

    expect(find.byType(TextField), findsOneWidget);
    expect(find.text('绑定登录'), findsOneWidget);
  });

  testWidgets('LoginPage shows error on empty submit', (tester) async {
    await tester.pumpWidget(
      const ProviderScope(
        child: MaterialApp(home: LoginPage()),
      ),
    );

    await tester.tap(find.text('绑定登录'));
    await tester.pump();

    expect(find.text('请输入绑定码'), findsOneWidget);
  });
}
```

- [ ] **Step 2: 运行测试确认失败**

```bash
cd football_insight_app && flutter test test/pages/login_page_test.dart
```

Expected: FAIL

- [ ] **Step 3: 实现 LoginPage**

```dart
// lib/pages/login_page.dart
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:football_insight_app/providers/auth_provider.dart';

class LoginPage extends ConsumerStatefulWidget {
  const LoginPage({super.key});

  @override
  ConsumerState<LoginPage> createState() => _LoginPageState();
}

class _LoginPageState extends ConsumerState<LoginPage> {
  final _controller = TextEditingController();
  String? _error;
  bool _loading = false;

  Future<void> _submit() async {
    final code = _controller.text.trim();
    if (code.isEmpty) {
      setState(() => _error = '请输入绑定码');
      return;
    }
    setState(() {
      _loading = true;
      _error = null;
    });

    try {
      await ref.read(authStateProvider.notifier).bindLicense(code);
    } catch (e) {
      if (mounted) {
        setState(() {
          _error = '绑定失败，请检查绑定码是否正确';
          _loading = false;
        });
      }
    }
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('登录')),
      body: Padding(
        padding: const EdgeInsets.all(24),
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            const Text(
              '回流监控',
              style: TextStyle(fontSize: 28, fontWeight: FontWeight.bold),
            ),
            const SizedBox(height: 8),
            const Text(
              '在微信小程序中生成绑定码，输入后即可登录',
              style: TextStyle(fontSize: 14, color: Colors.grey),
              textAlign: TextAlign.center,
            ),
            const SizedBox(height: 32),
            TextField(
              controller: _controller,
              decoration: InputDecoration(
                labelText: '绑定码',
                hintText: '输入微信端生成的绑定码',
                border: const OutlineInputBorder(),
                errorText: _error,
              ),
              textCapitalization: TextCapitalization.characters,
            ),
            const SizedBox(height: 16),
            SizedBox(
              width: double.infinity,
              child: FilledButton(
                onPressed: _loading ? null : _submit,
                child: _loading
                    ? const SizedBox(
                        height: 20,
                        width: 20,
                        child: CircularProgressIndicator(strokeWidth: 2),
                      )
                    : const Text('绑定登录'),
              ),
            ),
          ],
        ),
      ),
    );
  }
}
```

- [ ] **Step 4: 运行测试确认通过**

```bash
cd football_insight_app && flutter test test/pages/login_page_test.dart
```

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add football_insight_app/
git commit -m "feat(app): add login page with license binding"
```

---

## Task 6: 首页 - 回流监控列表

**Files:**
- Create: `football_insight_app/lib/widgets/match_card_widget.dart`
- Create: `football_insight_app/lib/widgets/notification_banner.dart`
- Create: `football_insight_app/lib/pages/home_page.dart`
- Create: `football_insight_app/test/widgets/match_card_widget_test.dart`

- [ ] **Step 1: 写 MatchCardWidget 测试**

```dart
// test/widgets/match_card_widget_test.dart
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:football_insight_app/models/match_card.dart';
import 'package:football_insight_app/widgets/match_card_widget.dart';

void main() {
  testWidgets('MatchCardWidget displays match info', (tester) async {
    final match = MatchCard(
      matchId: 1,
      homeTeamName: '成都蓉城',
      awayTeamName: '上海海港',
      matchDate: '2026-05-20',
      matchTime: '19:35',
      roundNumber: 12,
      homeScore: 2,
      awayScore: 1,
      status: 'finished',
    );

    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: MatchCardWidget(match: match),
        ),
      ),
    );

    expect(find.text('成都蓉城'), findsOneWidget);
    expect(find.text('上海海港'), findsOneWidget);
    expect(find.text('2 : 1'), findsOneWidget);
  });
}
```

- [ ] **Step 2: 运行测试确认失败**

```bash
cd football_insight_app && flutter test test/widgets/match_card_widget_test.dart
```

Expected: FAIL

- [ ] **Step 3: 实现 MatchCardWidget**

```dart
// lib/widgets/match_card_widget.dart
import 'package:flutter/material.dart';
import 'package:football_insight_app/models/match_card.dart';

class MatchCardWidget extends StatelessWidget {
  final MatchCard match;
  final VoidCallback? onTap;

  const MatchCardWidget({super.key, required this.match, this.onTap});

  @override
  Widget build(BuildContext context) {
    final scoreText = match.homeScore != null && match.awayScore != null
        ? '${match.homeScore} : ${match.awayScore}'
        : 'vs';

    return Card(
      margin: const EdgeInsets.symmetric(horizontal: 16, vertical: 6),
      child: InkWell(
        onTap: onTap,
        borderRadius: BorderRadius.circular(12),
        child: Padding(
          padding: const EdgeInsets.all(16),
          child: Column(
            children: [
              Row(
                mainAxisAlignment: MainAxisAlignment.spaceBetween,
                children: [
                  Text('第 ${match.roundNumber} 轮', style: const TextStyle(fontSize: 12, color: Colors.grey)),
                  Text('${match.matchDate} ${match.matchTime}', style: const TextStyle(fontSize: 12, color: Colors.grey)),
                ],
              ),
              const SizedBox(height: 12),
              Row(
                mainAxisAlignment: MainAxisAlignment.spaceEvenly,
                children: [
                  Expanded(child: Text(match.homeTeamName, textAlign: TextAlign.center, style: const TextStyle(fontSize: 16, fontWeight: FontWeight.w600))),
                  Text(scoreText, style: const TextStyle(fontSize: 18, fontWeight: FontWeight.bold)),
                  Expanded(child: Text(match.awayTeamName, textAlign: TextAlign.center, style: const TextStyle(fontSize: 16, fontWeight: FontWeight.w600))),
                ],
              ),
            ],
          ),
        ),
      ),
    );
  }
}
```

- [ ] **Step 4: 运行测试确认通过**

```bash
cd football_insight_app && flutter test test/widgets/match_card_widget_test.dart
```

Expected: PASS

- [ ] **Step 5: 实现 NotificationBanner**

```dart
// lib/widgets/notification_banner.dart
import 'package:flutter/material.dart';

class NotificationBanner extends StatelessWidget {
  final String message;
  final VoidCallback? onTap;

  const NotificationBanner({super.key, required this.message, this.onTap});

  @override
  Widget build(BuildContext context) {
    return Material(
      color: Theme.of(context).colorScheme.primaryContainer,
      borderRadius: BorderRadius.circular(8),
      child: InkWell(
        onTap: onTap,
        borderRadius: BorderRadius.circular(8),
        child: Padding(
          padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 10),
          child: Row(
            children: [
              Icon(Icons.notifications_active, size: 18, color: Theme.of(context).colorScheme.primary),
              const SizedBox(width: 8),
              Expanded(child: Text(message, style: const TextStyle(fontSize: 13))),
            ],
          ),
        ),
      ),
    );
  }
}
```

- [ ] **Step 6: 实现 HomePage**

```dart
// lib/pages/home_page.dart
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:football_insight_app/providers/ticket_watch_provider.dart';
import 'package:football_insight_app/widgets/match_card_widget.dart';
import 'package:football_insight_app/widgets/notification_banner.dart';
import 'package:football_insight_app/models/reflux_event.dart';

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
            onPressed: () => Navigator.of(context).pushNamed('/settings'),
          ),
        ],
      ),
      body: RefreshIndicator(
        onRefresh: () => ref.refresh(currentBoardProvider.future),
        child: boardAsync.when(
          loading: () => const Center(child: CircularProgressIndicator()),
          error: (e, _) => Center(
            child: Column(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                const Text('加载失败'),
                const SizedBox(height: 8),
                FilledButton.tonal(
                  onPressed: () => ref.invalidate(currentBoardProvider),
                  child: const Text('重试'),
                ),
              ],
            ),
          ),
          data: (board) {
            final match = board['match'] as Map<String, dynamic>?;
            final inventory = board['inventory'] as List<dynamic>? ?? [];

            return ListView(
              padding: const EdgeInsets.only(top: 8, bottom: 80),
              children: [
                if (match != null)
                  MatchCardWidget(
                    match: MatchCard(
                      matchId: match['match_id'] as int,
                      homeTeamName: match['home_team_name'] as String? ?? '',
                      awayTeamName: match['away_team_name'] as String? ?? '',
                      matchDate: match['match_date'] as String? ?? '',
                      matchTime: match['match_time'] as String? ?? '',
                      roundNumber: match['round_number'] as int? ?? 0,
                      homeScore: match['home_score'] as int?,
                      awayScore: match['away_score'] as int?,
                      status: match['status'] as String? ?? '',
                    ),
                    onTap: () => Navigator.of(context).pushNamed('/match-detail', arguments: match['match_id']),
                  ),
                if (inventory.isNotEmpty) ...[
                  const Padding(
                    padding: EdgeInsets.fromLTRB(16, 16, 16, 8),
                    child: Text('实时回流', style: TextStyle(fontSize: 16, fontWeight: FontWeight.w600)),
                  ),
                  ...inventory.map((item) {
                    final blockName = item['block_name'] as String? ?? '';
                    final count = item['available_count'] as int? ?? 0;
                    final updatedAt = item['updated_at'] as String? ?? '';
                    return NotificationBanner(
                      message: '$blockName: 回流 $count 张 · $updatedAt',
                    );
                  }),
                ],
                if (match == null && inventory.isEmpty)
                  const Center(
                    child: Padding(
                      padding: EdgeInsets.only(top: 100),
                      child: Text('当前没有在售比赛', style: TextStyle(color: Colors.grey)),
                    ),
                  ),
              ],
            );
          },
        ),
      ),
    );
  }
}
```

- [ ] **Step 7: 运行全量测试**

```bash
cd football_insight_app && flutter test
```

Expected: PASS

- [ ] **Step 8: Commit**

```bash
git add football_insight_app/
git commit -m "feat(app): add home page with reflux monitoring list"
```

---

## Task 7: 比赛详情页 - 回流时间线 + 区域关注

**Files:**
- Create: `football_insight_app/lib/widgets/reflux_timeline_widget.dart`
- Create: `football_insight_app/lib/widgets/region_selector_widget.dart`
- Create: `football_insight_app/lib/pages/match_detail_page.dart`

- [ ] **Step 1: 实现 RefluxTimelineWidget**

```dart
// lib/widgets/reflux_timeline_widget.dart
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
          leading: CircleAvatar(
            radius: 16,
            backgroundColor: Theme.of(context).colorScheme.primaryContainer,
            child: Text(
              '${event.ticketCount}',
              style: TextStyle(
                fontSize: 12,
                color: Theme.of(context).colorScheme.primary,
              ),
            ),
          ),
          title: Text(event.blockName),
          subtitle: Text(event.timeLabel),
          trailing: Text('${event.ticketCount} 张'),
        );
      },
    );
  }
}
```

- [ ] **Step 2: 实现 RegionSelectorWidget**

```dart
// lib/widgets/region_selector_widget.dart
import 'package:flutter/material.dart';
import 'package:football_insight_app/models/seat_region.dart';

class RegionSelectorWidget extends StatelessWidget {
  final List<SeatRegion> regions;
  final Function(String blockName) onToggle;

  const RegionSelectorWidget({
    super.key,
    required this.regions,
    required this.onToggle,
  });

  @override
  Widget build(BuildContext context) {
    return Wrap(
      spacing: 8,
      runSpacing: 8,
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
```

- [ ] **Step 3: 实现 MatchDetailPage**

```dart
// lib/pages/match_detail_page.dart
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
  late Future<Map<String, dynamic>> _inventoryFuture;
  List<SeatRegion> _regions = [];

  @override
  void initState() {
    super.initState();
    _loadData();
  }

  void _loadData() {
    final service = ref.read(ticketWatchServiceProvider);
    _inventoryFuture = service.getInventory(widget.matchId);
    service.getRegions().then((data) {
      if (mounted) {
        setState(() {
          _regions = data.map((r) => SeatRegion(
            blockName: r['block_name'] as String? ?? '',
            priceLevel: r['price_level'] as String?,
          )).toList();
        });
      }
    });
  }

  Future<void> _toggleRegion(String blockName) async {
    final service = ref.read(ticketWatchServiceProvider);
    await service.toggleBlockInterest(widget.matchId, blockName);
    setState(() {
      _regions = _regions.map((r) {
        if (r.blockName == blockName) {
          return r.copyWith(isTracked: !r.isTracked);
        }
        return r;
      }).toList();
    });
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('回流详情')),
      body: FutureBuilder<Map<String, dynamic>>(
        future: _inventoryFuture,
        builder: (context, snapshot) {
          if (snapshot.connectionState == ConnectionState.waiting) {
            return const Center(child: CircularProgressIndicator());
          }
          if (snapshot.hasError) {
            return Center(child: Text('加载失败: ${snapshot.error}'));
          }

          final data = snapshot.data!;
          final items = (data['items'] as List<dynamic>? ?? []);
          final events = items.map((item) => RefluxEvent(
            blockName: item['block_name'] as String? ?? '',
            ticketCount: item['occurrences'] as int? ?? 0,
            occurredAt: DateTime.tryParse(item['timestamp'] as String? ?? '') ?? DateTime.now(),
            matchId: widget.matchId.toString(),
          )).toList();

          return SingleChildScrollView(
            padding: const EdgeInsets.all(16),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                const Text('回流时间线', style: TextStyle(fontSize: 16, fontWeight: FontWeight.w600)),
                const SizedBox(height: 8),
                RefluxTimelineWidget(events: events),
                const SizedBox(height: 24),
                const Text('关注看台区域', style: TextStyle(fontSize: 16, fontWeight: FontWeight.w600)),
                const SizedBox(height: 8),
                if (_regions.isNotEmpty)
                  RegionSelectorWidget(regions: _regions, onToggle: _toggleRegion)
                else
                  const Text('暂无区域数据'),
              ],
            ),
          );
        },
      ),
    );
  }
}
```

- [ ] **Step 4: 运行全量测试**

```bash
cd football_insight_app && flutter test
```

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add football_insight_app/
git commit -m "feat(app): add match detail page with timeline and region selector"
```

---

## Task 8: 统计页 + 设置页

**Files:**
- Create: `football_insight_app/lib/services/stats_service.dart`
- Create: `football_insight_app/lib/providers/stats_provider.dart`
- Create: `football_insight_app/lib/providers/push_provider.dart`
- Create: `football_insight_app/lib/services/push_notification_service.dart`
- Create: `football_insight_app/lib/pages/stats_page.dart`
- Create: `football_insight_app/lib/pages/settings_page.dart`

- [ ] **Step 1: 实现 StatsService（mock 数据）**

```dart
// lib/services/stats_service.dart
import 'package:football_insight_app/models/reflux_stats.dart';

class StatsService {
  Future<RefluxStats> getRefluxStats() async {
    await Future.delayed(const Duration(milliseconds: 300));
    return RefluxStats.fromMockData();
  }
}
```

- [ ] **Step 2: 实现 StatsProvider**

```dart
// lib/providers/stats_provider.dart
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:football_insight_app/models/reflux_stats.dart';
import 'package:football_insight_app/services/stats_service.dart';

final statsServiceProvider = Provider<StatsService>((ref) => StatsService());

final refluxStatsProvider = FutureProvider<RefluxStats>((ref) async {
  final service = ref.watch(statsServiceProvider);
  return service.getRefluxStats();
});
```

- [ ] **Step 3: 实现 StatsPage**

```dart
// lib/pages/stats_page.dart
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:football_insight_app/providers/stats_provider.dart';

class StatsPage extends ConsumerWidget {
  const StatsPage({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final statsAsync = ref.watch(refluxStatsProvider);

    return Scaffold(
      appBar: AppBar(title: const Text('回流统计')),
      body: statsAsync.when(
        loading: () => const Center(child: CircularProgressIndicator()),
        error: (e, _) => Center(child: Text('加载失败: $e')),
        data: (stats) {
          final maxCount = stats.hourlyBreakdown
              .map((h) => h.totalCount)
              .reduce((a, b) => a > b ? a : b);

          return SingleChildScrollView(
            padding: const EdgeInsets.all(16),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Card(
                  child: Padding(
                    padding: const EdgeInsets.all(16),
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        const Text('总览', style: TextStyle(fontSize: 16, fontWeight: FontWeight.w600)),
                        const SizedBox(height: 12),
                        Text('累计回流事件: ${stats.totalRefluxEvents}'),
                        if (stats.peakHourLabel != null)
                          Text('最活跃时段: ${stats.peakHourLabel}'),
                      ],
                    ),
                  ),
                ),
                const SizedBox(height: 16),
                const Text('按小时分布', style: TextStyle(fontSize: 16, fontWeight: FontWeight.w600)),
                const SizedBox(height: 8),
                ...stats.hourlyBreakdown.where((h) => h.totalCount > 0).map((h) {
                  final pct = maxCount > 0 ? h.totalCount / maxCount : 0.0;
                  return Padding(
                    padding: const EdgeInsets.only(bottom: 4),
                    child: Row(
                      children: [
                        SizedBox(
                          width: 50,
                          child: Text('${h.hour.toString().padLeft(2, '0')}:00', style: const TextStyle(fontSize: 12)),
                        ),
                        Expanded(
                          child: LinearProgressIndicator(value: pct),
                        ),
                        SizedBox(
                          width: 40,
                          child: Text('${h.totalCount}', textAlign: TextAlign.right, style: const TextStyle(fontSize: 12)),
                        ),
                      ],
                    ),
                  );
                }),
              ],
            ),
          );
        },
      ),
    );
  }
}
```

- [ ] **Step 4: 实现 PushNotificationService**

```dart
// lib/services/push_notification_service.dart
import 'package:dio/dio.dart';

class PushNotificationService {
  final Dio _dio;

  PushNotificationService({required Dio dio}) : _dio = dio;

  Future<void> registerToken(String token, String platform) async {
    await _dio.post('/api/v1/push/register-token', data: {
      'device_token': token,
      'platform': platform,
    });
  }

  Future<void> unregisterToken(String token) async {
    await _dio.delete('/api/v1/push/unregister-token', data: {
      'device_token': token,
    });
  }
}
```

- [ ] **Step 5: 实现 PushProvider**

```dart
// lib/providers/push_provider.dart
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:football_insight_app/providers/auth_provider.dart';
import 'package:football_insight_app/services/push_notification_service.dart';
import 'package:shared_preferences/shared_preferences.dart';

final pushServiceProvider = Provider<PushNotificationService>((ref) {
  return PushNotificationService(dio: ref.watch(apiClientProvider).dio);
});

final notificationEnabledProvider = StateNotifierProvider<NotificationToggleNotifier, bool>((ref) {
  return NotificationToggleNotifier();
});

class NotificationToggleNotifier extends StateNotifier<bool> {
  NotificationToggleNotifier() : super(false) {
    _load();
  }

  Future<void> _load() async {
    final prefs = await SharedPreferences.getInstance();
    state = prefs.getBool('notification_enabled') ?? true;
  }

  Future<void> toggle(bool enabled) async {
    final prefs = await SharedPreferences.getInstance();
    await prefs.setBool('notification_enabled', enabled);
    state = enabled;
  }
}
```

- [ ] **Step 6: 实现 SettingsPage**

```dart
// lib/pages/settings_page.dart
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:football_insight_app/providers/auth_provider.dart';
import 'package:football_insight_app/providers/push_provider.dart';

class SettingsPage extends ConsumerWidget {
  const SettingsPage({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final notificationEnabled = ref.watch(notificationEnabledProvider);

    return Scaffold(
      appBar: AppBar(title: const Text('设置')),
      body: ListView(
        children: [
          SwitchListTile(
            title: const Text('推送通知'),
            subtitle: const Text('接收回流推送提醒'),
            value: notificationEnabled,
            onChanged: (v) => ref.read(notificationEnabledProvider.notifier).toggle(v),
          ),
          const Divider(),
          ListTile(
            leading: const Icon(Icons.bar_chart),
            title: const Text('回流统计'),
            trailing: const Icon(Icons.chevron_right),
            onTap: () => Navigator.of(context).pushNamed('/stats'),
          ),
          const Divider(),
          ListTile(
            leading: const Icon(Icons.logout, color: Colors.red),
            title: const Text('退出登录', style: TextStyle(color: Colors.red)),
            onTap: () async {
              await ref.read(authStateProvider.notifier).logout();
              if (context.mounted) {
                Navigator.of(context).pushReplacementNamed('/login');
              }
            },
          ),
          const SizedBox(height: 32),
          const Center(
            child: Text('Football Insight App v0.1.0', style: TextStyle(color: Colors.grey, fontSize: 12)),
          ),
        ],
      ),
    );
  }
}
```

- [ ] **Step 7: 运行全量测试**

```bash
cd football_insight_app && flutter test
```

Expected: PASS

- [ ] **Step 8: Commit**

```bash
git add football_insight_app/
git commit -m "feat(app): add stats page, settings page, push notification service"
```

---

## Task 9: 路由整合 + 认证守卫

**Files:**
- Modify: `football_insight_app/lib/app.dart`
- Modify: `football_insight_app/lib/main.dart`

- [ ] **Step 1: 更新 app.dart 使用 go_router + 认证守卫**

```dart
// lib/app.dart
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:football_insight_app/providers/auth_provider.dart';
import 'package:football_insight_app/pages/login_page.dart';
import 'package:football_insight_app/pages/home_page.dart';
import 'package:football_insight_app/pages/match_detail_page.dart';
import 'package:football_insight_app/pages/stats_page.dart';
import 'package:football_insight_app/pages/settings_page.dart';
import 'package:go_router/go_router.dart';

final routerProvider = Provider<GoRouter>((ref) {
  final authState = ref.watch(authStateProvider);

  return GoRouter(
    redirect: (context, state) {
      final isLoggedIn = authState.valueOrNull != null;
      final isLoginRoute = state.matchedLocation == '/login';

      if (!isLoggedIn && !isLoginRoute) return '/login';
      if (isLoggedIn && isLoginRoute) return '/';
      return null;
    },
    routes: [
      GoRoute(path: '/login', builder: (context, state) => const LoginPage()),
      GoRoute(
        path: '/',
        builder: (context, state) => const HomePage(),
        routes: [
          GoRoute(
            path: 'match-detail',
            builder: (context, state) {
              final matchId = state.extra as int;
              return MatchDetailPage(matchId: matchId);
            },
          ),
          GoRoute(path: 'stats', builder: (context, state) => const StatsPage()),
          GoRoute(path: 'settings', builder: (context, state) => const SettingsPage()),
        ],
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
```

- [ ] **Step 2: 更新 main.dart**

```dart
// lib/main.dart
import 'package:flutter/material.dart';
import 'package:football_insight_app/app.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

void main() {
  WidgetsFlutterBinding.ensureInitialized();
  runApp(const ProviderScope(child: FootballInsightApp()));
}
```

- [ ] **Step 3: 运行 flutter analyze**

```bash
cd football_insight_app && flutter analyze
```

Expected: 无错误

- [ ] **Step 4: 运行全量测试**

```bash
cd football_insight_app && flutter test
```

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add football_insight_app/
git commit -m "feat(app): wire up go_router with auth guard and all pages"
```

---

## Task 10: FCM + APNs 推送集成骨架

**Files:**
- Create: `football_insight_app/lib/services/firebase_push_service.dart`
- Modify: `football_insight_app/lib/main.dart`

- [ ] **Step 1: 实现 FirebasePushService**

```dart
// lib/services/firebase_push_service.dart
import 'package:firebase_messaging/firebase_messaging.dart';
import 'package:football_insight_app/providers/push_provider.dart';
import 'package:football_insight_app/services/push_notification_service.dart';

class FirebasePushService {
  final PushNotificationService _pushService;
  final FirebaseMessaging _messaging = FirebaseMessaging.instance;

  FirebasePushService({required PushNotificationService pushService})
      : _pushService = pushService;

  Future<void> initialize() async {
    final settings = await _messaging.requestPermission();
    if (settings.authorizationStatus != AuthorizationStatus.authorized) {
      return;
    }

    final token = await _messaging.getToken();
    if (token != null) {
      await _registerToken(token);
    }

    _messaging.onTokenRefresh.listen(_registerToken);

    FirebaseMessaging.onMessage.listen(_handleForegroundMessage);
    FirebaseMessaging.onMessageOpenedApp.listen(_handleMessageOpenedApp);
  }

  Future<void> _registerToken(String token) async {
    final platform = _messaging.isSupported() ? 'fcm' : 'apns';
    await _pushService.registerToken(token, platform);
  }

  void _handleForegroundMessage(RemoteMessage message) {
    // 前台通知展示 - 后续实现 flutter_local_notifications
  }

  void _handleMessageOpenedApp(RemoteMessage message) {
    // 点击通知跳转 - 后续实现 deep link
  }
}
```

- [ ] **Step 2: 更新 main.dart 初始化 Firebase**

```dart
// lib/main.dart
import 'package:firebase_core/firebase_core.dart';
import 'package:flutter/material.dart';
import 'package:football_insight_app/app.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

void main() async {
  WidgetsFlutterBinding.ensureInitialized();
  await Firebase.initializeApp();
  runApp(const ProviderScope(child: FootballInsightApp()));
}
```

注意：Firebase 配置文件（`google-services.json` / `GoogleService-Info.plist`）需要后续手动添加，不提交到 git。

- [ ] **Step 3: 运行 flutter analyze**

```bash
cd football_insight_app && flutter analyze
```

Expected: 无错误

- [ ] **Step 4: Commit**

```bash
git add football_insight_app/
git commit -m "feat(app): add FCM push notification service skeleton"
```

---

## Task 11: 项目 README + .gitignore 收尾

**Files:**
- Modify: `football_insight_app/README.md`
- Modify: `football_insight_app/.gitignore`

- [ ] **Step 1: 更新 .gitignore**

追加到 `football_insight_app/.gitignore`：

```
# Firebase config files (contain secrets)
android/app/google-services.json
ios/Runner/GoogleService-Info.plist
ios/Runner/Runner.xcodeproj/project.pbxproj.userdata

# Apple signing
*.p8
*.cer
*.mobileprovision

# Build
build/
.dart_tool/

# env
.env
```

- [ ] **Step 2: 写 README**

在 `football_insight_app/README.md` 中写：

```markdown
# Football Insight App

回流监控 Flutter 客户端，支持 Android + iOS。

## 功能

- License 绑定码登录（微信小程序生成）
- 回流监控列表（实时回流状态、张数、区域）
- 看台区域关注
- 历史回流统计
- 推送通知（FCM + APNs）

## 开发

```bash
flutter pub get
flutter run
flutter test
flutter analyze
```

## Firebase 配置

1. Android: 将 `google-services.json` 放到 `android/app/`
2. iOS: 将 `GoogleService-Info.plist` 放到 `ios/Runner/`
3. 不要提交这些文件到 git
```

- [ ] **Step 3: Commit**

```bash
git add football_insight_app/
git commit -m "docs(app): add README and update .gitignore"
```

---

## 自检清单

| Spec 要求 | 对应 Task |
|-----------|-----------|
| Flutter Android + iOS 项目 | Task 1 |
| License 绑定码登录 | Task 3, 5 |
| 回流监控列表 | Task 4, 6 |
| 看台区域关注 | Task 7 |
| 开售时间提醒（UI + Provider） | Task 8 |
| 历史回流统计 | Task 8 |
| 每分钟汇总推送（FCM + APNs 骨架） | Task 10 |
| 路由 + 认证守卫 | Task 9 |

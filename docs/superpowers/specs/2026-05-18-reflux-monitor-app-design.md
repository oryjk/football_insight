# 回流监控 App 设计文档

## 背景

当前 Football Insight 已有完整的回流监控链路：ticket-monitor 服务检测回流 → Rust 后端代理数据 → 前端展示。已有邮件通知调度逻辑（每分钟汇总回流并发送邮件）。

现需开发 Android + iOS 原生客户端，核心功能是回流监控 + 原生推送通知。

## 技术选型

- **客户端框架：** Flutter
- **推送方式：** FCM（Android）+ APNs（iOS）
- **登录方式：** 微信端生成 license 绑定码，App 端输入登录
- **后端：** 复用现有 Rust 后端，新增 push consumer 和少量 API

## 新增子项目

```
football_insight/
├── football_insight_app/       ← Flutter 项目（新增）
├── football_insight_mini/      ← 微信小程序（已有）
├── football_insight_service_backend_rs/  ← Rust 后端（已有，需扩展）
└── sina_csl_scraper/           ← Python 抓取器（已有，不改）
```

## 整体架构

```
微信小程序（已有）
    ↓ 用户点击"生成绑定码"
Rust 后端 → 生成 license key → 写入 f_i_user_licenses
    ↓
Flutter App 输入 license → 后端验证 → 返回 JWT
    ↓ 同时上传 FCM/APNs device_token
Rust 后端存储 device_token

推送流（复用现有邮件通知调度）：
  ticket-monitor 检测回流（已有）
      ↓
  Rust 后端每分钟定时任务（已有邮件通知逻辑）
      ↓
  回流数据过滤/汇总（已有）
      ↓
  ├── 邮件消费者（已有）
  └── 推送消费者（新增）
       ↓ 读取订阅用户的 device_token
       ↓ 按 FCM / APNs 分别发送汇总推送
       Flutter App 收到推送 → 展示通知
```

## 数据库变更

### f_i_user_licenses（新表）

```sql
CREATE TABLE f_i_user_licenses (
    id          BIGSERIAL PRIMARY KEY,
    user_id     BIGINT NOT NULL REFERENCES f_i_users(id),
    license_key TEXT NOT NULL UNIQUE,
    used_at     TIMESTAMPTZ,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at  TIMESTAMPTZ NOT NULL
);
```

- `license_key`: 随机生成的绑定码，8-12 位字母数字
- `used_at`: NULL 表示未使用
- `expires_at`: 30 分钟有效期

### f_i_user_device_tokens（新表）

```sql
CREATE TABLE f_i_user_device_tokens (
    id           BIGSERIAL PRIMARY KEY,
    user_id      BIGINT NOT NULL REFERENCES f_i_users(id),
    device_token TEXT NOT NULL,
    platform     TEXT NOT NULL,  -- 'fcm' | 'apns'
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(device_token)
);
```

- 同一个 device_token 只绑定一个用户（用户换号时旧绑定自动失效）
- 用户可以有多台设备（多 token）

## 后端新增 API

### License 相关

| Method | Path | 说明 | 认证 |
|--------|------|------|------|
| POST | `/api/v1/auth/generate-license` | 微信用户生成绑定码 | JWT |
| POST | `/api/v1/auth/bind-license` | App 端输入绑定码登录，返回 JWT | 无 |

**generate-license 请求/响应：**
```json
// POST /api/v1/auth/generate-license
// Response:
{
  "license_key": "ABCD1234EFGH",
  "expires_at": "2026-05-18T12:30:00Z"
}
```

**bind-license 请求/响应：**
```json
// POST /api/v1/auth/bind-license
// Request:
{
  "license_key": "ABCD1234EFGH"
}
// Response:
{
  "access_token": "eyJ...",
  "user": { ... }
}
```

### Device Token

| Method | Path | 说明 | 认证 |
|--------|------|------|------|
| POST | `/api/v1/push/register-token` | App 上传/更新 device_token | JWT |
| DELETE | `/api/v1/push/unregister-token` | App 注销时移除 token | JWT |

**register-token 请求：**
```json
// POST /api/v1/push/register-token
// Request:
{
  "device_token": "xxxxxx",
  "platform": "fcm"
}
```

### 回流监控数据（复用已有）

| Method | Path | 说明 |
|--------|------|------|
| GET | `/api/v1/ticket-watch/current-board` | 当前比赛 + 库存 + 区域关注 |
| GET | `/api/v1/ticket-watch/matches/{id}/inventory` | 回流历史数据 |
| POST | `/api/v1/ticket-watch/matches/{id}/interests/toggle` | 关注/取关看台区域 |
| GET | `/api/v1/ticket-watch/matches/{id}/tracked-interests` | 我的关注区域 |

### 历史回流统计（新增）

| Method | Path | 说明 | 认证 |
|--------|------|------|------|
| GET | `/api/v1/ticket-watch/reflux-stats` | 历史回流统计 | 无或 JWT |

### 开售提醒（新增）

| Method | Path | 说明 | 认证 |
|--------|------|------|------|
| POST | `/api/v1/ticket-watch/sale-reminders` | 设置开售提醒 | JWT |
| GET | `/api/v1/ticket-watch/sale-reminders` | 获取我的提醒列表 | JWT |

## 推送消费者设计

复用现有邮件通知的调度逻辑，新增 push consumer 并行处理：

```
现有每分钟定时任务
    ↓
查询最近 1 分钟回流增量（已实现）
    ↓
按用户订阅过滤（已实现）
    ↓
生成汇总内容（已实现）
    ↓
分发到消费者：
    ├── EmailSender（已有）→ 发邮件
    └── PushSender（新增）→ 查 device_token → 调 FCM/APNs 发推送
```

### 推送 payload 格式

```json
{
  "title": "回流监控",
  "body": "XX vs XX 新增回流 3 张：A区1张、B区2张",
  "data": {
    "match_id": "123",
    "type": "reflux_alert"
  }
}
```

### 技术实现

- Android: FCM HTTP v1 API（Google 服务账号 JSON）
- iOS: APNs HTTP/2 直接发送（Apple .p8 证书）
- Rust 端用 `reqwest` 调 HTTP API，不需要额外 SDK

### 新增环境变量

- `FCM_SERVICE_ACCOUNT_JSON` — Google 服务账号凭据路径
- `APNS_PRIVATE_KEY_PATH` — Apple .p8 证书路径
- `APNS_TEAM_ID`
- `APNS_KEY_ID`
- `APNS_BUNDLE_ID`

## Flutter App 页面结构

```
App
├── 登录页
│   └── 输入 license 绑定码
├── 首页（回流监控）
│   ├── 当前比赛卡片（队名、时间、状态）
│   ├── 实时回流汇总（区域、张数、时间）
│   └── 下拉刷新
├── 详情页
│   ├── 回流时间线（按分钟展示回流事件）
│   ├── 看台区域关注开关
│   └── 区域余票柱状图
├── 统计页
│   ├── 历史回流热力图（哪个时段回流最多）
│   └── 按比赛/区域聚合统计
├── 设置页
│   ├── 通知开关
│   ├── 关注区域管理
│   ├── 开售提醒设置
│   └── 退出登录
└── 推送通知处理
    ├── 点击通知 → 跳转对应比赛详情
    └── 前台通知展示
```

## 核心功能清单

1. **License 登录** — 微信端生成绑定码 → App 输入 → JWT 认证
2. **回流监控列表** — 当前比塞回流状态、时间、张数、看台区域
3. **看台区域关注** — 用户关注特定区域，推送基于关注过滤
4. **开售时间提醒** — 抢票开始前推送提醒
5. **历史回流统计** — 哪个时间段回流最多，按区域/比赛聚合
6. **每分钟汇总推送** — 每分钟检查回流变化，汇总所有新增回流量推送

## 开发约定

- Flutter App 使用 `flutter` CLI + Dart
- 后端新增模块遵循现有 hexagonal 架构
- 使用 TDD 驱动开发
- 不提交密钥/证书到 git
- Flutter App 项目放在 monorepo 根目录 `football_insight_app/`

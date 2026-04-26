# Football Insight Deployment

这份文档描述当前 Football Insight 在生产环境的部署约定。

## 生产环境

- 服务器别名：`jd`
- 服务器登录：`ssh jd`
- 前端访问地址：`https://match.oryjk.cn/football/`
- 后端接口前缀：`https://match.oryjk.cn/api/v1/`

## 生产目录

- 后端项目目录：`/root/projects/football_insight_service_backend_rs`
- 后端日志文件：`/root/projects/football_insight_service_backend_rs/service.log`
- 后端 PID 文件：`/root/projects/football_insight_service_backend_rs/service.pid`
- 前端静态目录：`/root/docker_data/nginx/html/football/`

## Nginx 约定

- `jd` 上 Nginx 运行在 Docker 容器里
- 当前只允许修改 football 相关路径
- 如果发现路由冲突，不要直接修改其他服务配置，先确认
- 如果修改了 Nginx 配置：
  - 先做备份
  - 先跑 `nginx -t`
  - 不要 `reload`
  - 直接重启 Nginx Docker 容器

## 前端部署

### 本地构建

```bash
cd football_insight_mini
bun install
bun run build:h5
```

### 同步到生产

```bash
rsync -av --delete \
  football_insight_mini/dist/build/h5/ \
  jd:/root/docker_data/nginx/html/football/
```

### 前端上线后验证

```bash
curl -I https://match.oryjk.cn/football/
```

浏览器里至少检查：

- 首页是否能打开
- 榜单页是否能正常拉数据
- 我的页面是否能正常读取系统配置

## 后端部署

### 1. 推送代码

先保证本地代码已经提交并 push。

### 2. 服务器拉代码

```bash
ssh jd
cd /root/projects/football_insight_service_backend_rs
git pull
```

### 3. 执行数据库迁移

如果这次改动包含 migration：

```bash
cd /root/projects/football_insight_service_backend_rs
cargo run --bin run_migrations
```

如果需要临时执行某一条单独的 SQL 文件，也可以直接走项目内置入口：

```bash
cd /root/projects/football_insight_service_backend_rs
cargo run --bin run_migrations -- migrations/<your_migration>.sql
```

当前不再把“服务器全局安装 `sqlx`”作为默认前提，优先使用仓库内的 `run_migrations`。

### 4. 编译

```bash
cd /root/projects/football_insight_service_backend_rs
cargo build --release
```

如果机器资源紧张，编译会比较慢，这是当前 `jd` 的正常情况。

### 5. 重启后端

当前生产环境后端以二进制方式直接运行：

```bash
cd /root/projects/football_insight_service_backend_rs

if [ -f service.pid ]; then
  kill "$(cat service.pid)" || true
fi

nohup ./target/release/football_insight_service_backend_rs > service.log 2>&1 &
echo $! > service.pid
```

### 6. 后端上线后验证

```bash
curl https://match.oryjk.cn/api/v1/live/overview
curl https://match.oryjk.cn/api/v1/system/public-config
```

如果本次涉及认证，再额外检查：

- 注册
- 登录
- `/api/v1/auth/me`

如果本次涉及公众号 webhook，再额外检查：

- `GET /football/wechat/webhook`
- 关注事件或文本指令实际回调

## 日志查看

```bash
ssh jd 'tail -n 100 -f /root/projects/football_insight_service_backend_rs/service.log'
```

说明：

- 当前日志保留 ANSI 颜色
- 已配置 `logrotate`
- 生产日志会按大小轮转，不会无限增长

## 当前发布顺序建议

### 纯前端改动

1. 本地 `bun run build`
2. 同步 `dist/`
3. 浏览器验证页面

### 纯后端改动

1. push 代码
2. `jd` 上 `git pull`
3. 如有 migration 先执行
4. `cargo build --release`
5. 重启后端
6. `curl` 验证接口
7. 看 `service.log`

### 同时改前后端

1. 先发布后端
2. 验证接口正常
3. 再同步前端静态文件
4. 最后整站联调

### 小程序改动

小程序不通过 `jd` 上的 Nginx 发布，发布流程是：

1. 本地 `bun run build:mp-weixin`
2. 用微信开发者工具打开 `dist/build/mp-weixin`
3. 在微信开发者工具里上传新版本

## 风险点

- `jd` 上 Nginx 承载了其他服务，不要顺手改非 football 路由
- 后端部署前如果忘了 push，服务器拉不到最新代码
- 有 migration 时，如果只重启服务不跑迁移，接口可能变成 `500`
- 如果后端代码改了，本地验证前必须先重启本地后端，不要在旧进程上验证新逻辑

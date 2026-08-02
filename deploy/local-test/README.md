# Local Test Environment

该环境只运行在 local233，不连接生产数据库：

- API：`http://172.16.60.233:18092/`
- PostgreSQL：`127.0.0.1:55432/football_insight_test`
- Redis：`127.0.0.1:56379`
- 管理员用户名：`owner`
- 管理员密码：`FootballTest2026!`

启动：

```bash
./deploy/local-test/up.sh
```

停止：

```bash
./deploy/local-test/down.sh
```

PostgreSQL 数据保存在独立 Docker volume `football-insight-local-test-pgdata`。停止服务不会删除测试数据，也不会读取生产服务器上的 `.env`。

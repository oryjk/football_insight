# Local Admin Environment

该环境的后端和 Redis 运行在 local233，但通过 SSH 隧道直接连接生产 `football_data` 数据库：

- API：`http://172.16.60.233:18092/`
- PostgreSQL：`peiqian` 本机 PostgreSQL，经 local233 私有 SSH 隧道访问
- Redis：`127.0.0.1:56379`
- 管理员用户名：`admin`
- 管理员密码：`admin123`

> 警告：该环境读取的是真实生产用户。会员调整、禁用/恢复和其他写操作会直接修改生产数据库。

启动：

```bash
./deploy/local-test/up.sh
```

停止：

```bash
./deploy/local-test/down.sh
```

启动脚本会从 `peiqian` 上正在运行的生产后端容器读取 `DATABASE_URL`，但不会打印或写入文件；随后建立仅 local233 本机和 Docker bridge 可访问的 SSH 隧道，执行 migration，再启动本地后端。停止脚本会关闭后端、Redis 和 SSH 隧道。

旧隔离测试库的数据卷 `football-insight-local-test-pgdata` 会保留，但当前环境不再挂载或读取它。

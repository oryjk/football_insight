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

从其他可 SSH 访问 local233 的开发机发布已提交并 push 的代码：

```bash
./deploy_local233.sh
```

脚本根据自身位置定位本地 monorepo，不依赖开发机上的绝对路径。它会要求本地和
local233 工作区均无未提交改动，确认当前分支 HEAD 已 push，然后在 local233 上执行
`git pull --ff-only`、`deploy/local-test/up.sh` 以及接口验证。可按需覆盖：

```bash
DEPLOY_HOST=local233 \
DEPLOY_REPO_DIR=/home/betalpha/projects/football_insight \
DEPLOY_BRANCH=main \
./deploy_local233.sh
```

也可以直接登录 local233，在部署工作区执行同一个命令：

```bash
cd /home/betalpha/projects/football_insight
./deploy_local233.sh
```

脚本检测到自身位于 `DEPLOY_REPO_DIR` 后会进入本机模式：允许当前分支落后于 origin，
直接拉取 `origin/<当前分支>` 并部署，不会再 SSH 回 local233。

停止：

```bash
./deploy/local-test/down.sh
```

启动脚本会从 `peiqian` 上正在运行的生产后端容器读取 `DATABASE_URL`，但不会打印或写入文件；随后建立仅 local233 本机和 Docker bridge 可访问的 SSH 隧道，执行 migration，再启动本地后端。停止脚本会关闭后端、Redis 和 SSH 隧道。

旧隔离测试库的数据卷 `football-insight-local-test-pgdata` 会保留，但当前环境不再挂载或读取它。

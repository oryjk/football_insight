# 球员头像小尺寸版本设计

日期：2026-10-20
状态：方向已在对话中确认（负责人："先做 2，然后做 1，可以交给 Kimi"）

## 1. 问题

提名弹层等处头像渲染很慢。排查（2026-10-20，生产只读）：
- `f_i_squad_players` 736 条带头像记录、619 个不重复地址；其中 356 条是 600×900 的 JPEG 标准照（每张 80–120KB），其余是 85×85 到 200×200 的 PNG（12–46KB）。界面上只显示成约 32px（64rpx）或更小的圆形头像。
- 头像直接从 `https://oryjk.cn:82/minio/football-insight/summary/players/<文件名>` 读取，单张首字节约 1–2 秒，响应没有 `Cache-Control`。
- 长列表一次渲染全部候选人（已由 `a770f52` 加 `lazy-load` 缓解）。

本设计解决"没有小尺寸版本"。服务器首字节慢与 Nginx 缓存头不在本设计内（需负责人确认后单独处理）。

## 2. 方案

**小图与原图同目录、按固定规则命名，不改数据库、不改后端接口。**

| 原图 | 小图 |
|---|---|
| `summary/players/<文件名>` | `summary/players/thumbs/96/<文件名>.jpg`、`summary/players/thumbs/240/<文件名>.jpg` |

例：`summary/players/475933.png` → `summary/players/thumbs/96/475933.png.jpg`；`summary/players/113121-team143-20260304.jpg` → `summary/players/thumbs/96/113121-team143-20260304.jpg.jpg`。保留原文件名再加 `.jpg`，保证不同扩展名的同名原图不会撞名。

**尺寸与用途**：96px 用于列表行、提名弹层、转会卡、球场圆点、共识卡片；240px 用于大号卡片与 canvas 海报（球员大头像）。

**生成规则（抓取器，Pillow）**：
- 先裁成正方形：高大于宽时（标准照 600×900）取**上方**的正方形区域（脸部在上方）；否则取居中的正方形。
- 等比缩放到目标尺寸（`LANCZOS`）；原图比目标小时不放大，直接用裁好的正方形。
- 透明背景铺白，转 RGB，JPEG 质量 82、`optimize=True`。
- 上传时 `ContentType=image/jpeg`，`CacheControl=public, max-age=31536000, immutable`（文件名本身带版本，可长期缓存）。
- 每次上传球员头像（`upload_player_avatar`）后，顺带生成并上传两个小图；小图生成失败只记警告，不影响原图上传与名单同步。

**补生成（一次性 + 可重复执行）**：新 CLI 命令 `sync-avatar-thumbs`：
- 从数据库读取所有不重复的球员头像地址（`f_i_squad_players.avatar_storage_url`），只处理落在 `<public_base>/<prefix>/players/` 下的地址。
- 已存在的小图跳过（`head_object`），支持 `--dry-run`（只统计、不下载不上传）与 `--limit`。
- 输出 JSON 统计：处理数、跳过数、生成数、失败数。单张失败继续下一张，最后退出码非 0 表示有失败。
- **执行补生成会写生产 MinIO，只能在 review 后由负责人确认执行**。

## 3. 小程序

- `src/utils/avatarThumb.ts`：`avatarThumbUrl(url, size: 96 | 240)`——只对路径中含 `/players/` 且最后一段是文件名的 MinIO 地址换算，其他地址原样返回；`null`/空返回空串。
- 新共享组件 `src/components/FiAvatar.vue`：属性 `src`、`size: 96 | 240`（选哪个小图）、`name`（首字占位）、`lazy?: boolean`。先显示小图；小图加载失败（`@error`）改用原图；原图也失败显示名字首字。外观（尺寸、圆角、边框）由使用方通过外层 view 控制，组件只负责"显示哪张图"。
- 替换以下位置的球员头像：`TransferNominateSheet`、`FiTransferCard`、`LineupPlayerSheet`、`LineupPlayerDot`、`InsightsConsensusCard`（均用 96）；转会海报与战术板海报绘制前取 240 小图（`transferPoster`/`lineupPoster` 的头像地址经 `avatarThumbUrl(…, 240)`，取不到时用原图兜底）。
- 长列表处保留 `lazy-load`（`scripts/lazy-avatars.test.ts` 继续守护；规则调整为检查 `FiAvatar` 带 `lazy`）。

## 4. 不做

- 不改数据库表、不改后端接口；不处理球队队徽、用户头像；不改 Nginx；不删除原图。

## 5. 测试

- **抓取器**：裁剪（竖图取上方、横图/方图取中间、小图不放大、透明铺白）、命名规则、上传参数（内容类型、缓存头）、上传原图后顺带生成小图且失败不影响原图、补生成命令（跳过已存在、`--dry-run` 不写、单张失败继续并以非 0 退出）。全部用假的存储与下载器，不连生产。
- **小程序**：`avatarThumbUrl` 换算与原样返回；`FiAvatar` 回退顺序（小图 → 原图 → 首字）由纯函数 `nextAvatarSource(stage)` 覆盖；长列表 `lazy` 审计；构建、类型、边界、组件注册。

# Mini Program Troubleshooting

## Tabbar 图标看起来很小

这次迁移里碰到过一个容易踩坑的问题：`tabbar` 图标文件本身不是尺寸不对，而是 **PNG 内容没有居中**，结果图形被贴到了画布左上角，最后在微信开发者工具里看起来就会特别小。

### 现象

- `pages.json` 已经正确配置了 `iconPath` / `selectedIconPath`
- 图片文件尺寸也是正确的透明 PNG
- 但在小程序里看起来依然非常小
- 单独打开 PNG，会发现图形偏在左上角，不在画布中间

### 根因

最初使用系统缩略图方式把 SVG 转成 PNG，这种方式会错误处理透明画布和图形位置，导致：

- 图标没有居中
- 实际有效图形区域远小于画布
- 在微信 `tabbar` 中显示时就像“图标特别小”

### 正确做法

不要再用系统缩略图导出。当前仓库已经改成：

1. 保留 SVG 源文件：
   - `/home/wangrui/projects/football_insight/football_insight_mini/src/static/tabbar`
2. 使用 Playwright 在固定画布中渲染 SVG
3. 直接截图导出透明 PNG

导出脚本：

- `/home/wangrui/projects/football_insight/football_insight_mini/scripts/generate-tabbar-pngs.mjs`

### 当前约定

- 输出格式：透明 PNG
- 输出尺寸：`81 x 81`
- 输出目录：
  - `/home/wangrui/projects/football_insight/football_insight_mini/src/static/tabbar-png`

### 重新生成命令

```bash
cd /home/wangrui/projects/football_insight/football_insight_mini
node ./scripts/generate-tabbar-pngs.mjs
```

如果开发监听正在运行，再回到微信开发者工具执行：

1. 清缓存
2. 重新编译
3. 或重新预览

## 控制台出现 `Error: timeout`

如果小程序控制台出现：

```text
Error: timeout
```

先不要直接判断成后端慢。当前项目里已经确认过几个核心接口响应很快，很多情况下是：

- 微信开发者工具网络抖动
- 本地调试环境偶发请求超时
- 未登录场景和首页并发请求叠加时，日志表现不够清楚

### 当前请求层处理

请求封装在：

- `/home/wangrui/projects/football_insight/football_insight_mini/src/utils/request.ts`

当前已经做了两点：

1. `GET` 请求超时时自动重试 1 次
2. 最终错误会带上具体 URL，例如：

```text
请求超时：/live/rankings
```

这样下次就能直接知道是哪一个接口超时，而不是只看到裸 `timeout`。

### 排查顺序

1. 先看具体超时的是哪个 URL
2. 用 `curl` 测真实响应时间
3. 如果接口很快，优先怀疑开发者工具环境而不是后端
4. 如果某个接口持续超过 20 秒，再回到后端排查

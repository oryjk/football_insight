# 成都蓉城高清队徽

- 旧队徽仅 85 × 85，在首页被放大到 560rpx 水印，产生明显模糊。
- 新队徽为同款 800 × 800 透明 PNG，19,845 字节，未经放大或重绘。
- 来源：[FCLOGO · 成都蓉城 2021 全色队徽](https://fclogo.top/zh-cn/cfa/club/chengdu-rongcheng-v2021)。队徽标识权利归俱乐部所有。
- 原文件：`https://cdn.sanity.io/images/11hmdf08/production/f019fe4e842503295367fb5ba2b20b514b3fd5a6-800x800.png`。
- 复用抓取器 `assets.MinioAssetTarget`，通过被 Git 忽略的 `.env.sync` 读取现有配置，上传到 `summary/teams/77680-hd-09aae1f314de.png`；未替换旧对象，避免旧缓存与自动同步覆盖。
- 公网地址：`https://oryjk.cn:82/minio/football-insight/summary/teams/77680-hd-09aae1f314de.png`。
- 上传后匿名回读成功，Content-Type 为 `image/png`，SHA-256 为 `09aae1f314de1239293a4f89948769747df712a10f41c667edc09754509a7226`，内容逐字节一致。
- 小程序共用资源配置指向 MinIO；首页成都蓉城头像、主队背景水印和今日话题水印使用新地址，其他球队继续用 API 的头像地址。原位置、大小和透明度保持。
- 未修改生产数据库或后端，不把图片打入小程序包。

验证：`bun run type-check`、H5 与微信生产构建、依赖边界、微信组件注册和生产 API 校验通过；微信产物的队徽常量为上述 MinIO URL，头像和水印共用该地址。未上传体验版，微信显示效果待复核。

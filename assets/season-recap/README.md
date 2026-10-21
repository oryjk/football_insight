# 赛季海报背景 v2

`poster-dark-v2.jpg`、`poster-light-v2.jpg` 是内置 imagegen 根据用户确认的深浅海报生成的生产背景，使用 JPEG 编码压缩。图片均保留在仓库，运行时使用同名 MinIO 对象：

- `https://oryjk.cn:82/minio/football-insight/static/season-recap/poster-dark-v2.jpg`
- `https://oryjk.cn:82/minio/football-insight/static/season-recap/poster-light-v2.jpg`

生成提示要点：保留批准的构图与「我的赛季」毛笔字形；球迷背穿12号红色球衣举「成都蓉城」围巾，看台巨幅12号TIFO与「致敬第12人」横幅呼应。深色用红色夜场、金色标题和深酒红底；浅色用日间球场、朱红标题和暖白纸感。删除顶部队徽/赛季及全部下方数据、卡片、进度、客场队徽和页脚，下半部分留空，供真实数据动态绘制。不要把示例数字或用户称号烘焙进背景。

原始生成尺寸为942×1672附近的竖幅，实际尺寸以JPEG元数据为准。代码在固定Canvas中绘制，背景作为装饰铺满；页脚小程序码由接口获取。真实球队队徽使用项目既有素材，不依赖生成图像还原。

使用内置 image_gen 工具，未使用CLI/API回退。更新素材时增加版本路径，不覆盖已发布对象；上传使用 scraper 主机现有 MinIO 配置，不导出或提交凭据。

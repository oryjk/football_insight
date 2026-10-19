# 微信开发工具图片错误与 WXSS 警告修复

日期：2026-10-19；基线 `86bd988`。仅小程序相关改动。

## 根因与处理

1. 截图中的 `/static/user/member-card-dots.png` 图片加载错误：当前源码无此引用，图片已于 `1f5c556` 删除；旧开发包 `dist/dev/mp-weixin/common/assets.js` 仍导出指向该图片的 pulseMatchCornerImage/memberCardDotsImage。当前 HomeStoryPanel 实际导入的是 `static/home/pulse-match-corner.png`。完整重新运行 `bun run dev:mp-weixin` 后，公共资源模块更新、旧引用消失，无需恢复废弃图片。
2. 截图中的组件 WXSS 警告：组件内仍使用标签或属性选择器。首页球队赛季弹层、弹层壳（当前为 HomeSheetShell）、赛果面板、主队面板、话题语音按钮、洞察锁定预览共 6 个组件，改为给对应节点明确的 class。主队面板的无用错误样式删除。
3. 语音按钮保留原生 disabled，增加由 `!ready` 驱动的状态 class；播放、暂停和等待图形使用明确的子节点 class。接口和事件不变。

## 验证

- `bun run type-check`、`bun run check:boundaries` 通过。
- 首页、洞察和页面分享相关测试：164 项通过、0 失败（13 文件）。
- 真实 FiTopicAudioEntry SFC 编译渲染核对：hero 未就绪/就绪/播放/加载，以及普通按钮未就绪/就绪共 6 种状态，disabled 与状态 class 同步，等待点和暂停条数量正确。
- 开发小程序完整编译、`bun run build:h5`、`MINI_REVIEW_SKIP=1 bun run build:mp-weixin` 通过；生产 API 与组件注册校验通过。
- 用 PostCSS 和 selector parser 检查开发包、生产包上述 6 个组件的 WXSS：不含标签、ID、属性选择器（排除 keyframes 的帧选择器）。
- 扫描开发包、生产包的 JS/WXSS/WXML：旧图片引用均为零；各 56 处静态图片引用均有对应打包文件，资源模块指向新图。
- 320/390/430 浏览器 mock 回归覆盖游客、普通用户和权益暂停，默认数据洞察、统一容器、背景渐隐、切队、排序、点赞与无横向溢出通过，无页面运行错误；不写生产。

## 使用新产物

微信开发工具应打开新生成的 `dist/dev/mp-weixin`。在其他电脑使用源码同步时，需获取此轮提交并重启 `bun run dev:mp-weixin`；`bun run dev:mp-weixin:sync` 检测到小程序源码更新后会自行重启编译器。若仍提示旧资源路径或旧组件名，清除微信开发工具的编译缓存后重新编译。

本次未操作用户电脑上的微信开发工具，因此仍需在该工具端复核控制台；截图顶部的微信 getSystemInfo 平台提示不属于本次资源错误或组件 WXSS 修复范围。未上传体验版、未提审；本地与生产后端均未部署/重启。

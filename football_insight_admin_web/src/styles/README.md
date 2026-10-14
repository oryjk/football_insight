# 样式归属

`src/styles.css` 是唯一入口，导入顺序即全局 cascade。

| 文件           | 归属                                                                         |
| -------------- | ---------------------------------------------------------------------------- |
| tokens.css     | Tailwind 语义主题别名                                                        |
| foundation.css | 深浅主题 Token、reset、共享 shadcn 控件/data-slot 样式                       |
| login.css      | 管理员登录                                                                   |
| shell.css      | Sidebar / Header / Main 布局和抽屉                                           |
| workspace.css  | Main Content Card、分区、表单、反馈                                          |
| products.css   | 商品筛选、表格、分页、详情、编辑、购买账号选择器、购买记录弹窗和页面响应规则 |
| settings.css   | 当前管理员账号/密码设置                                                      |
| responsive.css | 跨页面壳层 breakpoints                                                       |

业务类仅组织布局；控件使用 `components/ui`，按钮外观归 foundation。
所有颜色、字号、行高、间距、圆角引用 Token。36px 用于工具栏/表单，48px 用于登录。
页面代码不堆 Tailwind；不要再在 Main Card 外添加业务 Surface。

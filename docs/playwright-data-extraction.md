# Playwright 抓数笔记

这份文档记录一个核心原则：

- 能抓接口，就不要先截图再 OCR
- 能读 DOM，就不要先截图再 OCR
- 只有接口拿不到、DOM 也不稳定时，才把截图识别当兜底方案

Playwright 的优势不在“截图”，而在：

- 打开真实浏览器
- 触发真实交互
- 监听网络请求
- 读取页面 DOM

对这个项目后续做数据探索、竞品页面分析、低频人工抓取很有用。

## 运行前提

下面所有例子都默认在前端项目目录执行：

```bash
cd football_insight_mini
```

当前示例直接复用项目里的 Playwright 依赖：

```bash
./node_modules/.bin/playwright --version
```

如果只是跑 Node 脚本，也可以直接：

```bash
node -e "console.log(require('./node_modules/playwright').chromium !== undefined)"
```

## 例子 1：抓页面实际请求到的接口

这个例子适合：

- 想知道一个页面到底调用了哪些 JSON 接口
- 想确认是哪个接口返回了榜单、赛程、详情数据
- 想把页面分析从“看 HTML”升级到“抓真实数据源”

```bash
cd football_insight_mini

node <<'EOF'
const { chromium } = require('./node_modules/playwright');

(async () => {
  const browser = await chromium.launch({ headless: true });
  const page = await browser.newPage({
    viewport: { width: 430, height: 932 },
  });

  const apiResponses = [];

  page.on('response', async (response) => {
    const url = response.url();
    if (!url.includes('/api/') && !url.includes('/ajax/') && !url.includes('/json')) {
      return;
    }

    const contentType = response.headers()['content-type'] || '';
    if (!contentType.includes('application/json')) {
      return;
    }

    try {
      const body = await response.json();
      apiResponses.push({
        url,
        status: response.status(),
        keys: Object.keys(body).slice(0, 12),
      });
    } catch {
      apiResponses.push({
        url,
        status: response.status(),
        keys: ['<json parse failed>'],
      });
    }
  });

  await page.goto('https://match.oryjk.cn/football/', {
    waitUntil: 'networkidle',
    timeout: 20000,
  });

  console.log(JSON.stringify(apiResponses, null, 2));
  await browser.close();
})();
EOF
```

你会拿到类似这种结果：

```json
[
  {
    "url": "https://match.oryjk.cn/api/v1/live/overview",
    "status": 200,
    "keys": ["view_kind", "round_number", "current_season", "latest_scrape_finished_at"]
  }
]
```

优先这样做的原因：

- 接口一旦找到，后续抓数成本最低
- 返回是结构化 JSON，不需要 OCR 清洗
- 页面样式怎么变，接口通常都更稳定

## 例子 2：直接读取页面 DOM 表格或卡片内容

这个例子适合：

- 页面已经把数据渲染出来了
- 接口不方便直接复用
- 想快速把页面上的榜单、赛程、标题、比分抽出来

下面示例读取首页三张摘要卡：

```bash
cd football_insight_mini

node <<'EOF'
const { chromium } = require('./node_modules/playwright');

(async () => {
  const browser = await chromium.launch({ headless: true });
  const page = await browser.newPage({
    viewport: { width: 430, height: 932 },
  });

  await page.goto('https://match.oryjk.cn/football/', {
    waitUntil: 'networkidle',
    timeout: 20000,
  });

  const cards = await page.$$eval('.briefing-card', (elements) =>
    elements.map((el) => {
      const label = el.querySelector('span')?.textContent?.trim() || '';
      const title = el.querySelector('.briefing-card__title-block strong')?.textContent?.trim() || '';
      const subValue = el.querySelector('.briefing-card__subvalue')?.textContent?.trim() || '';
      const metricValue = el.querySelector('.briefing-card__metric-value')?.textContent?.trim() || '';
      const metricLabel = el.querySelector('.briefing-card__metric-label')?.textContent?.trim() || '';

      return { label, title, subValue, metricValue, metricLabel };
    }),
  );

  console.log(JSON.stringify(cards, null, 2));
  await browser.close();
})();
EOF
```

如果目标页面是榜单表格，也可以把选择器换成 `table tr`、`.ranking-row`、`.match-card` 这类更贴合页面结构的选择器。

优先这样做的原因：

- 速度快
- 可读性高
- 调试简单
- 不需要处理图片识别误差

## 例子 3：先点击 / 切换，再继续抓数据

这个例子适合：

- 页面初始内容不完整
- 必须点 tab、按钮、筛选项才能看到目标数据
- 想抓不同状态下的内容

下面示例先打开首页，再切到赛程页读取比赛卡：

```bash
cd football_insight_mini

node <<'EOF'
const { chromium } = require('./node_modules/playwright');

(async () => {
  const browser = await chromium.launch({ headless: true });
  const page = await browser.newPage({
    viewport: { width: 430, height: 932 },
  });

  await page.goto('https://match.oryjk.cn/football/', {
    waitUntil: 'networkidle',
    timeout: 20000,
  });

  await page.getByRole('link', { name: '赛程' }).click();
  await page.waitForURL('**/matches', { timeout: 10000 });
  await page.waitForLoadState('networkidle');

  const matches = await page.$$eval('.match-card', (elements) =>
    elements.slice(0, 5).map((el) => {
      const round = el.querySelector('.match-card__round')?.textContent?.trim() || '';
      const date = el.querySelector('.match-card__date')?.textContent?.trim() || '';
      const home = el.querySelector('.match-card__home strong')?.textContent?.trim() || '';
      const away = el.querySelector('.match-card__away strong')?.textContent?.trim() || '';
      const score = el.querySelector('.match-card__score')?.textContent?.trim() || '';

      return { round, date, home, away, score };
    }),
  );

  console.log(JSON.stringify(matches, null, 2));
  await browser.close();
})();
EOF
```

这个模式很适合后续抓：

- 切不同 tab 的榜单
- 打开弹窗后的完整数据
- 展开更多后的列表
- 登录后可见的会员或用户页数据

## 什么时候再考虑截图 + OCR

只有下面这些情况，才建议上截图识别：

- 数据绘制在 canvas 里
- 页面是图片，不是 DOM
- 接口拿不到
- DOM 结构极不稳定
- 只是低频抓取，能用就行

如果真要走 OCR，建议流程是：

1. 先用 Playwright 稳定截图
2. 保持固定视口和缩放
3. 尽量只截局部区域，不要整页
4. OCR 后再做结构化清洗

但这条路应当是兜底，不应是默认方案。

## 建议的抓数决策顺序

以后遇到任何“想把网页里的数据拿下来”的需求，建议按这个顺序判断：

1. 能不能直接找到接口
2. 找不到接口时，能不能直接读 DOM
3. 需要交互时，能不能先点开再读 DOM
4. 以上都不行，再考虑截图 + OCR

一句话总结：

**Playwright 最有价值的地方，不是截图，而是让你用“真实浏览器”的方式拿到结构化数据。**

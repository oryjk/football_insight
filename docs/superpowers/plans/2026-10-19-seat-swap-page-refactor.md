# 换座页重构（一期）实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 换座页按项目规范完成 token 化、层级统一与分层隔离，地图改为"首屏展示 + 划出视口后浮窗唤起弹层"，交互带动画，所有换座业务行为不变。

**Architecture:** 页面 `index.vue` 只做装配（模板、滚动锁、IntersectionObserver、生命周期、分享）；数据加载放 `useSeatSwapPage.ts`，浏览/筛选/浮窗状态放 `useSeatSwapBrowse.ts`，操作沿用 `useSeatSwapActions.ts`（不改）。展示组件纯 props/emit。纯函数集中在 `helpers.ts`，开发 mock 迁到 `fixtures.ts`。新增一个样式审计测试，把"无渐变、无散落 hex、无数字字重"变成机器检查。

**Tech Stack:** uni-app + Vue 3 `<script setup>` + TypeScript，Bun（`bun test`），样式 token 在 `src/styles/fi-tokens.css`。

**Spec:** `docs/superpowers/specs/2026-10-19-seat-swap-page-refactor-design.md`（执行者必须同时阅读 spec 与本计划）

所有路径相对 `football_insight_mini/`，命令都在该目录执行。

## Global Constraints

- 只改 `football_insight_mini`；不改后端、接口、`football_insight_h5`；不读取、搜索、修改 `football_insight_admin_android`。
- `useSeatSwapActions.ts`、`components/useSeatSwapPublishForm.ts` 一行不改。
- 颜色、间距、字号、行高、圆角、边框、阴影、字重、动效时长与曲线全部引用 token；`src/pages/seat-swap` 下除 `/* decorative:start */ … /* decorative:end */` 包裹的票价分区配色外无 hex；无 `gradient`；无数字 `font-weight`。
- 组件层不得 import `api/**`、store、session；页面之间不得互相 import；不得往 `scripts/check-import-boundaries.mjs` 的 ALLOWLIST 加例外。
- 运行时 Vue 组件直接从 `.vue` 文件导入，不走 barrel。
- 动画只动 `transform` 与 `opacity`。
- 不手改 `manifest.json` 的 `versionName`；**不上传小程序**。
- 提交信息 `feat(mini): ...` / `refactor(mini): ...` / `test(mini): ...` / `docs: ...`，subject 英文，末尾加一行 `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`；直接提交到 `main`，每个任务提交前 `git pull --rebase`，提交后 `git push`。工作区里不属于你的未提交改动不要碰。
- 每个任务结束都要跑：`bun test`、`bun run type-check`、`bun run check:boundaries`。涉及新增或改名组件的任务还要跑 `bun run verify:mp-components`。

## Review Focus

1. **地图节点晚于 `onReady` 出现**：加载中只渲染骨架屏，`#seat-swap-map-anchor` 不存在；observer 必须在每次加载成功后重新挂载（先 `disconnect` 旧的），否则浮窗永远不出现，或在 `onShow` 重载后失效。→ Task 7 Step 6 的挂载时机与 Task 3 的 `mapInViewport` 默认值 `true`。
2. **任一弹层打开时浮窗不该出现**（地图、发布、管理弹层）：浮窗在弹层遮罩之上会被误点。→ Task 3 `shouldShowSeatSwapMapFab` 的 `sheetOpen` 包括三个弹层，Task 4 测试覆盖。
3. **弹层关闭动画期间连点分区**：第二次点击不能再触发一次"关闭 + 滚动"。→ Task 7 `mapTapBusy` 守卫。
4. **点浮窗上的 ✕ 只清筛选，不能同时打开弹层**：✕ 的 tap 必须 `.stop`。→ Task 6 组件实现。
5. **`onShow` 重载后筛选分区已无意向**：浮窗文案退回分区 key 或保持名称，列表显示"该分区暂无换座意向"，不报错。→ Task 4 测试 `filterName` 回退。

---

## 文件结构

| 文件 | 动作 | 职责 |
|---|---|---|
| `src/styles/fi-tokens.css` | 改 | 新增 seat 语义色、motion、换座 component token |
| `src/static/seat-swap/*.svg` | 新增 | Lucide 图标 |
| `src/composables/useNavigationLayout.ts` | 改 | 额外暴露 `navigationHeightPx` |
| `src/pages/seat-swap/fixtures.ts` | 新增 | 开发 mock（从 helpers 迁出） |
| `src/pages/seat-swap/helpers.ts` | 改 | 删死代码/空包装，新增 4 个纯函数 |
| `src/pages/seat-swap/helpers.test.ts` | 改 | 同步删除与新增用例 |
| `src/pages/seat-swap/styleAudit.test.ts` | 新增 | 样式规范机器检查 |
| `src/pages/seat-swap/useSeatSwapPage.ts` | 新增 | 数据加载 |
| `src/pages/seat-swap/useSeatSwapBrowse.ts` | 新增 | 浏览、筛选、浮窗状态 |
| `src/pages/seat-swap/useSeatSwapComposables.test.ts` | 新增 | 两个 composable 的测试 |
| `src/pages/seat-swap/index.vue` | 改 | 只做装配 |
| `src/pages/seat-swap/components/SeatSwapMatchCard.vue` | 新增（替代 Hero） | 比赛卡 |
| `src/pages/seat-swap/components/SeatSwapHero.vue` | 删除 | — |
| `src/pages/seat-swap/components/SeatSwapNotice.vue` | 新增 | 提示条 |
| `src/pages/seat-swap/components/SeatSwapSkeleton.vue` | 新增 | 骨架屏 |
| `src/pages/seat-swap/components/SeatSwapMapFab.vue` | 新增 | 浮窗按钮 |
| `src/pages/seat-swap/components/SeatSwapMapSheet.vue` | 新增 | 地图弹层 |
| `src/pages/seat-swap/components/SeatSwapRegionPanel.vue` | 改 | token 化 + `embedded` |
| `src/pages/seat-swap/components/SeatSwapPoolList.vue` | 改 | FiPanel/FiSectionHeading + 动画 |
| `src/pages/seat-swap/components/SeatSwapCandidateCard.vue` | 改 | token 化 + 状态色简化 |
| `src/pages/seat-swap/components/SeatSwapDock.vue` | 改 | token 化 |
| `src/pages/seat-swap/components/StadiumMap.vue` | 改 | token 化 + decorative 标注 |
| `src/pages/seat-swap/components/SeatSwapPublishSheet.vue` | 改 | 只改样式 |
| `src/pages/seat-swap/components/SeatSwapManageSheet.vue` | 改 | 只改样式 |
| `docs/mini-design-system.md`、`../CHANGELOG.md`、`../FEATURES.md` | 改 | 文档 |

---

### Task 1: 样式审计测试、token 与图标（地基）

**Files:**
- Create: `src/pages/seat-swap/styleAudit.test.ts`
- Modify: `src/styles/fi-tokens.css`
- Create: `src/static/seat-swap/arrow-left-right.svg`、`info.svg`、`map.svg`、`chevron-down.svg`
- Modify: `src/composables/useNavigationLayout.ts`

**Interfaces:**
- Produces: CSS 变量 `--fi-color-seat-current`、`--fi-color-seat-current-bg`、`--fi-color-seat-desired`、`--fi-color-seat-desired-bg`、`--fi-motion-duration-fast|base|slow`、`--fi-motion-ease-out|in|spring`、`--fi-component-seat-swap-dock-height`、`--fi-component-seat-swap-action-width`、`--fi-component-seat-swap-action-min-height`；图标路径 `/static/seat-swap/<name>.svg`；`useNavigationLayout()` 返回值新增 `navigationHeightPx: ComputedRef<number>`。
- Produces: `styleAudit.test.ts` 导出无，测试名 `seat swap style audit`。本任务里它以 `test.todo` 形式存在的断言会在 Task 9 改为真正断言（见 Step 1 说明）。

- [ ] **Step 1: 写样式审计测试（先只对新组件生效）**

审计逻辑：扫描 `src/pages/seat-swap` 下所有 `.vue` 的 `<style>` 段，剔除 `/* decorative:start */` 到 `/* decorative:end */` 之间的内容后，断言不出现 `gradient`、hex 颜色、数字字重。由于旧组件要到后续任务才改完，先用 `AUDITED_FILES` 白名单逐步纳入，Task 9 改为全目录。

```ts
// src/pages/seat-swap/styleAudit.test.ts
import { describe, expect, test } from 'bun:test'
import { readdirSync, readFileSync } from 'node:fs'
import { join } from 'node:path'

const ROOT = join(import.meta.dir)

// 逐任务纳入；Task 9 改为 'ALL'。
const AUDITED_FILES: string[] | 'ALL' = []

function listVueFiles(dir: string): string[] {
  return readdirSync(dir, { withFileTypes: true }).flatMap((entry) => {
    const path = join(dir, entry.name)
    if (entry.isDirectory()) return listVueFiles(path)
    return entry.name.endsWith('.vue') ? [path] : []
  })
}

export function extractAuditableStyle(source: string): string {
  const styles = [...source.matchAll(/<style[^>]*>([\s\S]*?)<\/style>/g)].map((m) => m[1]).join('\n')
  return styles.replace(/\/\*\s*decorative:start[\s\S]*?decorative:end\s*\*\//g, '')
}

export function findStyleViolations(style: string): string[] {
  const violations: string[] = []
  style.split('\n').forEach((line, index) => {
    const code = line.replace(/\/\*.*?\*\//g, '')
    if (/gradient/i.test(code)) violations.push(`${index + 1}: gradient`)
    if (/#[0-9a-fA-F]{3,8}\b/.test(code)) violations.push(`${index + 1}: hex`)
    if (/font-weight:\s*\d/.test(code)) violations.push(`${index + 1}: numeric font-weight`)
  })
  return violations
}

describe('seat swap style audit helpers', () => {
  test('ignores decorative blocks and comments', () => {
    const style = extractAuditableStyle(
      '<style>\n/* decorative:start 票价分区配色 */\n.a { background: #336fbd; }\n/* decorative:end */\n.b { color: var(--fi-color-primary); } /* was #fff */\n</style>',
    )
    expect(findStyleViolations(style)).toEqual([])
  })

  test('flags gradients, hex colors and numeric font weights', () => {
    const style = '.a { background: linear-gradient(#fff, #000); }\n.b { font-weight: 400; }'
    expect(findStyleViolations(style)).toEqual(['1: gradient', '1: hex', '2: numeric font-weight'])
  })
})

describe('seat swap style audit', () => {
  const files = AUDITED_FILES === 'ALL'
    ? listVueFiles(ROOT)
    : AUDITED_FILES.map((name) => join(ROOT, name))

  for (const file of files) {
    test(`${file.replace(ROOT + '/', '')} follows token rules`, () => {
      const style = extractAuditableStyle(readFileSync(file, 'utf8'))
      expect(findStyleViolations(style)).toEqual([])
    })
  }
})
```

- [ ] **Step 2: 运行，确认辅助函数测试通过（审计列表为空）**

Run: `bun test src/pages/seat-swap/styleAudit.test.ts`
Expected: 2 pass，0 fail。

- [ ] **Step 3: 加 token**

先 `grep -nE "#167348|#eef8f2|#fff1f0" src/styles/fi-tokens.css` 确认：`#167348`、`#eef8f2` 只作为 ticket-watch 专用 primitive 存在（第 331–332 行），`#fff1f0` 不存在，因此新建通用 primitive。

在 `/* ===== Primitive：字重 ===== */` 段之前追加：

```css
  /* ===== Primitive：换座座位语义色（取自换座页原有值） ===== */
  --fi-primitive-green-deep: #167348;
  --fi-primitive-green-soft: #eef8f2;
  --fi-primitive-red-soft: #fff1f0;

  /* ===== Primitive：动效（取自 FiBottomSheet / FiSegmented 现值） ===== */
  --fi-primitive-motion-fast: 120ms;
  --fi-primitive-motion-base: 220ms;
  --fi-primitive-motion-slow: 280ms;
  --fi-primitive-ease-out: cubic-bezier(0.22, 1, 0.36, 1);
  --fi-primitive-ease-in: cubic-bezier(0.4, 0, 1, 1);
  --fi-primitive-ease-spring: cubic-bezier(0.3, 0.9, 0.3, 1);
```

在 semantic 颜色段（`--fi-color-success-accent` 所在行之后）追加：

```css
  --fi-color-seat-current: var(--fi-color-primary);
  --fi-color-seat-current-bg: var(--fi-primitive-red-soft);
  --fi-color-seat-desired: var(--fi-primitive-green-deep);
  --fi-color-seat-desired-bg: var(--fi-primitive-green-soft);
  --fi-motion-duration-fast: var(--fi-primitive-motion-fast);
  --fi-motion-duration-base: var(--fi-primitive-motion-base);
  --fi-motion-duration-slow: var(--fi-primitive-motion-slow);
  --fi-motion-ease-out: var(--fi-primitive-ease-out);
  --fi-motion-ease-in: var(--fi-primitive-ease-in);
  --fi-motion-ease-spring: var(--fi-primitive-ease-spring);
```

在 component 段（`--fi-component-loading-*` 附近）追加：

```css
  /* 换座页：Dock 与浮窗共用高度，确保浮窗定位在 Dock 之上 */
  --fi-component-seat-swap-dock-height: 96rpx;
  --fi-component-seat-swap-action-width: 168rpx;
  --fi-component-seat-swap-action-min-height: 152rpx;
```

- [ ] **Step 4: 加图标**

在 `src/static/seat-swap/` 新建 4 个 SVG。格式照抄 `src/static/ticket-watch/activity-active.svg`（文件头 Lucide 注释、24×24、`stroke-width="2"`、round 端点）。路径**必须**从 `https://raw.githubusercontent.com/lucide-icons/lucide/0.468.0/icons/<name>.svg` 取得并逐字核对；颜色：

| 文件 | stroke |
|---|---|
| `arrow-left-right.svg` | `#dc2626` |
| `info.svg` | `#7f8490` |
| `chevron-down.svg` | `#7f8490` |
| `map.svg` | `#ffffff` |

（SVG 是静态资源，不在样式审计范围内。）

- [ ] **Step 5: `useNavigationLayout` 暴露导航高度（像素）**

`src/composables/useNavigationLayout.ts` 第 34 行改为：

```ts
  return {
    navigationPageStyle: computed(() => layout.value ? formatPageStyle(layout.value) : ''),
    /** 品牌导航总高度（CSS px，H5 为 0），供 IntersectionObserver 扣除被导航遮挡的区域。 */
    navigationHeightPx: computed(() => layout.value?.totalHeight ?? 0),
  }
```

- [ ] **Step 6: 验证并提交**

Run: `bun test && bun run type-check && bun run check:boundaries`
Expected: 全部通过。

```bash
git pull --rebase
git add src/pages/seat-swap/styleAudit.test.ts src/styles/fi-tokens.css src/static/seat-swap src/composables/useNavigationLayout.ts
git commit -m "feat(mini): add seat swap tokens, icons and style audit test

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
git push
```

---

### Task 2: 整理 `helpers.ts`（迁出 mock、删死代码和空包装）

**Files:**
- Create: `src/pages/seat-swap/fixtures.ts`
- Modify: `src/pages/seat-swap/helpers.ts`（删除第 216–218、262–285 行附近的 3 个 sticky 函数，第 387–397 行的 2 个空包装，第 73–158 行与第 398 行之后的 mock）
- Modify: `src/pages/seat-swap/helpers.test.ts`
- Modify: `src/pages/seat-swap/index.vue`、`src/pages/seat-swap/components/SeatSwapManageSheet.vue`（如引用了被删函数）

**Interfaces:**
- Produces: `fixtures.ts` 导出 `buildSeatSwapMockRegions(): TicketWatchRegion[]`、`buildSeatSwapMockCurrentResponse(input?: { candidateCount?: number; includeMyRequest?: boolean }): SeatSwapCurrentResponse`（签名与原 helpers 中完全一致）。
- 删除：`shouldShowSeatSwapStickyMap`、`readSeatSwapViewportScrollTop`、`resolveSeatSwapStickyMapThreshold`、`formatSeatLabel`、`statusLabel`（helpers 版）。替代：`formatSeatSwapSeatLabel`、`seatSwapStatusLabel`（`src/utils/stadiumRegions.ts`）。

- [ ] **Step 1: 改测试 import（先让测试失败）**

`helpers.test.ts` 顶部：
- 从 `./helpers` 的 import 列表删除 `buildSeatSwapMockCurrentResponse`、`buildSeatSwapMockRegions`、`formatSeatLabel`、`resolveSeatSwapStickyMapThreshold`、`readSeatSwapViewportScrollTop`、`shouldShowSeatSwapStickyMap`、`statusLabel`。
- 新增：

```ts
import { buildSeatSwapMockCurrentResponse, buildSeatSwapMockRegions } from './fixtures'
import { formatSeatSwapSeatLabel, seatSwapStatusLabel } from '../../utils/stadiumRegions'
```

- 把用例里所有 `formatSeatLabel(` 改为 `formatSeatSwapSeatLabel(`，`statusLabel(` 改为 `seatSwapStatusLabel(`（第 187–203 行附近）。
- 删除整个 `describe('seat swap sticky map', ...)`（第 233–259 行，4 个用例）。
- 第 312 行用例标题 `'builds a large candidate list for scroll and sticky-map testing'` 改为 `'builds a large candidate list for scroll testing'`。

Run: `bun test src/pages/seat-swap/helpers.test.ts`
Expected: FAIL，提示无法解析 `./fixtures`。

- [ ] **Step 2: 新建 `fixtures.ts`，原样迁移 mock**

把 `helpers.ts` 中 `buildSeatSwapMockRegions`、`buildSeatSwapMockCurrentResponse`、私有 `buildSeatSwapMockRequest` 三个函数及它们独占使用的常量**剪切**到 `fixtures.ts`，函数体一个字符都不改。文件头：

```ts
/**
 * 换座页开发用 mock 数据，仅在 DEV 且 VITE_SEAT_SWAP_MOCK_LARGE_DATA=1 时由 useSeatSwapPage 使用。
 * 从 helpers.ts 原样迁出，避免与业务纯函数混放。
 */
import type {
  SeatSwapCandidate,
  SeatSwapCurrentResponse,
  SeatSwapDesiredSeat,
  SeatSwapRequest,
} from '../../types/seatSwap'
import type { TicketWatchRegion } from '../../types/ticketWatch'
```

（按迁过来的代码实际用到的类型调整 import；`helpers.ts` 中不再使用的类型 import 同步删除。）

- [ ] **Step 3: 删除死代码与空包装**

从 `helpers.ts` 删除 `shouldShowSeatSwapStickyMap`、`readSeatSwapViewportScrollTop`、`resolveSeatSwapStickyMapThreshold`、`formatSeatLabel`、`statusLabel`。若 `formatSeatSwapSeatLabel`/`seatSwapStatusLabel` 的 import 因此不再使用，一并删除。

运行 `grep -rnwE "formatSeatLabel|statusLabel|buildSeatSwapMock(Regions|CurrentResponse)" src --include=*.vue --include=*.ts`，逐处改：
- `index.vue`：`import { ... } from './helpers'` 中删 `buildSeatSwapMockCurrentResponse`、`buildSeatSwapMockRegions`、`formatSeatLabel`、`statusLabel`；新增 `import { buildSeatSwapMockCurrentResponse, buildSeatSwapMockRegions } from './fixtures'` 与 `import { formatSeatSwapSeatLabel, seatSwapStatusLabel } from '../../utils/stadiumRegions'`；`formatSeatLabel(` → `formatSeatSwapSeatLabel(`，`statusLabel(` → `seatSwapStatusLabel(`。
- `SeatSwapManageSheet.vue` 里的 `statusLabel` 是 **prop 名**，不是函数，保持不动。
- `pages/ticket-watch/components/TicketRefreshStatus.vue` 的 `statusLabel` 与本页无关，不动。

- [ ] **Step 4: 运行测试**

Run: `bun test src/pages/seat-swap && bun run type-check`
Expected: 全部通过；比之前少 4 个用例（sticky map）。

- [ ] **Step 5: 提交**

```bash
git pull --rebase
git add src/pages/seat-swap
git commit -m "refactor(mini): move seat swap mocks to fixtures and drop dead sticky map helpers

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
git push
```

---

### Task 3: 新增浮窗与点击规则纯函数

**Files:**
- Modify: `src/pages/seat-swap/helpers.ts`
- Test: `src/pages/seat-swap/helpers.test.ts`

**Interfaces:**
- Produces（Task 4、6、7 使用）：

```ts
export interface SeatSwapMapFabLabel { text: string; clearable: boolean }
export function resolveSeatSwapMapFabLabel(filterName: string): SeatSwapMapFabLabel
export function resolveSeatSwapStaggerDelayMs(index: number, step?: number, maxSteps?: number): number
export function shouldShowSeatSwapMapFab(input: { available: boolean; mapInViewport: boolean; sheetOpen: boolean }): boolean
export interface SeatSwapMapTapOutcome { filterKey: string; closeSheet: boolean; scrollTo: string | null }
export function resolveSeatSwapMapTap(input: { currentFilterKey: string; tappedKey: string; hasBadge: boolean; fromSheet: boolean }): SeatSwapMapTapOutcome
```

- [ ] **Step 1: 写失败的测试**

在 `helpers.test.ts` 的 import 列表加入 `resolveSeatSwapMapFabLabel`、`resolveSeatSwapStaggerDelayMs`、`shouldShowSeatSwapMapFab`、`resolveSeatSwapMapTap`，文件末尾追加：

```ts
describe('seat swap map floating button', () => {
  test.each([
    ['', { text: '地图', clearable: false }],
    ['   ', { text: '地图', clearable: false }],
    ['112 区', { text: '112 区', clearable: true }],
    [' 112 区 ', { text: '112 区', clearable: true }],
  ])('labels %p', (filterName, expected) => {
    expect(resolveSeatSwapMapFabLabel(filterName)).toEqual(expected)
  })

  const fabCases: Array<[boolean, boolean, boolean, boolean]> = [
    // available, mapInViewport, sheetOpen, expected
    [true, false, false, true],
    [true, true, false, false],
    [true, false, true, false],
    [true, true, true, false],
    [false, false, false, false],
    [false, true, false, false],
    [false, false, true, false],
    [false, true, true, false],
  ]
  test.each(fabCases)('available=%p mapInViewport=%p sheetOpen=%p → %p', (available, mapInViewport, sheetOpen, expected) => {
    expect(shouldShowSeatSwapMapFab({ available, mapInViewport, sheetOpen })).toBe(expected)
  })
})

describe('seat swap list stagger', () => {
  test.each([
    [0, 0], [1, 40], [5, 200], [9, 200], [-1, 0], [Number.NaN, 0], [Number.POSITIVE_INFINITY, 0],
  ])('index %p → %pms', (index, expected) => {
    expect(resolveSeatSwapStaggerDelayMs(index)).toBe(expected)
  })

  test('honours custom step and cap', () => {
    expect(resolveSeatSwapStaggerDelayMs(3, 30, 2)).toBe(60)
  })
})

describe('seat swap map tap outcome', () => {
  test('main map: tapping a region with candidates filters and scrolls without a sheet', () => {
    expect(resolveSeatSwapMapTap({ currentFilterKey: '', tappedKey: '112', hasBadge: true, fromSheet: false }))
      .toEqual({ filterKey: '112', closeSheet: false, scrollTo: '112' })
  })

  test('sheet: tapping a region with candidates filters, closes the sheet and scrolls', () => {
    expect(resolveSeatSwapMapTap({ currentFilterKey: '', tappedKey: '112', hasBadge: true, fromSheet: true }))
      .toEqual({ filterKey: '112', closeSheet: true, scrollTo: '112' })
  })

  test('tapping the active region again clears the filter and stays put', () => {
    expect(resolveSeatSwapMapTap({ currentFilterKey: '112', tappedKey: '112', hasBadge: true, fromSheet: true }))
      .toEqual({ filterKey: '', closeSheet: false, scrollTo: null })
  })

  test('tapping a region without candidates only switches the filter', () => {
    expect(resolveSeatSwapMapTap({ currentFilterKey: '', tappedKey: '113', hasBadge: false, fromSheet: true }))
      .toEqual({ filterKey: '113', closeSheet: false, scrollTo: null })
    expect(resolveSeatSwapMapTap({ currentFilterKey: '', tappedKey: '113', hasBadge: false, fromSheet: false }))
      .toEqual({ filterKey: '113', closeSheet: false, scrollTo: null })
  })
})
```

- [ ] **Step 2: 确认失败**

Run: `bun test src/pages/seat-swap/helpers.test.ts`
Expected: FAIL，`resolveSeatSwapMapFabLabel` 等未导出。

- [ ] **Step 3: 实现**

在 `helpers.ts` 中 `resolveSeatSwapBrowseFilterKey` 之后追加：

```ts
export interface SeatSwapMapFabLabel {
  text: string
  clearable: boolean
}

/** 浮窗文案：无筛选显示"地图"；有筛选显示分区名并允许清除。 */
export function resolveSeatSwapMapFabLabel(filterName: string): SeatSwapMapFabLabel {
  const name = filterName.trim()
  return name ? { text: name, clearable: true } : { text: '地图', clearable: false }
}

/** 列表逐张入场的延迟：每张错开 step 毫秒，最多错开 maxSteps 张。 */
export function resolveSeatSwapStaggerDelayMs(index: number, step = 40, maxSteps = 5): number {
  if (!Number.isFinite(index) || index <= 0) return 0
  return Math.min(Math.floor(index), maxSteps) * step
}

/** 地图划出视口、本场开放、且没有任何弹层打开时才显示浮窗。 */
export function shouldShowSeatSwapMapFab(input: {
  available: boolean
  mapInViewport: boolean
  sheetOpen: boolean
}): boolean {
  return input.available && !input.mapInViewport && !input.sheetOpen
}

export interface SeatSwapMapTapOutcome {
  filterKey: string
  closeSheet: boolean
  scrollTo: string | null
}

/**
 * 首屏地图与地图弹层共用的点击规则：
 * 有意向的分区被选中 → 滚动到该分组（来自弹层时先关闭弹层）；其余情况只切换筛选。
 */
export function resolveSeatSwapMapTap(input: {
  currentFilterKey: string
  tappedKey: string
  hasBadge: boolean
  fromSheet: boolean
}): SeatSwapMapTapOutcome {
  const filterKey = resolveSeatSwapBrowseFilterKey(input.currentFilterKey, input.tappedKey)
  const jump = input.hasBadge && filterKey !== ''
  return {
    filterKey,
    closeSheet: jump && input.fromSheet,
    scrollTo: jump ? input.tappedKey : null,
  }
}
```

- [ ] **Step 4: 确认通过**

Run: `bun test src/pages/seat-swap/helpers.test.ts`
Expected: PASS。

- [ ] **Step 5: 提交**

```bash
git pull --rebase
git add src/pages/seat-swap/helpers.ts src/pages/seat-swap/helpers.test.ts
git commit -m "feat(mini): add seat swap map floating button and tap rules

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
git push
```

---

### Task 4: 拆出 `useSeatSwapPage` 与 `useSeatSwapBrowse`，页面改为装配

本任务**不改任何模板外观**，只把 `index.vue` 的脚本逻辑迁入 composable；完成后页面行为与之前完全一致。

**Files:**
- Create: `src/pages/seat-swap/useSeatSwapPage.ts`
- Create: `src/pages/seat-swap/useSeatSwapBrowse.ts`
- Test: `src/pages/seat-swap/useSeatSwapComposables.test.ts`
- Modify: `src/pages/seat-swap/index.vue`

**Interfaces:**
- Consumes: Task 2 的 `fixtures.ts`；Task 3 的 4 个纯函数。
- Produces（Task 5–7 使用）：

```ts
// useSeatSwapPage.ts
export interface SeatSwapPageLoaders {
  getCurrentSeatSwap: () => Promise<SeatSwapCurrentResponse>
  getTicketWatchRegions: () => Promise<TicketWatchRegion[]>
}
export function useSeatSwapPage(options?: { loaders?: SeatSwapPageLoaders; useMock?: boolean; onLoaded?: () => void }): {
  loading: Ref<boolean>
  errorMessage: Ref<string>
  currentView: Ref<SeatSwapCurrentResponse | null>
  regions: Ref<TicketWatchRegion[]>
  loadPage: () => Promise<void>
}

// useSeatSwapBrowse.ts
export function useSeatSwapBrowse(input: {
  currentView: Ref<SeatSwapCurrentResponse | null>
  regions: Ref<TicketWatchRegion[]>
  sheetOpen: Ref<boolean>
}): {
  browsingFilterKey: Ref<string>
  expandedRegionKeys: Ref<string[]>
  mapInViewport: Ref<boolean>               // 默认 true
  displayCandidates: ComputedRef<SeatSwapCandidate[]>
  seatSwapRegionGroups: ComputedRef<SeatSwapRegionGroup<SeatSwapCandidate>[]>
  filteredCandidates: ComputedRef<SeatSwapCandidate[]>
  browsingFilterName: ComputedRef<string>
  regionBadgeCounts: ComputedRef<Record<string, number>>
  myDesiredKeys: ComputedRef<string[]>
  myDesiredSummary: ComputedRef<string>
  myRequestSeatLabel: ComputedRef<string>
  myRequestStatusLabel: ComputedRef<string>
  mainMapMode: ComputedRef<'browse' | 'filter' | 'published'>
  candidatesCount: ComputedRef<number>
  matchTitle: ComputedRef<string>
  matchSummary: ComputedRef<string>
  mapFabLabel: ComputedRef<SeatSwapMapFabLabel>
  mapFabVisible: ComputedRef<boolean>
  syncExpandedRegionKeys: () => void
  toggleRegionGroup: (key: string) => void
  clearFilter: () => void
  applyMapTap: (key: string, fromSheet: boolean) => SeatSwapMapTapOutcome
}
```

- [ ] **Step 1: 写失败的测试**

```ts
// src/pages/seat-swap/useSeatSwapComposables.test.ts
import { describe, expect, test } from 'bun:test'
import { ref } from 'vue'
import { buildSeatSwapMockCurrentResponse, buildSeatSwapMockRegions } from './fixtures'
import { useSeatSwapBrowse } from './useSeatSwapBrowse'
import { useSeatSwapPage } from './useSeatSwapPage'
import type { SeatSwapCurrentResponse } from '../../types/seatSwap'

function setupBrowse(view: SeatSwapCurrentResponse | null = buildSeatSwapMockCurrentResponse({ candidateCount: 12, includeMyRequest: true })) {
  const currentView = ref(view)
  const regions = ref(buildSeatSwapMockRegions())
  const sheetOpen = ref(false)
  const browse = useSeatSwapBrowse({ currentView, regions, sheetOpen })
  return { currentView, regions, sheetOpen, browse }
}

describe('useSeatSwapPage', () => {
  test('loads the current view and regions, then calls onLoaded', async () => {
    const view = buildSeatSwapMockCurrentResponse({ candidateCount: 3 })
    let loadedCalls = 0
    const page = useSeatSwapPage({
      loaders: {
        getCurrentSeatSwap: async () => view,
        getTicketWatchRegions: async () => buildSeatSwapMockRegions(),
      },
      onLoaded: () => { loadedCalls += 1 },
    })
    expect(page.loading.value).toBe(true)
    await page.loadPage()
    expect(page.loading.value).toBe(false)
    expect(page.errorMessage.value).toBe('')
    expect(page.currentView.value).toBe(view)
    expect(page.regions.value.length).toBeGreaterThan(0)
    expect(loadedCalls).toBe(1)
  })

  test('shows the fallback error message and skips onLoaded when loading fails', async () => {
    let loadedCalls = 0
    const page = useSeatSwapPage({
      loaders: {
        getCurrentSeatSwap: async () => { throw new Error('boom') },
        getTicketWatchRegions: async () => [],
      },
      onLoaded: () => { loadedCalls += 1 },
    })
    await page.loadPage()
    expect(page.loading.value).toBe(false)
    expect(page.errorMessage.value).not.toBe('')
    expect(loadedCalls).toBe(0)
  })

  test('uses mock data without calling loaders when useMock is set', async () => {
    const page = useSeatSwapPage({
      useMock: true,
      loaders: {
        getCurrentSeatSwap: async () => { throw new Error('should not be called') },
        getTicketWatchRegions: async () => { throw new Error('should not be called') },
      },
    })
    await page.loadPage()
    expect(page.errorMessage.value).toBe('')
    expect(page.currentView.value?.candidates.length).toBe(96)
  })
})

describe('useSeatSwapBrowse', () => {
  test('hides my own request from the pool', () => {
    const { currentView, browse } = setupBrowse()
    const myId = currentView.value?.my_request?.request_id
    expect(myId).toBeTruthy()
    expect(browse.displayCandidates.value.some((c) => c.request_id === myId)).toBe(false)
  })

  test('main-map tap on a region with candidates filters and asks to scroll', () => {
    const { browse } = setupBrowse()
    const key = Object.keys(browse.regionBadgeCounts.value)[0]
    const outcome = browse.applyMapTap(key, false)
    expect(outcome).toEqual({ filterKey: key, closeSheet: false, scrollTo: key })
    expect(browse.browsingFilterKey.value).toBe(key)
    expect(browse.mainMapMode.value).toBe('filter')
    expect(browse.mapFabLabel.value.clearable).toBe(true)
  })

  test('clearFilter resets the floating button label', () => {
    const { browse } = setupBrowse()
    const key = Object.keys(browse.regionBadgeCounts.value)[0]
    browse.applyMapTap(key, true)
    browse.clearFilter()
    expect(browse.browsingFilterKey.value).toBe('')
    expect(browse.mapFabLabel.value).toEqual({ text: '地图', clearable: false })
  })

  test('falls back to the region key when the filtered region is unknown', () => {
    const { browse } = setupBrowse()
    browse.applyMapTap('no-such-region', false)
    expect(browse.browsingFilterName.value).toBe('no-such-region')
    expect(browse.filteredCandidates.value).toEqual([])
  })

  test('floating button follows map visibility, sheets and availability', () => {
    const { currentView, sheetOpen, browse } = setupBrowse()
    expect(browse.mapInViewport.value).toBe(true)
    expect(browse.mapFabVisible.value).toBe(false)
    browse.mapInViewport.value = false
    expect(browse.mapFabVisible.value).toBe(currentView.value?.available === true)
    sheetOpen.value = true
    expect(browse.mapFabVisible.value).toBe(false)
    sheetOpen.value = false
    currentView.value = currentView.value ? { ...currentView.value, available: false } : null
    expect(browse.mapFabVisible.value).toBe(false)
  })

  test('expands every group by default and keeps expansion on toggle', () => {
    const { browse } = setupBrowse()
    browse.syncExpandedRegionKeys()
    const keys = browse.seatSwapRegionGroups.value.map((g) => g.region_key)
    expect(browse.expandedRegionKeys.value).toEqual(keys)
    browse.toggleRegionGroup(keys[0])
    expect(browse.expandedRegionKeys.value).not.toContain(keys[0])
    browse.toggleRegionGroup(keys[0])
    expect(browse.expandedRegionKeys.value).toContain(keys[0])
  })
})
```

注：若 `buildSeatSwapMockCurrentResponse` 产生的 `available` 不是 `true`，在 `setupBrowse` 里显式设为 `{ ...view, available: true }`，不要改 fixtures。

- [ ] **Step 2: 确认失败**

Run: `bun test src/pages/seat-swap/useSeatSwapComposables.test.ts`
Expected: FAIL，模块不存在。

- [ ] **Step 3: 实现 `useSeatSwapPage.ts`**

```ts
import { ref } from 'vue'
import { getCurrentSeatSwap } from '../../api/seatSwap'
import { getTicketWatchRegions } from '../../api/ticketWatch'
import type { SeatSwapCurrentResponse } from '../../types/seatSwap'
import type { TicketWatchRegion } from '../../types/ticketWatch'
import { extractApiErrorMessage } from '../../utils/apiError'
import { buildSeatSwapMockCurrentResponse, buildSeatSwapMockRegions } from './fixtures'

export interface SeatSwapPageLoaders {
  getCurrentSeatSwap: () => Promise<SeatSwapCurrentResponse>
  getTicketWatchRegions: () => Promise<TicketWatchRegion[]>
}

const DEFAULT_LOADERS: SeatSwapPageLoaders = { getCurrentSeatSwap, getTicketWatchRegions }

/** 换座页数据层：唯一调用接口的地方。 */
export function useSeatSwapPage(options: {
  loaders?: SeatSwapPageLoaders
  useMock?: boolean
  onLoaded?: () => void
} = {}) {
  const loaders = options.loaders ?? DEFAULT_LOADERS
  const loading = ref(true)
  const errorMessage = ref('')
  const currentView = ref<SeatSwapCurrentResponse | null>(null)
  const regions = ref<TicketWatchRegion[]>([])

  async function loadPage(): Promise<void> {
    loading.value = true
    errorMessage.value = ''
    try {
      if (options.useMock) {
        currentView.value = buildSeatSwapMockCurrentResponse({ candidateCount: 96, includeMyRequest: true })
        regions.value = buildSeatSwapMockRegions()
      } else {
        const [view, regionList] = await Promise.all([
          loaders.getCurrentSeatSwap(),
          loaders.getTicketWatchRegions(),
        ])
        currentView.value = view
        regions.value = regionList
      }
      options.onLoaded?.()
    } catch (err) {
      errorMessage.value = extractApiErrorMessage(err, '换座池加载失败')
    } finally {
      loading.value = false
    }
  }

  return { loading, errorMessage, currentView, regions, loadPage }
}
```

注意：原 `index.vue` 在 mock 分支和成功分支都调用 `syncExpandedRegionKeys()`；现在由页面通过 `onLoaded` 回调调用，语义一致。

- [ ] **Step 4: 实现 `useSeatSwapBrowse.ts`**

把 `index.vue` 里第 180–257 行的计算属性和函数迁入，名称保持不变：

```ts
import { computed, ref, type Ref } from 'vue'
import type { SeatSwapCandidate, SeatSwapCurrentResponse } from '../../types/seatSwap'
import type { TicketWatchRegion } from '../../types/ticketWatch'
import { formatDatetime } from '../../utils/format'
import { formatSeatSwapSeatLabel, seatSwapStatusLabel } from '../../utils/stadiumRegions'
import {
  countSeatSwapDesiredRegions,
  filterOutMySeatSwapRequest,
  filterSeatSwapRequestsByDesiredRegion,
  groupSeatSwapRequestsByRegion,
  resolveDefaultExpandedSeatSwapRegionKeys,
  resolveSeatSwapMapFabLabel,
  resolveSeatSwapMapTap,
  shouldShowSeatSwapMapFab,
  type SeatSwapMapTapOutcome,
} from './helpers'

/** 换座页浏览层：筛选、分组、展开、浮窗状态。不调用接口、不碰 uni API。 */
export function useSeatSwapBrowse(input: {
  currentView: Ref<SeatSwapCurrentResponse | null>
  regions: Ref<TicketWatchRegion[]>
  sheetOpen: Ref<boolean>
}) {
  const { currentView, regions, sheetOpen } = input
  const browsingFilterKey = ref('')
  const expandedRegionKeys = ref<string[]>([])
  // 默认视为可见：observer 尚未回报前不显示浮窗。
  const mapInViewport = ref(true)

  const matchTitle = computed(() => {
    const m = currentView.value?.current_match
    return m ? `${m.home_team_name} VS ${m.away_team_name}` : '暂无当前比赛'
  })
  const matchSummary = computed(() => {
    const raw = currentView.value?.current_match?.kickoff_at
    return raw ? `${formatDatetime(raw)} · 凤凰山专业足球场` : ''
  })
  const candidatesCount = computed(() => currentView.value?.candidates.length || 0)
  const displayCandidates = computed<SeatSwapCandidate[]>(() =>
    filterOutMySeatSwapRequest(currentView.value?.candidates || [], currentView.value?.my_request?.request_id),
  )
  const seatSwapRegionGroups = computed(() => groupSeatSwapRequestsByRegion(displayCandidates.value))
  const filteredCandidates = computed<SeatSwapCandidate[]>(() =>
    browsingFilterKey.value ? filterSeatSwapRequestsByDesiredRegion(displayCandidates.value, browsingFilterKey.value) : [],
  )

  function regionKey(region: TicketWatchRegion): string {
    return region.block_key || region.block_name
  }
  const browsingFilterName = computed(() => {
    const key = browsingFilterKey.value
    if (!key) return ''
    return regions.value.find((r) => regionKey(r) === key)?.block_name || key
  })
  const regionBadgeCounts = computed<Record<string, number>>(() => countSeatSwapDesiredRegions(displayCandidates.value))
  const myDesiredKeys = computed<string[]>(() => currentView.value?.my_request?.desired_seats.map((s) => s.region_key) || [])
  const myDesiredSummary = computed(() => {
    const names = currentView.value?.my_request?.desired_seats.map((s) => s.region_name) || []
    return names.length ? names.join(' / ') : '未选择'
  })
  const myRequestSeatLabel = computed(() =>
    currentView.value?.my_request ? formatSeatSwapSeatLabel(currentView.value.my_request) : '',
  )
  const myRequestStatusLabel = computed(() => seatSwapStatusLabel(currentView.value?.my_request?.status || ''))
  const mainMapMode = computed<'browse' | 'filter' | 'published'>(() => {
    if (browsingFilterKey.value) return 'filter'
    if (currentView.value?.my_request) return 'published'
    return 'browse'
  })
  const mapFabLabel = computed(() => resolveSeatSwapMapFabLabel(browsingFilterName.value))
  const mapFabVisible = computed(() => shouldShowSeatSwapMapFab({
    available: currentView.value?.available === true,
    mapInViewport: mapInViewport.value,
    sheetOpen: sheetOpen.value,
  }))

  function syncExpandedRegionKeys(): void {
    const available = seatSwapRegionGroups.value.map((g) => g.region_key)
    expandedRegionKeys.value = expandedRegionKeys.value.filter((k) => available.includes(k))
    if (!expandedRegionKeys.value.length && available.length) {
      expandedRegionKeys.value = resolveDefaultExpandedSeatSwapRegionKeys(available)
    }
  }
  function toggleRegionGroup(key: string): void {
    expandedRegionKeys.value = expandedRegionKeys.value.includes(key)
      ? expandedRegionKeys.value.filter((k) => k !== key)
      : [...expandedRegionKeys.value, key]
  }
  function clearFilter(): void {
    browsingFilterKey.value = ''
  }
  function applyMapTap(key: string, fromSheet: boolean): SeatSwapMapTapOutcome {
    const outcome = resolveSeatSwapMapTap({
      currentFilterKey: browsingFilterKey.value,
      tappedKey: key,
      hasBadge: Boolean(regionBadgeCounts.value[key]),
      fromSheet,
    })
    browsingFilterKey.value = outcome.filterKey
    return outcome
  }

  return {
    browsingFilterKey, expandedRegionKeys, mapInViewport,
    matchTitle, matchSummary, candidatesCount, displayCandidates, seatSwapRegionGroups,
    filteredCandidates, browsingFilterName, regionBadgeCounts, myDesiredKeys, myDesiredSummary,
    myRequestSeatLabel, myRequestStatusLabel, mainMapMode, mapFabLabel, mapFabVisible,
    syncExpandedRegionKeys, toggleRegionGroup, clearFilter, applyMapTap,
  }
}
```

注：原 `browsingFilterName` 在找不到分区时返回 `''`，模板里用 `filterName || filterKey` 兜底；这里直接回退到 key，模板兜底仍然成立，显示结果不变。

- [ ] **Step 5: 确认 composable 测试通过**

Run: `bun test src/pages/seat-swap/useSeatSwapComposables.test.ts`
Expected: PASS。

- [ ] **Step 6: 改 `index.vue` 脚本为装配**

替换 `<script setup>` 中第 107–361 行的对应部分（模板本任务不动）：

```ts
import { buildSharePath } from '../../utils/shareSource'
import { useNavigationLayout } from '../../composables/useNavigationLayout'
import { computed, nextTick, ref } from 'vue'
import { onShareAppMessage, onShow } from '@dcloudio/uni-app'
// 组件 import 保持原样（本任务不改模板）
import { useAppSession } from '../../stores/appSession'
import type { SeatSwapCandidate } from '../../types/seatSwap'
import { buildSeatSwapRegionAnchorId } from './helpers'
import { useSeatSwapActions } from './useSeatSwapActions'
import { useSeatSwapBrowse } from './useSeatSwapBrowse'
import { useSeatSwapPage } from './useSeatSwapPage'

onShareAppMessage(() => ({
  title: '球迷换座位，看看有没有合适的位置',
  path: buildSharePath('/pages/seat-swap/index', 'swap'),
}))

const { navigationPageStyle } = useNavigationLayout()

const USE_SEAT_SWAP_MOCK_LARGE_DATA =
  import.meta.env.DEV && String(import.meta.env.VITE_SEAT_SWAP_MOCK_LARGE_DATA || '').trim() === '1'

const isLoggedIn = useAppSession().hasAccessToken
const submitting = ref(false)
const publishSheetVisible = ref(false)
const manageSheetVisible = ref(false)
const pendingConfirmTarget = ref<SeatSwapCandidate | null>(null)
const pendingMatchedCancelTargetId = ref('')
const miniProgramNoticeEnabled = ref(false)
const cancelReason = ref('')

const anySheetOpen = computed(() => publishSheetVisible.value || manageSheetVisible.value)
const pageLockStyle = computed(() => anySheetOpen.value ? 'overflow: hidden;' : '')

const { loading, errorMessage, currentView, regions, loadPage } = useSeatSwapPage({
  useMock: USE_SEAT_SWAP_MOCK_LARGE_DATA,
  onLoaded: () => browse.syncExpandedRegionKeys(),
})
const browse = useSeatSwapBrowse({ currentView, regions, sheetOpen: anySheetOpen })
const {
  browsingFilterKey, expandedRegionKeys, matchTitle, matchSummary, candidatesCount,
  seatSwapRegionGroups, filteredCandidates, browsingFilterName, regionBadgeCounts,
  myDesiredKeys, myDesiredSummary, myRequestSeatLabel, myRequestStatusLabel, mainMapMode,
  toggleRegionGroup, clearFilter,
} = browse

const { submitForm, deleteRequest, confirmCandidate, cancelCandidateConfirmation, submitMatchedCancel } = useSeatSwapActions({
  submitting, currentView, publishSheetVisible, pendingConfirmTarget, manageSheetVisible,
  miniProgramNoticeEnabled, pendingMatchedCancelTargetId, cancelReason, loadPage, openPublishSheetForCandidate,
})

function handleMainMapTap(key: string): void {
  const outcome = browse.applyMapTap(key, false)
  if (outcome.scrollTo) void scrollToSeatSwapRegion(outcome.scrollTo)
}
// scrollToSeatSwapRegion / openPublishSheet / openPublishSheetForCandidate / openManageSheet /
// closeManageSheet / openEditFromManage / handleDockCtaTap / openMatchedCancelForCandidate /
// goToUserPage / onShow 保持原实现不变。
```

`onLoaded` 回调里引用的 `browse` 在 `loadPage` 被调用时（`onShow`）已初始化，不存在 TDZ 问题。`useSeatSwapActions` 的入参与原来完全一致（`loadPage` 改由 `useSeatSwapPage` 提供，签名相同）。删除不再使用的 import（`getCurrentSeatSwap`、`getTicketWatchRegions`、`extractApiErrorMessage`、`formatDatetime`、`fixtures` 等）。

- [ ] **Step 7: 全量验证并提交**

Run: `bun test && bun run type-check && bun run check:boundaries`
Expected: 全部通过。边界脚本允许页面目录内的 `useSeatSwapPage.ts` import `api/**`（与原 `index.vue` 同目录）；若报错，说明规则不同，**停下来报告**，不要加 ALLOWLIST。

```bash
git pull --rebase
git add src/pages/seat-swap
git commit -m "refactor(mini): split seat swap page into data and browse composables

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
git push
```

---

### Task 5: 比赛卡、提示条、骨架屏与背景遮罩

**Files:**
- Create: `src/pages/seat-swap/components/SeatSwapMatchCard.vue`
- Delete: `src/pages/seat-swap/components/SeatSwapHero.vue`
- Create: `src/pages/seat-swap/components/SeatSwapNotice.vue`
- Create: `src/pages/seat-swap/components/SeatSwapSkeleton.vue`
- Modify: `src/pages/seat-swap/index.vue`（模板与样式）
- Modify: `src/pages/seat-swap/styleAudit.test.ts`（纳入新文件与 `index.vue`）

**Interfaces:**
- Produces: `SeatSwapMatchCard` props `{ matchTitle: string; matchSummary: string; candidatesCount: number }` + 默认插槽；`SeatSwapNotice` props `{ text: string; actionLabel?: string }`，emit `action`；`SeatSwapSkeleton` 无 props。

- [ ] **Step 1: 把新文件纳入审计（先失败）**

`styleAudit.test.ts`：

```ts
const AUDITED_FILES: string[] | 'ALL' = [
  'index.vue',
  'components/SeatSwapMatchCard.vue',
  'components/SeatSwapNotice.vue',
  'components/SeatSwapSkeleton.vue',
]
```

Run: `bun test src/pages/seat-swap/styleAudit.test.ts`
Expected: FAIL（新文件不存在；`index.vue` 有 gradient、hex、数字字重）。

- [ ] **Step 2: `SeatSwapMatchCard.vue`**

```vue
<template>
  <FiPanel>
    <view class="match-card">
      <view class="match-card__icon-box">
        <image class="match-card__icon" src="/static/seat-swap/arrow-left-right.svg" mode="aspectFit" />
      </view>
      <view class="match-card__body">
        <text class="match-card__eyebrow">当前比赛 · 换座撮合</text>
        <text class="match-card__title">{{ matchTitle }}</text>
        <text v-if="matchSummary" class="match-card__summary">{{ matchSummary }}</text>
      </view>
      <text class="match-card__pill">{{ candidatesCount }} 条意向</text>
    </view>
    <!-- 二期"本场动态"预留；一期不传内容。 -->
    <slot />
  </FiPanel>
</template>

<script setup lang="ts">
import FiPanel from '../../../components/FiPanel.vue'

defineProps<{
  matchTitle: string
  matchSummary: string
  candidatesCount: number
}>()
</script>

<style scoped>
.match-card {
  display: flex;
  align-items: center;
  gap: var(--fi-space-18);
}

.match-card__icon-box {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  width: var(--fi-primitive-space-64);
  height: var(--fi-primitive-space-64);
  border-radius: var(--fi-radius-md);
  background: rgba(var(--fi-primitive-red-rgb), 0.1);
}

.match-card__icon {
  width: var(--fi-space-36, 36rpx);
  height: var(--fi-space-36, 36rpx);
}

.match-card__body {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-6);
}

.match-card__eyebrow {
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-22);
  font-weight: var(--fi-weight-bold);
}

.match-card__title {
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-36);
  font-weight: var(--fi-weight-extrabold);
  line-height: var(--fi-leading-tight);
}

.match-card__summary {
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-24);
  line-height: var(--fi-leading-normal);
}

.match-card__pill {
  flex-shrink: 0;
  padding: var(--fi-space-12) var(--fi-space-20);
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-page);
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-22);
  font-weight: var(--fi-weight-bold);
  line-height: var(--fi-leading-none);
  white-space: nowrap;
}
</style>
```

先 `grep -nE -e "--fi-space-36|--fi-primitive-space-64|--fi-space-64" src/styles/fi-tokens.css`：若有 semantic `--fi-space-64`/`--fi-space-36`，改用 semantic 并去掉 `var()` 回退值；若没有 `--fi-space-36`，用 `--fi-primitive-icon-md`（先确认其值为 36rpx）。不得保留写死的回退数值。

- [ ] **Step 3: `SeatSwapNotice.vue`**

```vue
<template>
  <view class="notice">
    <image class="notice__icon" src="/static/seat-swap/info.svg" mode="aspectFit" />
    <text class="notice__text">{{ text }}</text>
    <button v-if="actionLabel" class="notice__action" hover-class="notice__action--pressed" @tap="emit('action')">
      {{ actionLabel }}
    </button>
  </view>
</template>

<script setup lang="ts">
defineProps<{ text: string; actionLabel?: string }>()
const emit = defineEmits<{ (e: 'action'): void }>()
</script>

<style scoped>
.notice {
  display: flex;
  align-items: center;
  gap: var(--fi-space-12);
  padding: var(--fi-space-18) var(--fi-space-22);
  border-radius: var(--fi-radius-md);
  background: var(--fi-color-page);
}

.notice__icon {
  flex-shrink: 0;
  width: var(--fi-space-32);
  height: var(--fi-space-32);
}

.notice__text {
  flex: 1;
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-24);
  line-height: var(--fi-leading-normal);
}

.notice__action {
  flex-shrink: 0;
  margin: 0;
  padding: var(--fi-space-10) var(--fi-space-20);
  border-radius: var(--fi-radius-round);
  background: var(--fi-primitive-ink);
  color: var(--fi-primitive-white);
  font-size: var(--fi-font-22);
  font-weight: var(--fi-weight-bold);
  line-height: var(--fi-leading-snug);
}

.notice__action::after { border: 0; }
.notice__action--pressed { opacity: var(--fi-primitive-opacity-pressed); }
</style>
```

- [ ] **Step 4: `SeatSwapSkeleton.vue`**

```vue
<template>
  <view class="fi-skeleton-stack" role="status" aria-label="换座池加载中">
    <view class="fi-skeleton-panel" aria-hidden="true">
      <view class="fi-skeleton-heading">
        <view class="fi-skeleton-block seat-skeleton__icon" />
        <view class="fi-skeleton-copy">
          <view class="fi-skeleton-block fi-skeleton-kicker" />
          <view class="fi-skeleton-block fi-skeleton-title" />
        </view>
        <view class="fi-skeleton-block seat-skeleton__pill" />
      </view>
    </view>
    <view class="fi-skeleton-panel" aria-hidden="true">
      <view class="fi-skeleton-block seat-skeleton__map" />
    </view>
    <view class="fi-skeleton-panel" aria-hidden="true">
      <view class="fi-skeleton-copy">
        <view class="fi-skeleton-block fi-skeleton-kicker" />
        <view class="fi-skeleton-block fi-skeleton-title" />
      </view>
      <view class="seat-skeleton__list">
        <view v-for="row in 3" :key="row" class="seat-skeleton__row">
          <view class="fi-skeleton-block fi-skeleton-avatar" />
          <view class="fi-skeleton-copy">
            <view class="fi-skeleton-block fi-skeleton-kicker" />
            <view class="fi-skeleton-block seat-skeleton__line" />
          </view>
        </view>
      </view>
    </view>
  </view>
</template>

<style scoped>
.seat-skeleton__icon { flex-shrink: 0; width: var(--fi-primitive-space-64); height: var(--fi-primitive-space-64); border-radius: var(--fi-radius-md); }
.seat-skeleton__pill { flex-shrink: 0; width: 22%; height: var(--fi-space-40); }
.seat-skeleton__map { width: 100%; height: calc(var(--fi-primitive-space-64) * 6); border-radius: var(--fi-radius-md); }
.seat-skeleton__list { display: grid; gap: var(--fi-space-20); margin-top: var(--fi-space-20); }
.seat-skeleton__row { display: flex; align-items: center; gap: var(--fi-space-16); }
.seat-skeleton__line { width: 72%; }
</style>
```

（地图占位高度用 token 计算，约 384rpx，与真实地图首屏高度接近；实测差距大时调整倍数，不写死 rpx。）

- [ ] **Step 5: 接到 `index.vue` 模板并改背景遮罩**

- `import SeatSwapHero` 改为 `import SeatSwapMatchCard from './components/SeatSwapMatchCard.vue'`，新增 `SeatSwapNotice`、`SeatSwapSkeleton` 的 `.vue` 直接导入，删除 `SeatSwapHero.vue` 文件。
- 加载态：`<view v-if="loading" class="state-card">…</view>` 改为 `<SeatSwapSkeleton v-if="loading" />`。
- `<SeatSwapHero …/>` 外层 `<view>` 保留，组件换成 `<SeatSwapMatchCard :match-title="matchTitle" :match-summary="matchSummary" :candidates-count="candidatesCount" />`。
- 两个 `.info-row` 换成：

```vue
<SeatSwapNotice v-if="!currentView?.available" text="换座撮合只在成都蓉城当前比赛开放。" />
<SeatSwapNotice
  v-else-if="!isLoggedIn"
  text="未登录可浏览脱敏意向;发布、确认和查看联系方式需要登录。"
  action-label="去登录"
  @action="goToUserPage"
/>
```

（文案逐字照抄原模板，包括半角分号。）
- 内容整体淡入：`<template v-else>` 内第一层加一个包裹 `<view class="page-content">`，样式 `animation: fi-fade-in-content 200ms var(--fi-motion-ease-out) both;`。在本文件 `<style>` 中定义 `@keyframes fi-fade-in-content { from { opacity: 0; } to { opacity: 1; } }`，`.page-content` 需 `display: flex; flex-direction: column; gap: var(--fi-space-card-gap);`（因为 `.page` 的 gap 不再作用到内部子项）。
- 样式：删除 `.info-row*`、`.state-card` 的加载相关规则；`.state-card--error` 的 `color: #b42318` 改 `color: var(--fi-color-primary-deep)`；`.ghost-action` 的 `font-weight` 改 token；`.page-bg-fade` 改为：

```css
.page-bg-fade {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100vh;
  background: var(--fi-color-page-soft);
  pointer-events: none;
  z-index: 0;
}

/* 照片区浅遮罩：纯色半透明，替代原渐变（spec 第 7 节） */
.page-bg-fade::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 32vh;
  background: rgba(var(--fi-primitive-white-rgb), 0.5);
}
```

同时把 `.page-bg-fade` 的实底改为只覆盖照片区以下：给 `.page-bg-fade` 设 `top: 32vh; height: 68vh;`，`::before` 改为挂在 `.page-bg-img` 的兄弟节点上会更清晰——实现方式二选一，以"照片上 32vh 可见且被半透明白遮罩、以下为页面底色"为准，最终在交付说明中写明采用哪种。
- 错误态 `.state-card` 保留（文案和重试按钮不变），只 token 化。

- [ ] **Step 6: 验证并提交**

Run: `bun test && bun run type-check && bun run check:boundaries && bun run verify:mp-components`
Expected: 全部通过（含审计新纳入的 4 个文件）。

```bash
git pull --rebase
git add src/pages/seat-swap
git commit -m "feat(mini): restyle seat swap header, notices and loading skeleton

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
git push
```

---

### Task 6: 换座池、候选卡、Dock 与浮窗组件

**Files:**
- Modify: `src/pages/seat-swap/components/SeatSwapPoolList.vue`
- Modify: `src/pages/seat-swap/components/SeatSwapCandidateCard.vue`
- Modify: `src/pages/seat-swap/components/SeatSwapDock.vue`
- Create: `src/pages/seat-swap/components/SeatSwapMapFab.vue`
- Modify: `src/pages/seat-swap/styleAudit.test.ts`

**Interfaces:**
- Consumes: `resolveSeatSwapStaggerDelayMs`（Task 3）。
- Produces: `SeatSwapMapFab` props `{ visible: boolean; label: string; clearable: boolean }`，emits `open`、`clear`。`SeatSwapPoolList`、`SeatSwapCandidateCard`、`SeatSwapDock` 的 props/emits **不变**。

- [ ] **Step 1: 纳入审计（先失败）**

`AUDITED_FILES` 追加 `'components/SeatSwapPoolList.vue'`、`'components/SeatSwapCandidateCard.vue'`、`'components/SeatSwapDock.vue'`、`'components/SeatSwapMapFab.vue'`。

Run: `bun test src/pages/seat-swap/styleAudit.test.ts`
Expected: FAIL（列出 hex / gradient / 数字字重所在行）。

- [ ] **Step 2: `SeatSwapPoolList.vue`**

模板改动：
- 根节点换成 `<FiPanel>`（`import FiPanel from '../../../components/FiPanel.vue'`、`import FiSectionHeading from '../../../components/FiSectionHeading.vue'`）。
- 非筛选态标题：`<FiSectionHeading kicker="换座池" title="按分区浏览" />`，删除 `.section-row`。
- 筛选态标题：

```vue
<FiSectionHeading kicker="目标座位" :title="filterName || filterKey" align="center">
  <template #meta>
    <view class="filter-meta">
      <text class="filter-meta__count">{{ filteredCandidates.length }} 条</text>
      <button class="filter-meta__clear" hover-class="filter-meta__clear--pressed" @tap="emit('clear-filter')">✕ 清除</button>
    </view>
  </template>
</FiSectionHeading>
```

- 分组行的"展开/收起"文字换成图标：

```vue
<image
  class="group-row__caret"
  :class="{ 'group-row__caret--open': expandedKeys.includes(group.region_key) }"
  src="/static/seat-swap/chevron-down.svg"
  mode="aspectFit"
  :aria-label="expandedKeys.includes(group.region_key) ? '收起' : '展开'"
/>
```

- 列表入场动画：候选卡外包一层真实 `view`（布局与动画都放在包裹节点上，符合小程序样式隔离规则）：

```vue
<view class="candidate-list" :key="`group-${group.region_key}`">
  <view
    v-for="(candidate, index) in group.requests"
    :key="candidate.request_id"
    class="candidate-list__item"
    :style="{ animationDelay: `${resolveSeatSwapStaggerDelayMs(index)}ms` }"
  >
    <SeatSwapCandidateCard … 原有绑定不变 … />
  </view>
</view>
```

筛选态列表同样包裹，且容器 `:key="`filter-${filterKey}`"`，使切换筛选时重新播放入场。

样式（替换原样式中对应部分，保留 `.empty-row`、`.group-row` 的结构，全部 token 化）：

```css
.candidate-list { display: flex; flex-direction: column; gap: var(--fi-space-card-gap); margin: var(--fi-space-12) 0 var(--fi-space-card-gap); }
.candidate-list__item { animation: fi-rise-in var(--fi-motion-duration-base) var(--fi-motion-ease-out) both; }

.filter-meta { display: flex; align-items: center; gap: var(--fi-space-12); }
.filter-meta__count { color: var(--fi-color-text-muted); font-size: var(--fi-font-22); }
.filter-meta__clear {
  margin: 0;
  padding: var(--fi-space-8) var(--fi-space-18);
  border-radius: var(--fi-radius-round);
  border: var(--fi-border-width) solid var(--fi-color-border-chip);
  background: var(--fi-color-page);
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-22);
  font-weight: var(--fi-weight-bold);
  line-height: var(--fi-leading-snug);
}
.filter-meta__clear::after { border: 0; }
.filter-meta__clear--pressed { opacity: var(--fi-primitive-opacity-pressed); }

.group-row__name { color: var(--fi-color-text-strong); font-size: var(--fi-font-28); font-weight: var(--fi-weight-bold); }
.group-row__count { color: var(--fi-color-text-muted); font-size: var(--fi-font-22); }
.group-row__hit {
  padding: var(--fi-space-4) var(--fi-space-14);
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-seat-desired-bg);
  color: var(--fi-color-seat-desired);
  font-size: var(--fi-font-20);
  font-weight: var(--fi-weight-bold);
}
.group-row__caret {
  flex-shrink: 0;
  width: var(--fi-space-32);
  height: var(--fi-space-32);
  transition: transform var(--fi-motion-duration-base) var(--fi-motion-ease-out);
}
.group-row__caret--open { transform: rotate(180deg); }

@media (prefers-reduced-motion: reduce) {
  .candidate-list__item { animation-duration: 150ms; animation-name: fi-overlay-fade-in; animation-delay: 0ms !important; }
  .group-row__caret { transition: none; }
}
```

`group-row` 的 `border-top: 1rpx solid …` 改为 `var(--fi-border-width) solid var(--fi-color-divider-soft)`；`filter-row` 相关规则随结构删除。script 里 `import { buildSeatSwapRegionAnchorId, resolveSeatSwapCandidateAction, resolveSeatSwapStaggerDelayMs } from '../helpers'`。

`fi-rise-in`、`fi-overlay-fade-in` 是 `App.vue` 里的全局 keyframes，组件内可直接引用。

- [ ] **Step 3: `SeatSwapCandidateCard.vue`**

模板：状态胶囊加圆点：

```vue
<view v-if="statusText" class="candidate__status" :class="statusClass">
  <view class="candidate__status-dot" />
  <text>{{ statusText }}</text>
</view>
```

`statusClass` 改为只区分三类：

```ts
const statusClass = computed(() => ({
  'candidate__status--ready': props.candidate.status === 'communicable',
  'candidate__status--attention': props.candidate.status === 'peer_confirmed_me',
  'candidate__status--matched': props.candidate.status === 'matched',
}))
```

（`waiting_peer_confirmation` 用默认中性样式。）BEM 前缀由 `seat-swap-candidate` 统一改为 `candidate`，模板与样式同步改；其余模板、按钮条件、文案、emit 不变。

样式整体替换为：

```css
.candidate {
  padding: var(--fi-space-22);
  border-radius: var(--fi-radius-md);
  border: var(--fi-border-card);
  background: var(--fi-color-surface);
  box-shadow: var(--fi-shadow-card);
}
.candidate__main { display: flex; align-items: stretch; gap: var(--fi-space-18); }
.candidate__content { flex: 1; min-width: 0; }
.candidate__user { display: flex; align-items: flex-start; gap: var(--fi-space-16); }
.candidate__avatar {
  flex-shrink: 0;
  width: var(--fi-primitive-space-64);
  height: var(--fi-primitive-space-64);
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-page);
}
.candidate__avatar--fallback {
  display: flex; align-items: center; justify-content: center;
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-28);
  font-weight: var(--fi-weight-bold);
}
.candidate__id { flex: 1; min-width: 0; }
.candidate__name { display: block; color: var(--fi-color-text-strong); font-size: var(--fi-font-28); font-weight: var(--fi-weight-bold); }

.candidate__seat,
.candidate__wants {
  display: inline-flex;
  align-items: center;
  max-width: 100%;
  overflow: hidden;
  border-radius: var(--fi-radius-sm);
}
.candidate__seat { margin-top: var(--fi-space-8); background: var(--fi-color-seat-current-bg); color: var(--fi-color-seat-current); }
.candidate__wants {
  max-width: calc(100% - var(--fi-primitive-space-64) - var(--fi-space-16));
  margin-top: var(--fi-space-12);
  margin-left: calc(var(--fi-primitive-space-64) + var(--fi-space-16));
  background: var(--fi-color-seat-desired-bg);
  color: var(--fi-color-seat-desired);
}
.candidate__seat-label,
.candidate__wants-label {
  flex-shrink: 0;
  padding: var(--fi-space-6) var(--fi-space-10) var(--fi-space-6) var(--fi-space-12);
  font-size: var(--fi-font-20);
  font-weight: var(--fi-weight-bold);
  line-height: var(--fi-leading-none);
}
.candidate__seat-label { background: rgba(var(--fi-primitive-red-rgb), 0.12); }
.candidate__wants-label { background: rgba(var(--fi-primitive-ink-rgb), 0.04); }
.candidate__seat-value,
.candidate__wants-value {
  min-width: 0;
  padding: var(--fi-space-6) var(--fi-space-14) var(--fi-space-6) var(--fi-space-10);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  line-height: var(--fi-leading-none);
}
.candidate__seat-value { font-size: var(--fi-font-26); font-weight: var(--fi-weight-extrabold); }
.candidate__wants-value { font-size: var(--fi-font-24); font-weight: var(--fi-weight-bold); }

.candidate__status {
  flex-shrink: 0;
  display: inline-flex;
  align-items: center;
  gap: var(--fi-space-8);
  padding: var(--fi-space-6) var(--fi-space-14);
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-page);
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-20);
  font-weight: var(--fi-weight-bold);
  line-height: var(--fi-leading-none);
}
.candidate__status-dot { width: var(--fi-space-12); height: var(--fi-space-12); border-radius: var(--fi-radius-round); background: var(--fi-color-text-muted); }
.candidate__status--ready .candidate__status-dot { background: var(--fi-color-seat-desired); }
.candidate__status--attention { background: rgba(var(--fi-primitive-red-rgb), 0.1); color: var(--fi-color-primary); }
.candidate__status--attention .candidate__status-dot { background: var(--fi-color-primary); }

.candidate__contact {
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-4);
  margin-top: var(--fi-space-12);
  padding: var(--fi-space-16) var(--fi-space-18);
  border-radius: var(--fi-radius-sm);
  background: var(--fi-color-page);
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-22);
}
.candidate__contact-line { display: block; }

.candidate__action {
  flex-shrink: 0;
  align-self: stretch;
  display: flex;
  align-items: center;
  justify-content: center;
  width: var(--fi-component-seat-swap-action-width);
  min-height: var(--fi-component-seat-swap-action-min-height);
  margin: 0;
  padding: var(--fi-space-18) var(--fi-space-20);
  border-radius: var(--fi-radius-md);
  color: var(--fi-primitive-white);
  font-size: var(--fi-font-28);
  font-weight: var(--fi-weight-bold);
  line-height: var(--fi-leading-snug);
  text-align: center;
  transition: transform var(--fi-motion-duration-fast) var(--fi-motion-ease-out);
}
.candidate__action::after { border: 0; }
.candidate__action--pressed { transform: scale(0.96); }
.candidate__action--primary { background: var(--fi-primitive-ink); box-shadow: var(--fi-shadow-soft); }
.candidate__action--secondary {
  background: var(--fi-color-surface);
  color: var(--fi-color-text-primary);
  border: var(--fi-border-width) solid rgba(var(--fi-primitive-ink-rgb), 0.16);
}
.candidate__action--matched-cancel {
  background: rgba(var(--fi-primitive-white-rgb), 0.18);
  border: var(--fi-border-width) solid rgba(var(--fi-primitive-white-rgb), 0.28);
}

/* 已匹配：纯色主色卡（原红色渐变） */
.candidate--matched {
  background: var(--fi-color-primary);
  border-color: var(--fi-color-primary-deep);
  box-shadow: var(--fi-shadow-card-strong);
}
.candidate--matched .candidate__avatar { background: rgba(var(--fi-primitive-white-rgb), 0.18); }
.candidate--matched .candidate__avatar--fallback,
.candidate--matched .candidate__name,
.candidate--matched .candidate__seat,
.candidate--matched .candidate__wants,
.candidate--matched .candidate__contact,
.candidate--matched .candidate__status { color: var(--fi-primitive-white); }
.candidate--matched .candidate__seat,
.candidate--matched .candidate__wants,
.candidate--matched .candidate__status,
.candidate--matched .candidate__contact { background: rgba(var(--fi-primitive-white-rgb), 0.14); }
.candidate--matched .candidate__seat-label,
.candidate--matched .candidate__wants-label { background: rgba(var(--fi-primitive-white-rgb), 0.12); }
.candidate--matched .candidate__status-dot { background: var(--fi-primitive-white); }
.candidate--matched .candidate__contact {
  margin-top: var(--fi-space-16);
  padding: var(--fi-space-20) var(--fi-space-22);
  border: var(--fi-border-width) solid rgba(var(--fi-primitive-white-rgb), 0.16);
  font-size: var(--fi-font-26);
  gap: var(--fi-space-8);
}
```

按钮加 `hover-class="candidate__action--pressed"`。注意：原已匹配联系方式字号 25rpx，归入相邻刻度 26（设计系统允许 ≤2rpx 就近归档），交付说明中写明。

- [ ] **Step 4: `SeatSwapDock.vue`**

模板与 script 不变。样式中：
- `.dock-cta`、`.dock-status`：`min-height: var(--fi-component-seat-swap-dock-height); box-sizing: border-box;`，`border-radius: var(--fi-radius-md)`，`box-shadow: var(--fi-shadow-floating)`，字重 `var(--fi-weight-bold)`，删除 `letter-spacing: 1rpx`。
- `.dock-cta` 加 `transition: transform var(--fi-motion-duration-fast) var(--fi-motion-ease-out);`，按钮加 `hover-class="dock-cta--pressed"`，`.dock-cta--pressed { transform: scale(0.98); }`。
- `.dock-cta__icon` 的 `rgba(255, 255, 255, 0.18)` → `rgba(var(--fi-primitive-white-rgb), 0.18)`；`border-radius: 50%` → `var(--fi-radius-round)`；`width/height: 36rpx` → `var(--fi-primitive-icon-md)`（先确认其值为 36rpx）。
- `.dock-status__dot` 的 `#9aa0aa` → `var(--fi-color-text-caption)`，其 box-shadow → `0 0 0 var(--fi-space-6) rgba(var(--fi-primitive-white-rgb), 0.12)`。
- `.dock-status__seat--current`：`background: var(--fi-color-seat-current-bg); color: var(--fi-color-seat-current);`
- `.dock-status__seat--desired`：`background: var(--fi-color-seat-desired-bg); color: var(--fi-color-seat-desired);`
- `.dock-status__seat` 的 `border-radius: 12rpx` → `var(--fi-radius-sm)`；其他 `rgba(255, 255, 255, x)` 全部改为 `rgba(var(--fi-primitive-white-rgb), x)`。
- 所有 `font-weight: 400` 改为 `var(--fi-weight-bold)`（CTA、管理）或删除（继承正文）。

- [ ] **Step 5: `SeatSwapMapFab.vue`**

```vue
<template>
  <view
    class="map-fab"
    :class="{ 'map-fab--visible': visible }"
    role="button"
    :aria-label="clearable ? `当前筛选 ${label}，打开球场地图` : '打开球场地图'"
    :aria-hidden="!visible"
  >
    <view class="map-fab__main" hover-class="map-fab__main--pressed" @tap="emit('open')">
      <image class="map-fab__icon" src="/static/seat-swap/map.svg" mode="aspectFit" />
      <view class="map-fab__labels">
        <text :key="`in-${label}`" class="map-fab__label map-fab__label--in">{{ label }}</text>
        <text v-if="leavingLabel" :key="`out-${leavingLabel}`" class="map-fab__label map-fab__label--out">{{ leavingLabel }}</text>
      </view>
    </view>
    <view v-if="clearable" class="map-fab__clear" hover-class="map-fab__clear--pressed" aria-label="清除筛选" @tap.stop="emit('clear')">
      <text>✕</text>
    </view>
  </view>
</template>

<script setup lang="ts">
import { onBeforeUnmount, ref, watch } from 'vue'

const props = defineProps<{ visible: boolean; label: string; clearable: boolean }>()
const emit = defineEmits<{ (e: 'open'): void; (e: 'clear'): void }>()

// 文案交叉过渡：旧文案保留一个动画时长后移除。
const LABEL_SWAP_MS = 220
const leavingLabel = ref('')
let leaveTimer: ReturnType<typeof setTimeout> | null = null

watch(() => props.label, (_next, previous) => {
  if (!previous) return
  leavingLabel.value = previous
  if (leaveTimer) clearTimeout(leaveTimer)
  leaveTimer = setTimeout(() => { leavingLabel.value = '' }, LABEL_SWAP_MS)
})

onBeforeUnmount(() => { if (leaveTimer) clearTimeout(leaveTimer) })
</script>

<style scoped>
.map-fab {
  position: fixed;
  right: var(--fi-space-24);
  bottom: calc(var(--fi-space-24) + var(--window-bottom) + var(--fi-component-seat-swap-dock-height) + var(--fi-space-16));
  z-index: 9;
  display: flex;
  align-items: center;
  border-radius: var(--fi-radius-round);
  background: var(--fi-primitive-ink);
  box-shadow: var(--fi-shadow-floating);
  opacity: 0;
  transform: translateY(var(--fi-space-16)) scale(0.95);
  pointer-events: none;
  transition:
    opacity 200ms var(--fi-motion-ease-in),
    transform 200ms var(--fi-motion-ease-in);
}

.map-fab--visible {
  opacity: 1;
  transform: translateY(0) scale(1);
  pointer-events: auto;
  transition:
    opacity var(--fi-motion-duration-slow) var(--fi-motion-ease-out),
    transform var(--fi-motion-duration-slow) var(--fi-motion-ease-out);
}

.map-fab__main {
  display: flex;
  align-items: center;
  gap: var(--fi-space-10);
  padding: var(--fi-space-18) var(--fi-space-24);
  transition: transform var(--fi-motion-duration-fast) var(--fi-motion-ease-out);
}
.map-fab__main--pressed { transform: scale(0.94); }

.map-fab__icon { flex-shrink: 0; width: var(--fi-space-32); height: var(--fi-space-32); }

.map-fab__labels { position: relative; overflow: hidden; }

.map-fab__label {
  display: block;
  color: var(--fi-primitive-white);
  font-size: var(--fi-font-24);
  font-weight: var(--fi-weight-bold);
  line-height: var(--fi-leading-snug);
  white-space: nowrap;
}
.map-fab__label--in { animation: map-fab-label-in var(--fi-motion-duration-base) var(--fi-motion-ease-out) both; }
.map-fab__label--out {
  position: absolute;
  top: 0;
  left: 0;
  animation: map-fab-label-out var(--fi-motion-duration-base) var(--fi-motion-ease-out) both;
}

.map-fab__clear {
  display: flex;
  align-items: center;
  justify-content: center;
  min-width: var(--fi-primitive-space-64);
  min-height: var(--fi-primitive-space-64);
  margin-right: var(--fi-space-6);
  border-radius: var(--fi-radius-round);
  color: rgba(var(--fi-primitive-white-rgb), 0.84);
  font-size: var(--fi-font-24);
  transition: transform var(--fi-motion-duration-fast) var(--fi-motion-ease-out);
}
.map-fab__clear--pressed { transform: scale(0.9); }

@keyframes map-fab-label-in {
  from { opacity: 0; transform: translateY(var(--fi-space-8)); }
  to { opacity: 1; transform: translateY(0); }
}
@keyframes map-fab-label-out {
  from { opacity: 1; transform: translateY(0); }
  to { opacity: 0; transform: translateY(calc(-1 * var(--fi-space-8))); }
}

@media (prefers-reduced-motion: reduce) {
  .map-fab,
  .map-fab--visible { transform: none; transition: opacity 150ms linear; }
  .map-fab__label--in,
  .map-fab__label--out { animation-name: fi-overlay-fade-in; animation-duration: 150ms; }
  .map-fab__label--out { display: none; }
}
</style>
```

说明：组件常驻 DOM，用 class 切换进出场，保证退场动画播完；隐藏时 `pointer-events: none`，不会挡住下方内容。

- [ ] **Step 6: 验证并提交**

Run: `bun test && bun run type-check && bun run check:boundaries && bun run verify:mp-components`
Expected: 全部通过。

```bash
git pull --rebase
git add src/pages/seat-swap
git commit -m "feat(mini): restyle seat swap pool, candidate cards and dock, add map floating button

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
git push
```

---

### Task 7: 地图区改造、地图弹层与浮窗接线

**Files:**
- Modify: `src/pages/seat-swap/components/SeatSwapRegionPanel.vue`
- Modify: `src/pages/seat-swap/components/StadiumMap.vue`
- Create: `src/pages/seat-swap/components/SeatSwapMapSheet.vue`
- Modify: `src/pages/seat-swap/index.vue`
- Modify: `src/pages/seat-swap/styleAudit.test.ts`

**Interfaces:**
- Consumes: Task 4 `browse.applyMapTap`、`browse.mapInViewport`、`browse.mapFabLabel`、`browse.mapFabVisible`；Task 6 `SeatSwapMapFab`；Task 1 `navigationHeightPx`。
- Produces: `SeatSwapRegionPanel` 新增可选 prop `embedded?: boolean`；`SeatSwapMapSheet` props `{ visible: boolean; mode: 'browse' | 'filter' | 'published'; regions: TicketWatchRegion[]; badges: Record<string, number>; filterKey: string; currentKey: string; desiredKeys: string[]; hasMyRequest: boolean }`，emits `update:visible`、`region-tap`。

- [ ] **Step 1: 纳入审计（先失败）**

`AUDITED_FILES` 追加 `'components/SeatSwapRegionPanel.vue'`、`'components/StadiumMap.vue'`、`'components/SeatSwapMapSheet.vue'`。

Run: `bun test src/pages/seat-swap/styleAudit.test.ts`
Expected: FAIL。

- [ ] **Step 2: `SeatSwapRegionPanel.vue`**

- props 增加 `embedded?: boolean`；根节点 `:class="{ 'seat-map-panel--embedded': embedded }"`。
- 样式：

```css
.seat-map-panel { padding: var(--fi-space-10) var(--fi-space-24) var(--fi-space-6); background: var(--fi-color-surface-frosted); box-shadow: var(--fi-shadow-soft); }
.seat-map-panel--embedded { padding: 0; background: transparent; box-shadow: none; }
.seat-map-panel__title { color: var(--fi-color-text-strong); font-size: var(--fi-font-26); font-weight: var(--fi-weight-bold); }
.seat-map-panel__hint { color: var(--fi-color-text-muted); font-size: var(--fi-font-22); }
.stadium-wrap { margin: 0 calc(-1 * var(--fi-space-24)); }
.seat-map-panel--embedded .stadium-wrap { margin: 0; }
.legend__dot { width: var(--fi-space-16); height: var(--fi-space-16); border-radius: var(--fi-radius-round); }
.legend__dot--current { background: var(--fi-color-seat-current); }
.legend__dot--desired { background: var(--fi-color-seat-desired); }
.legend__dot--hot { background: var(--fi-primitive-ink); }
```

（先确认 `--fi-color-surface-frosted` 的值接近原 `rgba(255,255,255,0.97)`；若差异明显，改用 `rgba(var(--fi-primitive-white-rgb), 0.97)`。）

- [ ] **Step 3: `StadiumMap.vue`**

- 把票价分区配色规则（`.stadium-region--blue` … `.stadium-region--muted`，以及 `.stadium-price-legend__ticket--*`）原样包进 decorative 块：

```css
/* decorative:start 票价分区配色（业务数据色，按设计系统颜色例外保留原值） */
.stadium-region--blue { background: #336fbd; color: #fff; }
/* …其余分区规则原样… */
.stadium-price-legend__ticket--vip { background: #b90000; }
/* decorative:end */
```

两段规则不相邻时各自包一个 decorative 块。
- decorative 之外的值替换：

| 原值 | 改为 |
|---|---|
| `#8f9198`（提示文字） | `var(--fi-color-text-muted)` |
| `#17191f` | `var(--fi-color-text-strong)` |
| `#f3f2ef` | `var(--fi-color-page)` |
| `#fff`（非分区规则） | `var(--fi-primitive-white)` |
| review 态 `#d9dee7` 背景 | `var(--fi-color-divider)` |
| review 态 `#6f7683` | `var(--fi-color-text-secondary)` |
| `.stadium-region--review-current` 的 `#fff1f0` 与 `outline` 色 | `var(--fi-color-seat-current-bg)`、`rgba(var(--fi-primitive-red-rgb), 0.34)` |
| `.stadium-region--review-desired` 的 `#eef8f2` 与 `outline` 色 | `var(--fi-color-seat-desired-bg)`、`rgba(var(--fi-primitive-ink-rgb), 0.2)`（原 `rgba(29,138,85,0.45)` 绿色描边改为 seat-desired 描边需要 rgb token；若不想新增 rgb primitive，用 `var(--fi-color-seat-desired)` 实色 4rpx 描边，选其一并在交付说明写明） |
| 字号、圆角、字重、行高的字面量 | 对应 `--fi-font-*`、`--fi-radius-*`、`--fi-weight-*`、`--fi-leading-*`（差值 ≤2rpx 就近归档） |
| 阴影 `0 14rpx 28rpx rgba(18,25,20,0.32)` 等 | `var(--fi-shadow-card-strong)`；review 态弱阴影用 `var(--fi-shadow-soft)` |

`.stadium-region--muted` 若在 decorative 块内则保留；`.stadium-map--review .stadium-region` 的 `#d9dee7` 不在 decorative 内，按表替换。
- 交互逻辑与模板不变。

- [ ] **Step 4: `SeatSwapMapSheet.vue`**

```vue
<template>
  <FiBottomSheet
    :visible="visible"
    eyebrow="球场地图"
    title="点分区筛选意向"
    height="80vh"
    fit-content
    @update:visible="emit('update:visible', $event)"
  >
    <view class="map-sheet">
      <SeatSwapRegionPanel
        embedded
        :mode="mode"
        :regions="regions"
        :badges="badges"
        :filter-key="filterKey"
        :current-key="currentKey"
        :desired-keys="desiredKeys"
        :has-my-request="hasMyRequest"
        @region-tap="(key: string) => emit('region-tap', key)"
      />
    </view>
  </FiBottomSheet>
</template>

<script setup lang="ts">
import FiBottomSheet from '../../../components/FiBottomSheet.vue'
import SeatSwapRegionPanel from './SeatSwapRegionPanel.vue'
import type { TicketWatchRegion } from '../../../types/ticketWatch'

defineProps<{
  visible: boolean
  mode: 'browse' | 'filter' | 'published'
  regions: TicketWatchRegion[]
  badges: Record<string, number>
  filterKey: string
  currentKey: string
  desiredKeys: string[]
  hasMyRequest: boolean
}>()

const emit = defineEmits<{
  (e: 'update:visible', value: boolean): void
  (e: 'region-tap', key: string): void
}>()
</script>

<style scoped>
.map-sheet { padding-bottom: var(--fi-space-24); }
</style>
```

- [ ] **Step 5: `index.vue` 模板接线**

- 地图区：`.seat-map-section` 去掉 sticky，改为普通包裹节点并加锚点 id：

```vue
<view id="seat-swap-map-anchor" class="seat-map-section">
  <SeatSwapRegionPanel … 原绑定 … :has-my-request="!!currentView?.my_request" @region-tap="handleMainMapTap" />
</view>
```

样式改为 `.seat-map-section { margin: 0 calc(-1 * var(--fi-space-24)); }`（删除 `position: sticky; top: …; z-index: 5;`）。
- 在 Dock 之后、发布弹层之前加：

```vue
<SeatSwapMapFab
  v-if="!loading && !errorMessage"
  :visible="mapFabVisible"
  :label="mapFabLabel.text"
  :clearable="mapFabLabel.clearable"
  @open="openMapSheet"
  @clear="clearFilter"
/>
<SeatSwapMapSheet
  v-model:visible="mapSheetVisible"
  :mode="mainMapMode"
  :regions="regions"
  :badges="regionBadgeCounts"
  :filter-key="browsingFilterKey"
  :current-key="currentView?.my_request?.current_region_key || ''"
  :desired-keys="myDesiredKeys"
  :has-my-request="!!currentView?.my_request"
  @region-tap="handleSheetMapTap"
/>
```

（`.vue` 直接导入 `SeatSwapMapFab`、`SeatSwapMapSheet`；从 `browse` 解构 `mapFabVisible`、`mapFabLabel`。）

- [ ] **Step 6: `index.vue` 脚本：弹层、点击时序与 observer**

```ts
import { getCurrentInstance, onBeforeUnmount } from 'vue'
import { onHide, onUnload } from '@dcloudio/uni-app'

const mapSheetVisible = ref(false)
// anySheetOpen 改为包含地图弹层（同时驱动滚动锁与浮窗隐藏）
const anySheetOpen = computed(() => publishSheetVisible.value || manageSheetVisible.value || mapSheetVisible.value)

const { navigationPageStyle, navigationHeightPx } = useNavigationLayout()

const SHEET_CLOSE_MS = 220          // 与 FiBottomSheet 关闭动画一致
const REGION_HIGHLIGHT_MS = 120
const SCROLL_MS = 300
const MAP_VISIBILITY_DEBOUNCE_MS = 150

function wait(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms))
}

function openMapSheet(): void {
  mapSheetVisible.value = true
}

let mapTapBusy = false
async function handleSheetMapTap(key: string): Promise<void> {
  if (mapTapBusy) return
  const outcome = browse.applyMapTap(key, true)
  if (!outcome.closeSheet) return
  mapTapBusy = true
  try {
    await wait(REGION_HIGHLIGHT_MS)
    mapSheetVisible.value = false
    await wait(SHEET_CLOSE_MS)
    if (outcome.scrollTo) await scrollToSeatSwapRegion(outcome.scrollTo)
  } finally {
    mapTapBusy = false
  }
}
```

`scrollToSeatSwapRegion` 中 `duration: 220` 改为 `duration: SCROLL_MS`。

IntersectionObserver（每次加载成功后重新挂载）：

```ts
const pageInstance = getCurrentInstance()
let mapObserver: UniApp.IntersectionObserver | null = null
let mapVisibilityTimer: ReturnType<typeof setTimeout> | null = null

function detachMapObserver(): void {
  mapObserver?.disconnect()
  mapObserver = null
  if (mapVisibilityTimer) clearTimeout(mapVisibilityTimer)
  mapVisibilityTimer = null
}

function attachMapObserver(): void {
  detachMapObserver()
  browse.mapInViewport.value = true
  if (!pageInstance || !currentView.value?.available) return
  nextTick(() => {
    mapObserver = uni.createIntersectionObserver(pageInstance.proxy as never)
    // 扣除被品牌导航遮挡的顶部区域
    mapObserver.relativeToViewport({ top: -navigationHeightPx.value })
    mapObserver.observe('#seat-swap-map-anchor', (res) => {
      const visible = (res.intersectionRatio ?? 0) > 0
      if (mapVisibilityTimer) clearTimeout(mapVisibilityTimer)
      mapVisibilityTimer = setTimeout(() => {
        browse.mapInViewport.value = visible
      }, MAP_VISIBILITY_DEBOUNCE_MS)
    })
  })
}
```

`useSeatSwapPage` 的 `onLoaded` 改为：

```ts
onLoaded: () => {
  browse.syncExpandedRegionKeys()
  attachMapObserver()
},
```

并加：

```ts
onHide(detachMapObserver)
onUnload(detachMapObserver)
onBeforeUnmount(detachMapObserver)
```

`onLoaded` 在 `loading` 仍为 `true` 时被调用（`finally` 之前），此时地图节点尚未渲染；`attachMapObserver` 内的 `nextTick` 发生在 `loading=false` 之后的渲染周期。**实现者必须在 H5 与 mp-weixin 构建产物中各验证一次 observer 能触发**；若 mp 端一个 `nextTick` 不够，改为在 `watch(loading, (v) => { if (!v && !errorMessage.value) attachMapObserver() }, { flush: 'post' })` 中挂载，并删除 `onLoaded` 里的调用（两种方式选其一，在交付说明中写明）。`UniApp.IntersectionObserver` 类型名以 `@dcloudio/types` 实际导出为准，类型检查不过时改用 `ReturnType<typeof uni.createIntersectionObserver>`。

- [ ] **Step 7: 验证并提交**

Run: `bun test && bun run type-check && bun run check:boundaries && bun run verify:mp-components && bun run build:mp-weixin`
Expected: 全部通过；构建尾部无 `Unregistered mini-program components`。

```bash
git pull --rebase
git add src/pages/seat-swap
git commit -m "feat(mini): replace sticky seat map with a floating button and map sheet

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
git push
```

---

### Task 8: 发布弹层、管理弹层 token 化

**Files:**
- Modify: `src/pages/seat-swap/components/SeatSwapPublishSheet.vue`（只改 `<style>`）
- Modify: `src/pages/seat-swap/components/SeatSwapManageSheet.vue`（只改 `<style>`）
- Modify: `src/pages/seat-swap/styleAudit.test.ts`

- [ ] **Step 1: 纳入审计（先失败）**

`AUDITED_FILES` 追加两个弹层文件。Run: `bun test src/pages/seat-swap/styleAudit.test.ts`，Expected: FAIL。

- [ ] **Step 2: `SeatSwapPublishSheet.vue` 样式**

- `regionTagClass` 对应的分区配色规则（原第 278–285 行一组 `#336fbd`、`#46ab59`、`#6c369b`、`#f4c23a`、`#0f215e`、`#ec3b20`、`#b90000`、`#d9dee7`、`#17191f`）是票价分区配色，原样包进 `/* decorative:start … */ … /* decorative:end */`。
- 其余替换：

| 原值 | 改为 |
|---|---|
| `.steps__item--done` 的 `#eef8f2` / `rgba(29,138,85,0.3)` / `#167348` | `var(--fi-color-seat-desired-bg)` / `transparent` / `var(--fi-color-seat-desired)` |
| `#fff1f0`、`#c52018`（当前座位相关） | `var(--fi-color-seat-current-bg)`、`var(--fi-color-seat-current)` |
| `#b42318`（字段错误） | `var(--fi-color-primary-deep)` |
| `#c5c9d2`（禁用/占位） | `var(--fi-color-text-caption)` |
| `font-weight: 400`/`500`/`600`/`700`/`800` | 删除（正文）/ `var(--fi-weight-bold)` / `var(--fi-weight-extrabold)`，按元素层级：按钮与标签 bold，座位值 extrabold |
| `border-radius` 字面量 | 就近 `--fi-radius-sm|md|round` |
| `1rpx solid` 边框 | `var(--fi-border-width) solid` |

- [ ] **Step 3: `SeatSwapManageSheet.vue` 样式**

| 原值 | 改为 |
|---|---|
| `#121212`、`#15161b` | `var(--fi-color-text-strong)`、`var(--fi-primitive-ink)` |
| `#8f9198`、`#988f84`、`#6d7280` | `var(--fi-color-text-muted)`、`var(--fi-color-text-caption)`、`var(--fi-color-text-secondary)` |
| `#b42318`（危险文字） | `var(--fi-color-primary-deep)` |
| `#fff`、`#ffffff` | `var(--fi-primitive-white)` / `var(--fi-color-surface)`（文字用前者，背景用后者） |
| `#fff7f6`（危险按钮底） | `rgba(var(--fi-primitive-red-rgb), 0.06)` |
| 7 处字号字面量 | `--fi-font-*` |
| 4 处数字字重 | `var(--fi-weight-bold)` 或删除 |
| `line-height` 字面量 | `--fi-leading-*` |
| 4 处圆角字面量 | `--fi-radius-*` |

两个文件的模板与 script 一行不改。

- [ ] **Step 4: 验证并提交**

Run: `bun test && bun run type-check && bun run check:boundaries`
Expected: 全部通过。

```bash
git pull --rebase
git add src/pages/seat-swap
git commit -m "feat(mini): tokenize seat swap publish and manage sheets

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
git push
```

---

### Task 9: 全目录审计、文档与最终验证

**Files:**
- Modify: `src/pages/seat-swap/styleAudit.test.ts`
- Modify: `docs/mini-design-system.md`
- Modify: `../CHANGELOG.md`、`../FEATURES.md`

- [ ] **Step 1: 审计改为全目录**

`const AUDITED_FILES: string[] | 'ALL' = 'ALL'`

Run: `bun test src/pages/seat-swap/styleAudit.test.ts`
Expected: PASS（`src/pages/seat-swap` 下所有 `.vue`）。若失败，按报出的行修复后再跑。

- [ ] **Step 2: 残留检查**

```bash
grep -rn "SeatSwapHero\|shouldShowSeatSwapStickyMap\|readSeatSwapViewportScrollTop\|resolveSeatSwapStickyMapThreshold\|position: sticky" src/pages/seat-swap
grep -rn "seat-swap-candidate" src/pages/seat-swap
```

Expected: 两条命令都无输出。

- [ ] **Step 3: 文档**

`docs/mini-design-system.md`：
- 「Token 三层」后新增一小节「动效 token」：列出 6 个 `--fi-motion-*` 及取值来源，规定新动效优先引用，并说明只动 transform/opacity。
- 新增「换座页」小节：seat-current/desired 语义色含义；`decorative:start/end` 标注约定与 `styleAudit.test.ts` 的作用；地图浮窗与弹层的交互规则（引用 spec 路径）。
- 「弹框滚动锁约定」中参考实现的 `seat-swap/index.vue 的球队赛季弹框` 改为 `seat-swap/index.vue 的发布、管理与地图弹层`。

`CHANGELOG.md`、`FEATURES.md` 各补一条（按文件现有格式），内容要点：换座页按项目规范完成 token 化与层级统一；地图不再吸顶，划出视口后可通过浮窗快速打开地图弹层筛选；原因：与首页、洞察页、回流看板统一，扩大列表可视区域，为二期分享与留存功能准备组件结构。

- [ ] **Step 4: 最终验证**

Run（在 `football_insight_mini/`）：

```bash
bun test
bun run type-check
bun run check:boundaries
bun run verify:mp-components
bun run build:mp-weixin
```

Expected: 全部通过。记录测试总数（应为原总数 − 4 个 sticky 用例 + 新增用例）。

- [ ] **Step 5: 提交**

```bash
git pull --rebase
git add src/pages/seat-swap docs/mini-design-system.md ../CHANGELOG.md ../FEATURES.md
git commit -m "docs: record seat swap page refactor and motion tokens

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
git push
```

- [ ] **Step 6: 交付说明（写在最终回复里，不提交文件）**

逐条列出：每个组件的结构与所用 token、参照的现有样式出处；所有"二选一"决定（背景遮罩实现、observer 挂载方式、review 描边处理、`IntersectionObserver` 类型名）；被删除的 4 个 sticky 用例；字号就近归档的位置（如 25rpx→26）；未能在终端验证的视觉与动效项，提醒用户在 mp-weixin 真机体验版上检查 spec 第 11 节审查清单 1–4 条。**不要上传小程序。**

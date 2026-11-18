import { describe, expect, test } from 'bun:test'
import { readdirSync, readFileSync } from 'node:fs'
import { join, relative } from 'node:path'
import { fileURLToPath } from 'node:url'

// 全局样式与跨端架构规范机器审计（随 `bun test` 自动执行）：
// 1. 字号、内边距、间距、圆角这四类属性只引用 token，不直接写数字：
//    - 数值和通用 token 完全一致的（如 16rpx → var(--fi-space-16)）一律不许写死；
//    - 不在刻度上的旧值登记在 LEGACY_OFF_SCALE，只许减不许增；
//    - calc(... env(safe-area-inset-*) ...) 与 0 不检查。
// 2. 禁止在 .vue <style> 中直接引用第一层原始变量 `--fi-primitive-*`（存量登记在 LEGACY_PRIMITIVE_REFS，只许减不许增）。
// 3. 禁止在 .vue <style> 中使用 `[disabled]` 属性选择器（触发微信 WXSS 警告，应改用修饰 class；存量登记在 LEGACY_DISABLED_SELECTORS）。
// 4. 禁止在 src/ 运行时代码中使用浏览器 DOM / Web Storage API（document.createElement / getElementById / querySelector、new Image()、localStorage、sessionStorage）。
// 5. 单个 .vue 文件不超过 600 行（存量超标文件登记在 LEGACY_OVERSIZED_VUE，只许减不许增）。
// 6. 展示组件（src/components/**、src/pages/**/components/**）禁止直接调用 uni.showModal（确认弹窗归页面层编排）。
const SRC = fileURLToPath(new URL('../src', import.meta.url))
const TOKENS = readFileSync(join(SRC, 'styles/fi-tokens.css'), 'utf8')

const AUDITED_PROPERTY =
  /^(font-size|gap|row-gap|column-gap|padding(-(top|right|bottom|left))?|border(-(top|bottom)-(left|right))?-radius)$/
const LITERAL = /(?<![\w.-])(\d+(?:\.\d+)?)(rpx|px)\b/g
const SCALE_RADIUS = ['xs', 'sm', 'md', 'lg', 'xl', 'round']
const MAX_VUE_LINES = 600
const FORBIDDEN_DOM_API_RE =
  /\b(document\.(createElement|getElementById|querySelector|querySelectorAll|body|head)|localStorage\.|sessionStorage\.|new\s+Image\s*\()/g

/** 不在刻度上的旧值：文件 → 允许的最大数量。数量下降时请同步调小。 */
const LEGACY_OFF_SCALE = {
  'components/FiGoalMatchDialog.vue': 1, // font-size: 128rpx
  'pages/home/components/HomeSkeleton.vue': 2, // padding: 26rpx, border-radius: 20rpx
  'pages/home/components/HomeStoryPanel.vue': 2, // font-size: 60rpx, padding-left: 26rpx
  'pages/home/components/HomeTechStatsSheet.vue': 2, // font-size: 88rpx, font-size: 56rpx
  'pages/rankings/components/RankingsPlayerSheet.vue': 1, // border-radius: 32rpx
  'pages/rankings/components/RankingsPosterSheet.vue': 1, // border-radius: 32rpx
  'pages/rankings/components/RankingsStandingsLauncher.vue': 1, // border-radius: 20rpx
  'pages/rankings/components/RankingsTeamSeasonSheet.vue': 5, // border-radius: 32rpx, padding: 62rpx, padding-right: 128rpx, border-radius: 16rpx, border-radius: 16rpx
  'pages/seat-swap/components/StadiumMap.vue': 3, // gap: 7rpx, border-radius: 7rpx, padding: 11rpx
  'pages/seat-swap/index.vue': 1, // padding-bottom: 240rpx
  'pages/user/components/UserGuestChantWall.vue': 3, // font-size: 42rpx, font-size: 31rpx, font-size: 39rpx
  'pages/user/components/UserH5TestLoginPanel.vue': 4, // font-size: 25rpx, font-size: 25rpx, border-radius: 22rpx, font-size: 25rpx
  'pages/user/components/UserPasswordLoginSheet.vue': 1, // border-radius: 22rpx
  'pages/user/components/UserSkeleton.vue': 4, // border-radius: 22rpx, border-radius: 22rpx, border-radius: 22rpx, border-radius: 42rpx
  'pages/user/components/UserWechatBindSheet.vue': 1, // border-radius: 22rpx
  'pages/user/index.vue': 1, // padding: 26rpx
}

/** 存量直接引用 --fi-primitive-* 的 .vue 文件名额（只许减不许增，新文件一律为 0）。 */
const LEGACY_PRIMITIVE_REFS = {
  'components/FiResultDots.vue': 1,
  'pages/home/components/HomeFavoriteTeamSheet.vue': 3,
  'pages/home/components/HomeSheetShell.vue': 2,
  'pages/home/components/HomeStoryPanel.vue': 5,
  'pages/home/components/HomeSupportPanel.vue': 6,
  'pages/home/components/HomeTeamSeasonSheet.vue': 4,
  'pages/lineup/components/LineupFormationBar.vue': 1,
  'pages/lineup/components/LineupPitch.vue': 1,
  'pages/lineup/components/LineupPlayerDot.vue': 10,
  'pages/lineup/components/LineupPlayerSheet.vue': 4,
  'pages/lineup/components/LineupTeamPicker.vue': 1,
  'pages/lineup/index.vue': 1,
  'pages/membership-purchase/components/MembershipInviteSheet.vue': 1,
  'pages/membership-purchase/components/MembershipLevelsSheet.vue': 1,
  'pages/membership-purchase/components/MembershipTierList.vue': 2,
  'pages/membership-purchase/index.vue': 3,
  'pages/rankings/components/RankingsPlayerSheet.vue': 3,
  'pages/rankings/components/RankingsPosterSheet.vue': 3,
  'pages/rankings/components/RankingsStandingsLauncher.vue': 1,
  'pages/rankings/components/RankingsSurfacePanel.vue': 2,
  'pages/rankings/components/RankingsTeamList.vue': 1,
  'pages/rankings/components/RankingsTeamSeasonSheet.vue': 7,
  'pages/rankings/components/TeamAbilityHexagon.vue': 2,
  'pages/seat-swap/components/SeatSwapCandidateCard.vue': 19,
  'pages/seat-swap/components/SeatSwapDock.vue': 5,
  'pages/seat-swap/components/SeatSwapManageSheet.vue': 3,
  'pages/seat-swap/components/SeatSwapMatchedSheet.vue': 11,
  'pages/seat-swap/components/SeatSwapNotice.vue': 2,
  'pages/seat-swap/components/SeatSwapPoolList.vue': 1,
  'pages/seat-swap/components/SeatSwapPublishSheet.vue': 7,
  'pages/seat-swap/components/SeatSwapSeatThumb.vue': 1,
  'pages/seat-swap/components/StadiumMap.vue': 3,
  'pages/user/components/UserPasswordLoginSheet.vue': 2,
  'pages/user/components/UserWechatBindSheet.vue': 2,
  'pages/user/index.vue': 4,
  'pages/user/settings/index.vue': 3,
}

/** 存量使用 [disabled] 属性选择器的 .vue 文件名额（只许减不许增，新文件一律为 0）。 */
const LEGACY_DISABLED_SELECTORS = {
  'components/FiOfficialBindSheet.vue': 2,
  'components/FiSecondaryNav.vue': 1,
  'pages/lineup/components/LineupActionBar.vue': 2,
  'pages/lineup/components/LineupShareSheet.vue': 1,
  'pages/membership-purchase/index.vue': 1,
  'pages/seat-swap/components/SeatSwapPublishSheet.vue': 1,
  'pages/student/components/StudentNotificationEmailSheet.vue': 1,
  'pages/student/index.vue': 2,
  'pages/ticket-watch/components/TicketMatchIdSheet.vue': 1,
  'pages/user/components/UserPasswordLoginSheet.vue': 1,
  'pages/user/components/UserWechatBindSheet.vue': 1,
}

/** 存量超过 600 行的 .vue 文件行数上限（只许减不许增，新文件不得超过 600 行）。 */
const LEGACY_OVERSIZED_VUE = {
  'pages/home/index.vue': 617,
  'pages/rankings/components/RankingsTeamSeasonSheet.vue': 702,
  'pages/ticket-watch/index.vue': 721,
  'pages/user/index.vue': 687,
}

/** fi-tokens.css 里通用刻度 token 的取值：rpx 数值 → token 名。 */
export function scaleTokens(tokens) {
  const primitive = (name) => tokens.match(new RegExp(`--fi-primitive-${name}:\\s*([\\d.]+)rpx`))?.[1]
  const table = (kind, names) => {
    const out = new Map()
    for (const name of names) {
      if (!new RegExp(`--fi-${kind}-${name}:\\s*var\\(--fi-primitive-${kind}-${name}\\)`).test(tokens)) continue
      const value = primitive(`${kind}-${name}`)
      if (value !== undefined && !out.has(value)) out.set(value, `--fi-${kind}-${name}`)
    }
    return out
  }
  const numbered = (kind) => [...tokens.matchAll(new RegExp(`--fi-${kind}-(\\d+):`, 'g'))].map((m) => m[1])
  return {
    space: new Map([...table('space', numbered('space'))].filter(([value, name]) => name === `--fi-space-${value}`)),
    font: new Map([...table('font', numbered('font'))].filter(([value, name]) => name === `--fi-font-${value}`)),
    radius: table('radius', SCALE_RADIUS),
  }
}

export function extractStyleContent(source) {
  return [...source.matchAll(/<style[^>]*>([\s\S]*?)<\/style>/g)]
    .map((m) => m[1])
    .join('\n')
    .replace(/\/\*[\s\S]*?\*\//g, '')
}

/** 找出 <style> 中这四类属性里写死的数字；返回 { onScale, offScale } 两类。 */
export function findLiteralSizes(source, scale) {
  const style = extractStyleContent(source)
  const onScale = []
  const offScale = []
  for (const [, property, value] of style.matchAll(/(?<![\w-])([\w-]+)\s*:\s*([^;{}]+);/g)) {
    if (!AUDITED_PROPERTY.test(property) || value.includes('env(')) continue
    const table = property === 'font-size' ? scale.font : property.endsWith('radius') ? scale.radius : scale.space
    for (const [literal, number, unit] of value.matchAll(LITERAL)) {
      if (Number(number) === 0) continue
      const entry = `${property}: ${literal}`
      if (unit === 'rpx' && table.has(number)) onScale.push(`${entry} → var(${table.get(number)})`)
      else offScale.push(entry)
    }
  }
  return { onScale, offScale }
}

export function findStyleRuleViolations(source) {
  const style = extractStyleContent(source)
  return {
    primitiveRefs: [...style.matchAll(/--fi-primitive-[\w-]+/g)].map((m) => m[0]),
    disabledSelectors: [...style.matchAll(/\[disabled[^\]]*\]/g)].map((m) => m[0]),
  }
}

function listFiles(dir, extRe) {
  return readdirSync(dir, { withFileTypes: true }).flatMap((entry) => {
    const path = join(dir, entry.name)
    if (entry.isDirectory()) return listFiles(path, extRe)
    return extRe.test(entry.name) && !entry.name.endsWith('.test.ts') ? [path] : []
  })
}

const scale = scaleTokens(TOKENS)

describe('style token audit helpers', () => {
  test('reads the general scale from fi-tokens', () => {
    expect(scale.space.get('16')).toBe('--fi-space-16')
    expect(scale.font.get('28')).toBe('--fi-font-28')
    expect(scale.radius.get('24')).toBe('--fi-radius-md')
    expect(scale.radius.has('22')).toBe(false) // 票务专用圆角不算通用刻度
  })

  test('splits literals into on-scale and off-scale, skipping comments, zero and safe-area calc', () => {
    const source = `<template/><style>
/* padding: 16rpx; */
.a { padding: 16rpx 25rpx; font-size: 28rpx; border-radius: 22rpx; gap: 0; }
.b { padding-bottom: calc(28rpx + env(safe-area-inset-bottom)); margin: 16rpx; width: 16rpx; }
.c { padding: var(--fi-space-16); font-size: 12px; }
</style>`
    const { onScale, offScale } = findLiteralSizes(source, scale)
    expect(onScale).toEqual([
      'padding: 16rpx → var(--fi-space-16)',
      'font-size: 28rpx → var(--fi-font-28)',
    ])
    expect(offScale).toEqual(['padding: 25rpx', 'border-radius: 22rpx', 'font-size: 12px'])
  })

  test('detects --fi-primitive-* and [disabled] selectors in <style> while ignoring comments', () => {
    const source = `<template/><style>
/* .btn[disabled] { color: var(--fi-primitive-ink); } */
.btn[disabled] { background: var(--fi-primitive-white); }
</style>`
    expect(findStyleRuleViolations(source)).toEqual({
      primitiveRefs: ['--fi-primitive-white'],
      disabledSelectors: ['[disabled]'],
    })
  })
})

describe('mini-program style token & architecture audit', () => {
  const files = listFiles(SRC, /\.vue$/).map((path) => {
    const raw = readFileSync(path, 'utf8')
    return {
      name: relative(SRC, path).replaceAll('\\', '/'),
      raw,
      lineCount: raw.split('\n').length,
      ...findLiteralSizes(raw, scale),
      ...findStyleRuleViolations(raw),
    }
  })

  test('font-size / padding / gap / border-radius use tokens when one exists', () => {
    const violations = files.flatMap((file) => file.onScale.map((entry) => `${file.name} ${entry}`))
    expect(violations).toEqual([])
  })

  test('off-scale literals do not grow beyond the legacy allowance', () => {
    const grown = files
      .filter((file) => file.offScale.length > (LEGACY_OFF_SCALE[file.name] ?? 0))
      .map((file) => `${file.name}: ${file.offScale.length} > ${LEGACY_OFF_SCALE[file.name] ?? 0} (${file.offScale.join(', ')})`)
    expect(grown).toEqual([])
  })

  test('legacy allowance is tightened when off-scale literals are removed', () => {
    const stale = Object.entries(LEGACY_OFF_SCALE)
      .filter(([name, allowed]) => (files.find((file) => file.name === name)?.offScale.length ?? 0) < allowed)
      .map(([name, allowed]) => `${name}: allowed ${allowed}`)
    expect(stale).toEqual([])
  })

  test('direct --fi-primitive-* references in .vue do not grow beyond the legacy allowance', () => {
    const grown = files
      .filter((file) => file.primitiveRefs.length > (LEGACY_PRIMITIVE_REFS[file.name] ?? 0))
      .map(
        (file) =>
          `${file.name}: ${file.primitiveRefs.length} > ${LEGACY_PRIMITIVE_REFS[file.name] ?? 0} (${file.primitiveRefs.join(', ')})`,
      )
    expect(grown).toEqual([])
  })

  test('legacy --fi-primitive-* allowance is tightened when cleaned up', () => {
    const stale = Object.entries(LEGACY_PRIMITIVE_REFS)
      .filter(([name, allowed]) => (files.find((file) => file.name === name)?.primitiveRefs.length ?? 0) < allowed)
      .map(([name, allowed]) => `${name}: allowed ${allowed}`)
    expect(stale).toEqual([])
  })

  test('[disabled] attribute selectors in .vue <style> do not grow beyond the legacy allowance', () => {
    const grown = files
      .filter((file) => file.disabledSelectors.length > (LEGACY_DISABLED_SELECTORS[file.name] ?? 0))
      .map(
        (file) =>
          `${file.name}: ${file.disabledSelectors.length} > ${LEGACY_DISABLED_SELECTORS[file.name] ?? 0}`,
      )
    expect(grown).toEqual([])
  })

  test('legacy [disabled] selector allowance is tightened when cleaned up', () => {
    const stale = Object.entries(LEGACY_DISABLED_SELECTORS)
      .filter(([name, allowed]) => (files.find((file) => file.name === name)?.disabledSelectors.length ?? 0) < allowed)
      .map(([name, allowed]) => `${name}: allowed ${allowed}`)
    expect(stale).toEqual([])
  })

  test('.vue files do not exceed the 600-line split threshold (or legacy ceiling)', () => {
    const oversized = files
      .filter((file) => file.lineCount > (LEGACY_OVERSIZED_VUE[file.name] ?? MAX_VUE_LINES))
      .map(
        (file) =>
          `${file.name}: ${file.lineCount} lines > ${LEGACY_OVERSIZED_VUE[file.name] ?? MAX_VUE_LINES}`,
      )
    expect(oversized).toEqual([])
  })

  test('display components do not call uni.showModal directly', () => {
    const modalInComponents = files
      .filter((file) => file.name.startsWith('components/') || file.name.includes('/components/'))
      .filter((file) => /\buni\.showModal\b/.test(file.raw))
      .map((file) => file.name)
    expect(modalInComponents).toEqual([])
  })

  test('runtime code does not use forbidden browser DOM or Web Storage APIs', () => {
    const runtimeFiles = listFiles(SRC, /\.(vue|ts)$/)
    const domViolations = runtimeFiles.flatMap((path) => {
      const name = relative(SRC, path).replaceAll('\\', '/')
      const raw = readFileSync(path, 'utf8').replace(/\/\*[\s\S]*?\*\/|\/\/.*$/gm, '')
      return [...raw.matchAll(FORBIDDEN_DOM_API_RE)].map((m) => `${name}: ${m[0]}`)
    })
    expect(domViolations).toEqual([])
  })
})

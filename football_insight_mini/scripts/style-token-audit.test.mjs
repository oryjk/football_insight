import { describe, expect, test } from 'bun:test'
import { readdirSync, readFileSync } from 'node:fs'
import { join, relative } from 'node:path'
import { fileURLToPath } from 'node:url'

// 全局样式规范：字号、内边距、间距、圆角这四类属性只引用 token，不直接写数字。
// - 数值和通用 token 完全一致的（如 16rpx → var(--fi-space-16)）一律不许写死。
// - 不在刻度上的旧值（没有对应 token）登记在 LEGACY_OFF_SCALE，按文件计数，只许减不许增；
//   新增这类值时应先在 fi-tokens.css 里补 token，而不是往这里加名额。
// - calc(... env(safe-area-inset-*) ...) 这类安全区写法不检查，0 不检查。
const SRC = fileURLToPath(new URL('../src', import.meta.url))
const TOKENS = readFileSync(join(SRC, 'styles/fi-tokens.css'), 'utf8')

const AUDITED_PROPERTY =
  /^(font-size|gap|row-gap|column-gap|padding(-(top|right|bottom|left))?|border(-(top|bottom)-(left|right))?-radius)$/
const LITERAL = /(?<![\w.-])(\d+(?:\.\d+)?)(rpx|px)\b/g
const SCALE_RADIUS = ['xs', 'sm', 'md', 'lg', 'xl', 'round']

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

/** 找出 <style> 中这四类属性里写死的数字；返回 { onScale, offScale } 两类。 */
export function findLiteralSizes(source, scale) {
  const style = [...source.matchAll(/<style[^>]*>([\s\S]*?)<\/style>/g)]
    .map((m) => m[1])
    .join('\n')
    .replace(/\/\*[\s\S]*?\*\//g, '')
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

function listVueFiles(dir) {
  return readdirSync(dir, { withFileTypes: true }).flatMap((entry) => {
    const path = join(dir, entry.name)
    if (entry.isDirectory()) return listVueFiles(path)
    return entry.name.endsWith('.vue') ? [path] : []
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
})

describe('mini-program style token audit', () => {
  const files = listVueFiles(SRC).map((path) => ({
    name: relative(SRC, path),
    ...findLiteralSizes(readFileSync(path, 'utf8'), scale),
  }))

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
})

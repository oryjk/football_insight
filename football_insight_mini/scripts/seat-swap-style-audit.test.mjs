import { describe, expect, test } from 'bun:test'
import { readdirSync, readFileSync } from 'node:fs'
import { join } from 'node:path'
import { fileURLToPath } from 'node:url'

// 换座页样式规范的机器检查：无渐变、无散落 hex、无数字字重（decorative 块除外）。
const ROOT = fileURLToPath(new URL('../src/pages/seat-swap', import.meta.url))

// 逐任务纳入；全部组件迁移完成后改为 'ALL'。
/** @type {string[] | 'ALL'} */
const AUDITED_FILES = [
  'index.vue',
  'components/SeatSwapMatchCard.vue',
  'components/SeatSwapNotice.vue',
  'components/SeatSwapSkeleton.vue',
  'components/SeatSwapPoolList.vue',
  'components/SeatSwapCandidateCard.vue',
  'components/SeatSwapDock.vue',
  'components/SeatSwapMapFab.vue',
]

function listVueFiles(dir) {
  return readdirSync(dir, { withFileTypes: true }).flatMap((entry) => {
    const path = join(dir, entry.name)
    if (entry.isDirectory()) return listVueFiles(path)
    return entry.name.endsWith('.vue') ? [path] : []
  })
}

/** 取出 <style> 内容，并剔除 decorative:start…decorative:end 包裹的业务配色。 */
function extractAuditableStyle(source) {
  const styles = [...source.matchAll(/<style[^>]*>([\s\S]*?)<\/style>/g)].map((m) => m[1]).join('\n')
  return styles.replace(/\/\*\s*decorative:start[\s\S]*?decorative:end\s*\*\//g, '')
}

function findStyleViolations(style) {
  const violations = []
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
    test(`${file.replace(`${ROOT}/`, '')} follows token rules`, () => {
      const style = extractAuditableStyle(readFileSync(file, 'utf8'))
      expect(findStyleViolations(style)).toEqual([])
    })
  }
})

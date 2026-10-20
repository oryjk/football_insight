import { describe, expect, test } from 'bun:test'
import { readdirSync, readFileSync } from 'node:fs'
import { join } from 'node:path'
import { fileURLToPath } from 'node:url'

// 转会投票样式规范：无渐变、无散写 hex、无数字字重。
const SRC = fileURLToPath(new URL('../src', import.meta.url))
const listFiles = (dir, ext) => readdirSync(dir, { withFileTypes: true }).flatMap((entry) => {
  const path = join(dir, entry.name)
  if (entry.isDirectory()) return listFiles(path, ext)
  return ext.some((e) => entry.name.endsWith(e)) ? [path] : []
})
const FILES = [
  ...listFiles(join(SRC, 'pages/transfer'), ['.vue', '.css']),
  join(SRC, 'components/FiTransferCard.vue'),
  join(SRC, 'pages/insights/components/InsightsTransferPanel.vue'),
]
function styleOf(path) {
  const source = readFileSync(path, 'utf8')
  return path.endsWith('.css') ? source : [...source.matchAll(/<style[^>]*>([\s\S]*?)<\/style>/g)].map((m) => m[1]).join('\n')
}
function violations(style) {
  const found = []
  style.split('\n').forEach((line, index) => {
    const code = line.replace(/\/\*.*?\*\//g, '')
    if (/gradient/i.test(code)) found.push(`${index + 1}: gradient`)
    if (/#[0-9a-fA-F]{3,8}\b/.test(code)) found.push(`${index + 1}: hex`)
    if (/font-weight:\s*\d/.test(code)) found.push(`${index + 1}: numeric font-weight`)
  })
  return found
}

describe('transfer style audit', () => {
  test('covers the subpackage and shared components', () => {
    expect(FILES.length >= 6).toBe(true)
  })
  for (const file of FILES) {
    test(`${file.replace(`${SRC}/`, '')} follows token rules`, () => {
      expect(violations(styleOf(file))).toEqual([])
    })
  }
})

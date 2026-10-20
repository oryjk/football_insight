import { expect, test } from 'bun:test'
import { readdirSync, readFileSync } from 'node:fs'
import { join, relative } from 'node:path'

// 静态 SVG 必须是合法 XML：小程序 <image> 加载非法 SVG 时整张图不显示，只留下空位。
// 最容易犯的是注释里写了 "--"（XML 规定注释内不能出现连续两个减号），例如注释里引用 CSS 变量名。
const STATIC = join(import.meta.dir, '../src/static')
const listSvg = (dir: string): string[] => readdirSync(dir, { withFileTypes: true }).flatMap((entry) => {
  const path = join(dir, entry.name)
  if (entry.isDirectory()) return listSvg(path)
  return entry.name.endsWith('.svg') ? [path] : []
})

test('svg comments never contain a double hyphen', () => {
  const broken = listSvg(STATIC).flatMap((path) => {
    const comments = [...readFileSync(path, 'utf8').matchAll(/<!--([\s\S]*?)-->/g)].map((m) => m[1])
    return comments.some((body) => body.includes('--')) ? [relative(STATIC, path)] : []
  })
  expect(broken).toEqual([])
})

test('every static svg has a root svg element', () => {
  const missing = listSvg(STATIC).filter((path) => !/<svg[\s>]/.test(readFileSync(path, 'utf8'))).map((path) => relative(STATIC, path))
  expect(missing).toEqual([])
})

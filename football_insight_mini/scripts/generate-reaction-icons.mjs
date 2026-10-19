// 生成战术赞踩胶囊的状态图标：bun scripts/generate-reaction-icons.mjs
// 图形来自 Lucide thumbs-up（ISC，见 src/static/home/lucide-LICENSE.txt）；thumbs-down 即其旋转 180°。
// 小程序 image 不能按状态改色，所以每种状态各出一张 PNG，颜色与 fi-tokens.css 的对应 primitive 一致。
import { writeFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { chromium } from 'playwright'

const OUT = fileURLToPath(new URL('../src/static/lineup/', import.meta.url))
const SIZE = 72
const THUMB = '<path d="M15 5.88 14 10h5.83a2 2 0 0 1 1.92 2.56l-2.33 8A2 2 0 0 1 17.5 22H4a2 2 0 0 1-2-2v-8a2 2 0 0 1 2-2h2.76a2 2 0 0 0 1.79-1.11L12 2a3.13 3.13 0 0 1 3 3.88Z"/><path d="M7 10v12"/>'
const COLORS = {
  idle: '#6d7280', // --fi-primitive-grey-text-deep（--fi-color-text-secondary）
  like: '#dc2626', // --fi-primitive-red（--fi-color-primary）
  dislike: '#121212', // --fi-primitive-ink-deep（--fi-color-text-strong）
}
const ICONS = {
  'reaction-like-idle': [COLORS.idle, false],
  'reaction-like-active': [COLORS.like, false],
  'reaction-dislike-idle': [COLORS.idle, true],
  'reaction-dislike-active': [COLORS.dislike, true],
}

// 本机浏览器版本与 playwright 不一致时，用 CHROMIUM_PATH 指向已安装的 chrome。
const browser = await chromium.launch({ headless: true, executablePath: process.env.CHROMIUM_PATH || undefined })
const page = await browser.newPage({ viewport: { width: SIZE, height: SIZE }, deviceScaleFactor: 1 })
for (const [name, [stroke, down]] of Object.entries(ICONS)) {
  const svg = `<svg xmlns="http://www.w3.org/2000/svg" width="${SIZE}" height="${SIZE}" viewBox="0 0 24 24" fill="none" stroke="${stroke}" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><g${down ? ' transform="rotate(180 12 12)"' : ''}>${THUMB}</g></svg>`
  await page.setContent(`<!doctype html><html><body style="margin:0;background:transparent">${svg}</body></html>`)
  writeFileSync(`${OUT}${name}.png`, await page.locator('svg').screenshot({ omitBackground: true }))
}
await browser.close()
console.log(`wrote ${Object.keys(ICONS).length} icons to ${OUT}`)

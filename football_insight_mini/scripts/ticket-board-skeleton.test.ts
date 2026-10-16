import { expect, test } from 'bun:test'
import { readFileSync } from 'node:fs'
import { createRequire } from 'node:module'
import { compileScript, parse } from '@vue/compiler-sfc'
import { createSSRApp } from 'vue'
import { renderToString } from '@vue/server-renderer'
import ts from 'typescript'

const filename = new URL('../src/pages/ticket-watch/components/TicketBoardSkeleton.vue', import.meta.url)
const descriptor = parse(readFileSync(filename, 'utf8'), { filename: filename.pathname }).descriptor
const compiled = compileScript(descriptor, {
  id: 'ticket-board-skeleton-test',
  inlineTemplate: true,
  templateOptions: { compilerOptions: { isCustomElement: tag => tag === 'view' } },
})
const js = ts.transpileModule(compiled.content, {
  compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2020 },
}).outputText
const componentExports: any = {}
new Function('require', 'exports', js)(createRequire(filename), componentExports)

test('current skeleton retains three tab placeholders by default', async () => {
  const html = await renderToString(createSSRApp(componentExports.default, { mode: 'current' }))
  expect(html.match(/class="fi-skeleton-block ticket-skeleton__tab"/g)?.length ?? 0).toBe(3)
})

test('current skeleton can hide tab placeholders while preserving the loading match card', async () => {
  const html = await renderToString(createSSRApp(componentExports.default, { mode: 'current', showTabs: false }))
  expect(html.match(/class="fi-skeleton-block ticket-skeleton__tab"/g)?.length ?? 0).toBe(0)
  expect(html.match(/class="fi-skeleton-block ticket-skeleton__status"/g)?.length ?? 0).toBe(1)
  expect(html.match(/class="fi-skeleton-block ticket-skeleton__button"/g)?.length ?? 0).toBe(3)
})

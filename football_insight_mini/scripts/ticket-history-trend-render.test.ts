import { expect, test } from 'bun:test'
import { readFileSync } from 'node:fs'
import { createRequire } from 'node:module'
import { compileScript, parse } from '@vue/compiler-sfc'
import { createSSRApp, h } from 'vue'
import { renderToString } from '@vue/server-renderer'
import ts from 'typescript'

const filename = new URL('../src/pages/ticket-watch/components/TicketHistoryTrend.vue', import.meta.url)
const descriptor = parse(readFileSync(filename, 'utf8'), { filename: filename.pathname }).descriptor
const compiled = compileScript(descriptor, {
  id: 'ticket-history-trend-test', inlineTemplate: true,
  templateOptions: { compilerOptions: { isCustomElement: tag => ['view', 'text', 'scroll-view'].includes(tag) } },
})
const js = ts.transpileModule(compiled.content, {
  compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2020 },
}).outputText
const componentExports: any = {}
new Function('require', 'exports', js)(createRequire(filename), componentExports)
const emptyTrend = { points: [], maxTotal: 0, averageTotal: 0, latestTotal: 0, loadedCount: 0, totalCount: 0 }

async function renderTrend(props: Record<string, unknown>) {
  const app = createSSRApp(componentExports.default, props)
  // The uni compiler resolves scroll-view as a native component; forward its slot in SSR.
  app.component('scroll-view', {
    setup(_, { slots }) { return () => h('scroll-view', slots.default?.()) },
  })
  return renderToString(app)
}

test('empty history statistics explain that no games have reflux data', async () => {
  const html = await renderTrend({ loading: false, trend: emptyTrend })
  expect(html).toContain('暂无有回流数据的历史比赛。')
  expect(html).not.toContain('正在加载历史回流数据')
})

test('pending history statistics retain loading feedback while empty rows are hidden', async () => {
  const html = await renderTrend({ loading: true, trend: emptyTrend })
  expect(html).toContain('正在加载历史回流数据…')
  expect(html).not.toContain('暂无有回流数据的历史比赛')
})

test('nonempty history statistics keep the match list and suppress the empty message', async () => {
  const trend = { ...emptyTrend, loadedCount: 1, totalCount: 1, points: [{
    match_id: 1, label: '10-16', teams: '浙江队', total_occurrences: 8,
    bar_percent: 100, is_selected: false, is_loaded: true, tone: 'high',
  }] }
  const html = await renderTrend({ loading: false, trend })
  expect(html).toContain('浙江队')
  expect(html).not.toContain('暂无有回流数据的历史比赛')
})

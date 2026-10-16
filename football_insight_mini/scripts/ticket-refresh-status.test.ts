import { afterEach, expect, spyOn, test } from 'bun:test'
import { readFileSync } from 'node:fs'
import { createRequire } from 'node:module'
import { compileScript, parse } from '@vue/compiler-sfc'
import { createRenderer, h, nextTick, reactive } from 'vue'
import ts from 'typescript'

const filename = new URL('../src/pages/ticket-watch/components/TicketRefreshStatus.vue', import.meta.url)
const descriptor = parse(readFileSync(filename, 'utf8'), { filename: filename.pathname }).descriptor
const compiled = compileScript(descriptor, {
  id: 'ticket-refresh-status-test',
  inlineTemplate: true,
  templateOptions: { compilerOptions: { isCustomElement: tag => tag === 'view' || tag === 'text' } },
})
const js = ts.transpileModule(compiled.content, {
  compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2020 },
}).outputText
const componentExports: any = {}
new Function('require', 'exports', js)(createRequire(filename), componentExports)

type HostNode = { text: string; children: HostNode[]; parent?: HostNode }
const node = (text = ''): HostNode => ({ text, children: [] })
const renderer = createRenderer<HostNode, HostNode>({
  createElement: () => node(), createText: node, createComment: () => node(),
  setText: (target, text) => { target.text = text },
  setElementText: (target, text) => { target.text = text; target.children = [] },
  parentNode: target => target.parent ?? null,
  nextSibling: () => null,
  patchProp() {},
  insert(target, parent, anchor) {
    target.parent = parent
    const index = anchor ? parent.children.indexOf(anchor) : -1
    if (index < 0) parent.children.push(target)
    else parent.children.splice(index, 0, target)
  },
  remove(target) {
    const siblings = target.parent?.children
    if (siblings) siblings.splice(siblings.indexOf(target), 1)
  },
})
const readText = (target: HostNode): string => target.text + target.children.map(readText).join('')
const cleanups: Array<() => void> = []
afterEach(() => { while (cleanups.length) cleanups.pop()!() })

function mountStatus(overrides = {}) {
  let now = 1_000_000
  let nextId = 0
  const timers = new Map<number, { callback: () => void; interval: number; at: number }>()
  const nowSpy = spyOn(Date, 'now').mockImplementation(() => now)
  const intervalSpy = spyOn(globalThis, 'setInterval').mockImplementation(((callback: () => void, interval: number) => {
    const id = ++nextId
    timers.set(id, { callback, interval, at: now + interval })
    return id
  }) as any)
  const clearSpy = spyOn(globalThis, 'clearInterval').mockImplementation(((id: number) => { timers.delete(id) }) as any)
  cleanups.push(() => { nowSpy.mockRestore(); intervalSpy.mockRestore(); clearSpy.mockRestore() })
  const props = reactive({ monitoring: true, refreshing: false, pollFailing: false, intervalSeconds: 15, lastUpdatedAtMs: now - 5_000 as number | null, ticking: true, ...overrides })
  const root = node()
  const app = renderer.createApp({ render: () => h(componentExports.default, props) })
  app.mount(root)
  cleanups.push(() => app.unmount())
  return {
    props, timers, text: () => readText(root), unmount: () => app.unmount(),
    async advance(milliseconds: number) {
      const end = now + milliseconds
      while (true) {
        const due = [...timers.values()].filter(timer => timer.at <= end).sort((a, b) => a.at - b.at)[0]
        if (!due) break
        now = due.at
        due.at += due.interval
        due.callback()
        await nextTick()
      }
      now = end
      await nextTick()
    },
  }
}

test('monitoring freshness updates each second and switches to a slower idle clock', async () => {
  const status = mountStatus()
  expect(status.text()).toContain('5 秒前 更新')
  await status.advance(1_000)
  expect(status.text()).toContain('6 秒前 更新')
  status.props.monitoring = false
  await nextTick()
  await status.advance(14_000)
  expect(status.text()).toContain('6 秒前 更新')
  await status.advance(1_000)
  expect(status.text()).toContain('21 秒前 更新')
})

test('hidden status stops its clock and becomes current immediately when shown', async () => {
  const status = mountStatus({ ticking: false })
  expect(status.text()).toContain('5 秒前 更新')
  expect(status.timers.size).toBe(0)
  await status.advance(10_000)
  expect(status.text()).toContain('5 秒前 更新')
  status.props.ticking = true
  await nextTick()
  expect(status.text()).toContain('15 秒前 更新')
  status.props.ticking = false
  await nextTick()
  expect(status.timers.size).toBe(0)
  await status.advance(10_000)
  expect(status.text()).toContain('15 秒前 更新')
})

test('unmount releases the active clock', () => {
  const status = mountStatus()
  expect(status.timers.size).toBe(1)
  status.unmount()
  expect(status.timers.size).toBe(0)
})

test('status prioritizes monitoring and in-flight retry while preserving last successful freshness', async () => {
  const status = mountStatus({ monitoring: false, refreshing: true, pollFailing: true })
  expect(status.text()).toContain('未开启监控 · 数据不会自动刷新')
  status.props.monitoring = true
  await nextTick()
  expect(status.text()).toContain('刷新中…')
  status.props.refreshing = false
  await nextTick()
  expect(status.text()).toContain('刷新失败，将自动重试')
  expect(status.text()).toContain('上次成功 5 秒前')
  status.props.pollFailing = false
  await nextTick()
  expect(status.text()).toContain('监控中 · 每 15 秒 刷新')
  status.props.lastUpdatedAtMs = null
  await nextTick()
  expect(status.text()).toContain('尚未更新')
  expect(status.text()).not.toContain('尚未更新 更新')
  status.props.lastUpdatedAtMs = Number.NaN
  await nextTick()
  expect(status.text()).toContain('尚未更新')
  expect(status.text()).not.toContain('尚未更新 更新')
  status.props.intervalSeconds = 0
  await nextTick()
  expect(status.text()).toContain('监控中')
  expect(status.text()).not.toContain('每')
})

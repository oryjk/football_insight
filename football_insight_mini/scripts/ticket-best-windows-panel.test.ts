import { afterEach, expect, spyOn, test } from 'bun:test'
import { readFileSync } from 'node:fs'
import { createRequire } from 'node:module'
import { resolve, dirname } from 'node:path'
import { compileScript, parse } from '@vue/compiler-sfc'
import { createRenderer, h, nextTick, reactive } from 'vue'
import ts from 'typescript'

function loadComponent(filename: string): any {
 const descriptor = parse(readFileSync(filename, 'utf8'), { filename }).descriptor
 const compiled = compileScript(descriptor, { id: 'best-windows-test', inlineTemplate: true, templateOptions: { compilerOptions: { isCustomElement: tag => ['view', 'text', 'button'].includes(tag) } } })
 const js = ts.transpileModule(compiled.content, { compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2020 } }).outputText
 const exports: any = {}
 const require = createRequire(filename)
 new Function('require', 'exports', js)((specifier: string) => specifier.endsWith('.vue') ? { default: loadComponent(resolve(dirname(filename), specifier)) } : require(specifier), exports)
 return exports.default
}
const Panel = loadComponent(new URL('../src/pages/ticket-watch/components/TicketBestWindowsPanel.vue', import.meta.url).pathname)
type Node = { text: string; props: Record<string, any>; children: Node[]; parent?: Node }
const node = (text = ''): Node => ({ text, props: {}, children: [] })
const renderer = createRenderer<Node, Node>({
 createElement: () => node(), createText: node, createComment: () => node(),
 setText: (n, text) => { n.text = text }, setElementText: (n, text) => { n.text = text; n.children = [] },
 parentNode: n => n.parent ?? null, nextSibling: n => { const siblings = n.parent?.children; return siblings?.[(siblings.indexOf(n)) + 1] ?? null },
 patchProp: (n, key, _, value) => { n.props[key] = value },
 insert(n, parent, anchor) { const old = n.parent?.children; if (old) { const i = old.indexOf(n); if (i >= 0) old.splice(i, 1) }; n.parent = parent; const i = anchor ? parent.children.indexOf(anchor) : -1; if (i < 0) parent.children.push(n); else parent.children.splice(i, 0, n) },
 remove(n) { const siblings = n.parent?.children; if (siblings) siblings.splice(siblings.indexOf(n), 1) },
})
const all = (n: Node): Node[] => [n, ...n.children.flatMap(all)]
const text = (n: Node): string => n.text + n.children.map(text).join('')
const byClass = (n: Node, name: string) => all(n).filter(n => String(n.props.class).split(' ').includes(name))
const cleanups: Array<() => void> = []
afterEach(() => { while (cleanups.length) cleanups.pop()!() })
const full = { status: 'ok', access: 'full', sample_size: 5, generated_at: null,
 top_windows: [12, 11, 14].map(hour => ({ start_hour: hour, end_hour: hour + 1, avg_events: 10, hit_matches: 3 })),
 hours: Array.from({ length: 24 }, (_, hour) => ({ hour, avg_events: hour + 1, hit_matches: hour === 12 ? 4 : 2, share: hour === 12 ? 0.153 : 0.02 })) }
function mount(overrides: Record<string, unknown> = {}) {
 const props = reactive({ data: full, loading: false, visible: true, onUpgrade: () => {}, ...overrides })
 const root = node(); const app = renderer.createApp({ render: () => h(Panel, props) }); app.mount(root)
 cleanups.push(() => app.unmount())
 return { props, root, text: () => text(root), bars: () => byClass(root, 'best-windows__bar'), unmount: () => app.unmount() }
}
test('full renders exactly 24 bars, top three and one current marker with exact conclusion', async () => {
 const panel = mount()
 expect(panel.bars()).toHaveLength(24)
 expect(byClass(panel.root, 'best-windows__bar--peak')).toHaveLength(3)
 expect(byClass(panel.root, 'best-windows__now-line')).toHaveLength(1)
 expect(panel.text()).not.toContain('现在')
 expect(panel.text()).toContain('近 5 场主场的补票分布')
 expect(panel.text()).toContain('柱子越高，这个钟点补票越频繁')
 expect(panel.text()).toContain('补票最集中在 12:00–13:00、11:00–12:00、14:00–15:00')
 // Chart spans x 0..240, so each hour column is 10 wide.
 const original = (globalThis as any).uni
 ;(globalThis as any).uni = { createSelectorQuery: () => ({ in: () => ({ select: () => ({ boundingClientRect: () => ({ exec: (cb: (rows: unknown[]) => void) => cb([{ left: 0, width: 240 }]) }) }) }) }) }
 cleanups.push(() => { (globalThis as any).uni = original })
 const chart = byClass(panel.root, 'best-windows__chart')[0]
 const touch = (type: string, x: number) => chart.props[type === 'touchstart' ? 'onTouchstart' : 'onTouchmove']({ type, touches: [{ clientX: x }] })
 touch('touchstart', 125); await nextTick()
 expect(panel.text()).toContain('12 点：近 5 场里有 4 场出现补票，占全天补票的 15.3%')
 expect(byClass(panel.root, 'best-windows__bar--selected')).toHaveLength(1)
 touch('touchmove', 145); await nextTick()
 expect(panel.text()).not.toContain('12 点：')
 expect(panel.text()).toContain('14 点：近 5 场里有 2 场出现补票，占全天补票的 2%')
 touch('touchmove', 999); await nextTick()
 expect(panel.text()).toContain('23 点：')
})
test('preview has no distribution or secondary peaks and emits upgrade', () => {
 let taps = 0, invites = 0
 const panel = mount({ data: { ...full, access: 'preview', hours: [], top_windows: [{ ...full.top_windows[0], start_hour: null, end_hour: null }] }, onUpgrade: () => { taps++ }, onInvite: () => { invites++ }, inviteHint: '累计邀请 3 位好友注册，就能免费升到 V6。' })
 expect(panel.bars()).toHaveLength(0)
 expect(panel.text()).toContain('最容易出票的时段：**:**–**:**')
 expect(panel.text()).not.toContain('12:00')
 expect(panel.text()).toContain('V6 及以上会员可查看全天分布和前三个高峰时段')
 expect(panel.text()).toContain('累计邀请 3 位好友注册')
 expect(panel.text()).not.toContain('柱子越高')
 expect(panel.text()).not.toContain('11:00–12:00')
 byClass(panel.root, 'upgrade-actions__button--purchase')[0].props.onTap()
 expect(taps).toBe(1)
 byClass(panel.root, 'upgrade-actions__button--invite')[0].props.onTap()
 expect(invites).toBe(1)
})
test('loading uses global skeleton and absent data renders no panel', () => {
 const panel = mount({ data: null, loading: true })
 expect(byClass(panel.root, 'fi-skeleton-panel')).toHaveLength(1)
 expect(panel.bars()).toHaveLength(0)
 const empty = mount({ data: null })
 expect(empty.text()).toBe('')
})
test('minute clock runs only while full panel is visible and cleans up on hide and unmount', async () => {
 const timers = new Map<number, () => void>(); let id = 0
 const interval = spyOn(globalThis, 'setInterval').mockImplementation(((fn: () => void, ms: number) => { expect(ms).toBe(60000); timers.set(++id, fn); return id }) as any)
 const clear = spyOn(globalThis, 'clearInterval').mockImplementation(((key: number) => timers.delete(key)) as any)
 cleanups.push(() => { interval.mockRestore(); clear.mockRestore() })
 const panel = mount()
 expect(timers.size).toBe(1)
 panel.props.visible = false; await nextTick(); expect(timers.size).toBe(0)
 panel.props.visible = true; await nextTick(); expect(timers.size).toBe(1)
 panel.props.data = { ...full, access: 'preview' }; await nextTick(); expect(timers.size).toBe(0)
 panel.props.data = full; await nextTick(); expect(timers.size).toBe(1)
 panel.unmount(); expect(timers.size).toBe(0)
})

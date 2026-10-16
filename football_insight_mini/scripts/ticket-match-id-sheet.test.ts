import { afterEach, expect, test } from 'bun:test'
import { readFileSync } from 'node:fs'
import { createRequire } from 'node:module'
import { compileScript, parse } from '@vue/compiler-sfc'
import { createRenderer, h, nextTick, reactive } from 'vue'
import ts from 'typescript'

const filename = new URL('../src/pages/ticket-watch/components/TicketMatchIdSheet.vue', import.meta.url)
const descriptor = parse(readFileSync(filename, 'utf8'), { filename: filename.pathname }).descriptor
const compiled = compileScript(descriptor, {
  id: 'ticket-match-id-sheet-test', inlineTemplate: true,
  templateOptions: { compilerOptions: { isCustomElement: tag => ['view', 'text', 'button'].includes(tag) } },
})
const js = ts.transpileModule(compiled.content, {
  compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2020 },
}).outputText
const componentExports: any = {}
new Function('require', 'exports', js)(createRequire(filename), componentExports)
type HostNode = { text: string; children: HostNode[]; props: Record<string, any>; parent?: HostNode }
const node = (text = ''): HostNode => ({ text, children: [], props: {} })
const renderer = createRenderer<HostNode, HostNode>({
  createElement: () => node(), createText: node, createComment: () => node(),
  setText: (target, text) => { target.text = text },
  setElementText: (target, text) => { target.text = text; target.children = [] },
  parentNode: target => target.parent ?? null, nextSibling: () => null,
  patchProp: (target, key, _, value) => { target.props[key] = value },
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
const flatten = (target: HostNode): HostNode[] => [target, ...target.children.flatMap(flatten)]
const cleanups: Array<() => void> = []
afterEach(() => { while (cleanups.length) cleanups.pop()!() })

function mountSheet(overrides = {}) {
  const originalUni = (globalThis as any).uni
  const loadingCalls: unknown[] = []
  let hideLoadingCalls = 0
  ;(globalThis as any).uni = {
    ...originalUni,
    showLoading: (options: unknown) => { loadingCalls.push(options) },
    hideLoading: () => { hideLoadingCalls += 1 },
  }
  cleanups.push(() => { (globalThis as any).uni = originalUni })
  const props = reactive({ visible: true, matchId: '78' as string | null, matchLabel: '成都蓉城 VS 浙江队', state: 'loading', via: null, ...overrides })
  const root = node()
  const app = renderer.createApp({ render: () => h(componentExports.default, props) })
  app.mount(root)
  cleanups.push(() => app.unmount())
  return { props, loadingCalls, hiddenLoading: () => hideLoadingCalls, text: () => readText(root), button: (label: string) => flatten(root).find(item => item.props.onTap && readText(item).trim() === label) }
}

test('loading entitlement uses native progress and renders the sheet only after resolution', async () => {
  const sheet = mountSheet({ visible: false })
  sheet.props.visible = true
  await nextTick()
  expect(sheet.text()).toBe('')
  expect(sheet.loadingCalls).toEqual([{ title: '查询中', mask: true }])
  sheet.props.state = 'unlocked'
  await nextTick()
  expect(sheet.text()).toContain('78')
  expect(sheet.hiddenLoading()).toBe(1)
})

test('the copy action preserves the full business ID rather than a schedule key', () => {
  const originalUni = (globalThis as any).uni
  const copied: string[] = []
  ;(globalThis as any).uni = { setClipboardData: ({ data }: { data: string }) => copied.push(data) }
  cleanups.push(() => { (globalThis as any).uni = originalUni })
  const sheet = mountSheet({ state: 'unlocked', matchId: '08', via: 'membership', watermark: '测试昵称' })
  expect(sheet.text()).toContain('测试昵称')
  sheet.button('复制')!.props.onTap()
  expect(copied).toEqual(['08'])
})

test('a missing business ID is explained and cannot be copied as an empty value', () => {
  const sheet = mountSheet({ state: 'unlocked', matchId: null })
  expect(sheet.text()).toContain('业务 ID 暂不可用')
  expect(sheet.button('复制')?.props.disabled).toBe(true)
})

test('locked and paying states retain upgrade and single-match purchase actions', async () => {
  const sheet = mountSheet({ state: 'locked' })
  expect(sheet.text()).toContain('¥5 解锁本场')
  expect(sheet.text()).toContain('升级到 V6')
  sheet.props.state = 'paying'
  await nextTick()
  expect(sheet.button('处理中...')?.props.disabled).toBe(true)
  expect(sheet.button('升级到 V6')?.props.disabled).toBe(true)
})

import { afterEach, expect, spyOn, test } from 'bun:test'
import { readFileSync } from 'node:fs'
import { createRequire } from 'node:module'
import { compileScript, parse } from '@vue/compiler-sfc'
import { createRenderer, h, nextTick, reactive } from 'vue'
import ts from 'typescript'

function loadComponent(filename: string): any {
 const descriptor = parse(readFileSync(filename, 'utf8'), { filename }).descriptor
 const compiled = compileScript(descriptor, { id: 'fi-collapse-test', inlineTemplate: true, templateOptions: { compilerOptions: { isCustomElement: tag => ['view', 'text'].includes(tag) } } })
 const js = ts.transpileModule(compiled.content, { compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2020 } }).outputText
 const exports: any = {}
 new Function('require', 'exports', js)(createRequire(filename), exports)
 return exports.default
}
const Collapse = loadComponent(new URL('../src/components/FiCollapse.vue', import.meta.url).pathname)
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
const byClass = (n: Node, name: string) => all(n).filter(n => String(n.props.class).split(' ').includes(name))
const cleanups: Array<() => void> = []
afterEach(() => { while (cleanups.length) cleanups.pop()!() })

function setup(open: boolean) {
 const timers: Array<{ fn: () => void; ms: number }> = []
 const spy = spyOn(globalThis, 'setTimeout').mockImplementation(((fn: () => void, ms: number) => { timers.push({ fn, ms }); return timers.length }) as any)
 const clear = spyOn(globalThis, 'clearTimeout').mockImplementation(((id: number) => { if (timers[id - 1]) timers[id - 1] = { fn: () => {}, ms: -1 } }) as any)
 const props = reactive({ open })
 const root = node()
 const app = renderer.createApp({ render: () => h(Collapse, props, { default: () => h('view', { class: 'body' }) }) })
 app.mount(root)
 cleanups.push(() => { app.unmount(); spy.mockRestore(); clear.mockRestore() })
 const flush = async () => { for (let i = 0; i < 6; i++) await Promise.resolve(); await nextTick() }
 const runTimers = async () => { const due = timers.splice(0); due.forEach(t => t.fn()); await flush() }
 const wrapper = () => byClass(root, 'fi-collapse')[0]
 const classes = () => String(wrapper()?.props.class ?? '')
 return { props, flush, runTimers, classes, wrapper, body: () => byClass(root, 'body').length, timers }
}

test('starts open without animation and starts closed without mounting the content', async () => {
 const open = setup(true)
 await open.flush()
 expect(open.classes()).toContain('fi-collapse--open')
 expect(open.body()).toBe(1)
 const closed = setup(false)
 await closed.flush()
 expect(closed.wrapper()).toBeUndefined()
 expect(closed.body()).toBe(0)
})

test('collapse plays the closing animation, then unmounts the content after the duration', async () => {
 const view = setup(true)
 await view.flush()
 view.props.open = false
 await view.flush()
 expect(view.classes()).toContain('fi-collapse--closing')
 expect(view.body()).toBe(1)
 expect(view.timers.at(-1)?.ms).toBe(280)
 await view.runTimers()
 expect(view.body()).toBe(0)
})

test('expand mounts the content, plays the opening animation, then settles', async () => {
 const view = setup(false)
 await view.flush()
 view.props.open = true
 await view.flush()
 expect(view.body()).toBe(1)
 expect(view.classes()).toContain('fi-collapse--opening')
 await view.runTimers()
 expect(view.classes()).toContain('fi-collapse--open')
 expect(view.classes()).not.toContain('opening')
})

test('a quick re-toggle cancels the stale unmount', async () => {
 const view = setup(true)
 await view.flush()
 view.props.open = false
 await view.flush()
 view.props.open = true // reopen before the collapse timer fires
 await view.flush()
 await view.runTimers()
 expect(view.body()).toBe(1)
 expect(view.classes()).toContain('fi-collapse--open')
})

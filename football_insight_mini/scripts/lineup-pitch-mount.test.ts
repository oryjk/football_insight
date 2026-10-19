import { afterEach, expect, test } from 'bun:test'
import { readFileSync } from 'node:fs'
import { createRequire } from 'node:module'
import { compileScript, parse } from '@vue/compiler-sfc'
import { createRenderer, defineComponent, h, nextTick, reactive } from 'vue'
import ts from 'typescript'
import { boxPositionPx } from '../src/pages/lineup/dragGeometry'
import { getFormation } from '../src/pages/lineup/formations'
import type { PitchPlayer } from '../src/pages/lineup/lineupViewModel'

const filename = new URL('../src/pages/lineup/components/LineupPitch.vue', import.meta.url)
const isCustomElement = (tag: string) => ['view', 'movable-area', 'movable-view'].includes(tag)
const descriptor = parse(readFileSync(filename, 'utf8'), { filename: filename.pathname, templateParseOptions: { isCustomElement } }).descriptor
const compiled = compileScript(descriptor, {
  id: 'pitch-mount-test', inlineTemplate: true,
  templateOptions: { compilerOptions: { isCustomElement } },
})
const js = ts.transpileModule(compiled.content, { compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2020 } }).outputText
const output: { default?: any } = {}
const require = createRequire(filename)
new Function('require', 'exports', js)((id: string) => id.endsWith('LineupPlayerDot.vue')
  ? { default: defineComponent({ setup: () => () => h('view') }) }
  : require(id), output)

type Node = { tag: string; props: Record<string, any>; children: Node[]; parent?: Node }
const node = (tag = ''): Node => ({ tag, props: {}, children: [] })
const renderer = createRenderer<Node, Node>({
  createElement: node, createText: () => node(), createComment: () => node(),
  setText() {}, setElementText(n) { n.children = [] },
  parentNode: n => n.parent ?? null,
  nextSibling: n => { const siblings = n.parent?.children; return siblings?.[(siblings.indexOf(n)) + 1] ?? null },
  patchProp: (n, key, _, value) => { n.props[key] = value },
  insert(n, parent, anchor) {
    if (n.parent) n.parent.children.splice(n.parent.children.indexOf(n), 1)
    n.parent = parent
    const i = anchor ? parent.children.indexOf(anchor) : -1
    if (i < 0) parent.children.push(n); else parent.children.splice(i, 0, n)
  },
  remove(n) { if (n.parent) n.parent.children.splice(n.parent.children.indexOf(n), 1) },
  insertStaticContent(_content, parent, anchor) {
    const n = node('static')
    n.parent = parent
    const i = anchor ? parent.children.indexOf(anchor) : -1
    if (i < 0) parent.children.push(n); else parent.children.splice(i, 0, n)
    return [n, n]
  },
})
const all = (n: Node): Node[] => [n, ...n.children.flatMap(all)]
const players = (): PitchPlayer[] => getFormation('4-3-3').map(spot => ({ ...spot, player: null }))
const cleanups: Array<() => void> = []
afterEach(() => { while (cleanups.length) cleanups.pop()!() })
const flush = async () => { for (let i = 0; i < 6; i++) await nextTick() }

function setup(readonly = false) {
  const previous = globalThis.uni
  const requests: Array<{ selector: string; reply: (rect: unknown) => void }> = []
  let resize: (() => void) | undefined
  globalThis.uni = {
    getWindowInfo: () => ({ windowWidth: 375 }),
    onWindowResize: (fn: () => void) => { resize = fn },
    offWindowResize: () => { resize = undefined },
    createSelectorQuery() {
      let selector = '', callback: (rect: unknown) => void
      const query = {
        in: () => query,
        select(value: string) { selector = value; return query },
        boundingClientRect(fn: (rect: unknown) => void) { callback = fn; return query },
        exec() { requests.push({ selector, reply: callback }) },
      }
      return query
    },
  } as unknown as UniApp.Uni
  const props = reactive({ players: players(), readonly, fitContainer: true })
  const root = node()
  const app = renderer.createApp({ render: () => h(output.default, props) })
  app.mount(root)
  let mounted = true
  const unmount = () => { if (mounted) { app.unmount(); mounted = false } }
  cleanups.push(() => { unmount(); globalThis.uni = previous })
  return { root, requests, props, unmount, resize: () => resize?.(), movers: () => all(root).filter(n => n.tag === 'movable-view') }
}

test('first entry mounts the native area before the eleven 4-3-3 movers', async () => {
  const pitch = setup()
  await flush()
  expect(pitch.movers()).toHaveLength(0)
  const field = pitch.requests.shift()!
  expect(field.selector).toBe('#lineup-pitch-field')
  field.reply({ width: 351, height: 468 })
  await flush()
  expect(all(pitch.root).filter(n => n.tag === 'movable-area')).toHaveLength(1)
  // Parent data is ready, but WeChat has not laid out the native area yet.
  expect(pitch.movers()).toHaveLength(0)
  const area = pitch.requests.shift()!
  expect(area.selector).toBe('.pitch__area')
  area.reply({ width: 351, height: 468 })
  await flush()
  expect(pitch.movers()).toHaveLength(11)
  expect(pitch.movers().map(n => ({ x: n.props.x, y: n.props.y }))).toEqual(
    pitch.props.players.map(p => boxPositionPx(p, { width: 351, height: 468 }, 375)),
  )
  expect(new Set(pitch.movers().map(n => `${n.props.x}:${n.props.y}`)).size).toBe(11)
})

test('late layout callbacks cannot mount movers after the pitch has unmounted', async () => {
  const pitch = setup()
  await flush()
  pitch.requests.shift()!.reply({ width: 351, height: 468 })
  await flush()
  const area = pitch.requests.shift()
  expect(area).toBeDefined()
  pitch.unmount()
  area!.reply({ width: 351, height: 468 })
  await flush()
  expect(pitch.movers()).toHaveLength(0)
  expect(pitch.requests).toHaveLength(0)
})

test('an area still at zero or default size keeps players hidden until its layout catches up', async () => {
  const pitch = setup()
  await flush()
  pitch.requests.shift()!.reply({ width: 351, height: 468 })
  await flush()
  pitch.requests.shift()!.reply({ width: 10, height: 10 })
  await flush()
  expect(pitch.movers()).toHaveLength(0)
  await new Promise(resolve => setTimeout(resolve, 140))
  const retry = pitch.requests.shift()!
  expect(retry.selector).toBe('.pitch__area')
  retry.reply({ width: 351, height: 468 })
  await flush()
  expect(pitch.movers()).toHaveLength(11)
})

test('resize waits for the new native area and preserves saved custom positions', async () => {
  const pitch = setup()
  pitch.props.players[5] = { ...pitch.props.players[5], x: 0.41, y: 0.57 }
  await flush()
  pitch.requests.shift()!.reply({ width: 351, height: 468 })
  await flush()
  pitch.requests.shift()!.reply({ width: 351, height: 468 })
  await flush()
  pitch.resize()
  pitch.requests.shift()!.reply({ width: 400, height: 600 })
  await flush()
  expect(pitch.movers()).toHaveLength(0)
  pitch.requests.shift()!.reply({ width: 400, height: 600 })
  await flush()
  expect(pitch.movers()).toHaveLength(11)
  expect(pitch.movers()[5].props.x).toBe(boxPositionPx(pitch.props.players[5], { width: 400, height: 600 }, 375).x)
  expect(pitch.props.players[5].x).toBe(0.41)
  expect(pitch.props.players[5].y).toBe(0.57)
})

test('a stale area callback cannot release movers for a newer resize', async () => {
  const pitch = setup()
  await flush()
  pitch.requests.shift()!.reply({ width: 351, height: 468 })
  await flush()
  const stale = pitch.requests.shift()!
  pitch.resize()
  pitch.requests.shift()!.reply({ width: 400, height: 600 })
  await flush()
  stale.reply({ width: 351, height: 468 })
  await flush()
  expect(pitch.movers()).toHaveLength(0)
  pitch.requests.shift()!.reply({ width: 400, height: 600 })
  await flush()
  expect(pitch.movers()).toHaveLength(11)
})

test('read-only public pitches use the same initial layout while disabling dragging', async () => {
  const pitch = setup(true)
  await flush()
  pitch.requests.shift()!.reply({ width: 351, height: 468 })
  await flush()
  expect(pitch.movers()).toHaveLength(0)
  pitch.requests.shift()!.reply({ width: 351, height: 468 })
  await flush()
  expect(pitch.movers()).toHaveLength(11)
  expect(pitch.movers().every(n => n.props.disabled === true)).toBe(true)
})

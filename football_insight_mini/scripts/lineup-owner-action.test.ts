import { expect, test } from 'bun:test'
import { readFileSync } from 'node:fs'
import { createRequire } from 'node:module'
import vm from 'node:vm'
import { compileScript, compileTemplate, parse } from '@vue/compiler-sfc'
import * as vue from 'vue'
import ts from 'typescript'

const filename = new URL('../src/pages/lineup/view.vue', import.meta.url)
const { descriptor } = parse(readFileSync(filename, 'utf8'), { filename: filename.pathname })
const script = compileScript(descriptor, { id: 'owner-action-test' })
const template = compileTemplate({ source: descriptor.template!.content, filename: filename.pathname, id: 'owner-action-test' })
const compile = (source: string) => ts.transpileModule(source, {
  compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2020 },
}).outputText
const require = createRequire(filename)

function action(owner: boolean, pending = false) {
  const urls: string[] = []
  const detail = vue.ref({ team: { team_id: 77680, team_name: '成都蓉城' }, author: { display_name: '球迷' }, formation: '4-3-3', likes_count: 0 })
  const noop = () => {}
  const modules: Record<string, any> = {
    vue: { ...vue, onBeforeUnmount: noop, watch: noop, resolveComponent: (name: string) => name },
    '@dcloudio/uni-app': { onLoad: noop, onShow: noop, onShareAppMessage: noop },
    '../../composables/useNavigationLayout': { useNavigationLayout: () => ({ navigationPageStyle: vue.ref('') }) },
    '../../stores/appSession': { useAppSession: () => ({ revision: vue.ref(0) }) },
    '../../composables/useLineupCommunity': { createLineupDetail: () => ({ detail, loading: vue.ref(false), error: vue.ref(''), pendingLikes: vue.ref([]), dispose: noop }) },
    './useLineupOwnership': { useLineupOwnership: () => ({ isOwner: vue.ref(owner), loading: vue.ref(pending), dispose: noop }) },
    './lineupPoster': { publicPitch: () => [], lineupSharePayload: noop },
    './useLineupShare': { useLineupShare: () => ({ visible: vue.ref(false), busy: vue.ref(false), error: vue.ref(''), imagePath: vue.ref(''), payload: vue.ref(null), open: noop }) },
  }
  const load = (id: string) => modules[id] ?? (id.endsWith('.vue') ? {} : require(id))
  const scriptExports: Record<string, any> = {}
  vm.runInNewContext(compile(script.content), { exports: scriptExports, require: load, uni: { navigateTo: ({ url }: { url: string }) => urls.push(url) } })
  const bindings = scriptExports.default.setup({}, { expose: noop })
  const templateExports: Record<string, any> = {}
  vm.runInNewContext(compile(template.code), { exports: templateExports, require: load })
  const tree = templateExports.render(vue.proxyRefs(bindings), [])
  function all(vnode: any): any[] {
    if (!vnode || typeof vnode !== 'object') return []
    return [vnode, ...(Array.isArray(vnode.children) ? vnode.children.flatMap(all) : [])]
  }
  const button = all(tree).find(vnode => vnode.type === 'button' && vnode.props?.class?.includes('community-action--active'))
  return { button, urls }
}

test('own public lineup shows Edit and routes to the existing editor for its team', () => {
  const { button, urls } = action(true)
  expect(button.children).toBe('编辑')
  button.props.onTap()
  expect(urls).toEqual(['/pages/lineup/index?teamId=77680'])
})

test('someone else’s lineup retains Arrange my own and opens the same team editor', () => {
  const { button, urls } = action(false)
  expect(button.children).toBe('我也排一套')
  button.props.onTap()
  expect(urls).toEqual(['/pages/lineup/index?teamId=77680'])
})

test('ownership lookup shows a disabled loading action instead of a premature label', () => {
  const { button } = action(false, true)
  expect(button.children).toBe('加载中…')
  expect(button.props.disabled).toBe(true)
})

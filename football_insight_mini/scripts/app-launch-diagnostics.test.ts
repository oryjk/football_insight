import { expect, test } from 'bun:test'
import { readFileSync } from 'node:fs'
import vm from 'node:vm'
import { compileScript, parse } from '@vue/compiler-sfc'
import ts from 'typescript'

const compile = (source: string) => ts.transpileModule(source, {
  compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2020 },
}).outputText
const buildInfoSource = readFileSync(new URL('../src/utils/devBuildInfo.ts', import.meta.url), 'utf8')

function launch(buildInfo?: { revision: string; builtAt: string }) {
  const globals = buildInfo ? { __FOOTBALL_BUILD_INFO__: buildInfo } : {}
  const infoExports: Record<string, any> = {}
  vm.runInNewContext(compile(buildInfoSource), { exports: infoExports, ...globals })
  const calls: string[] = []
  const action = (name: string) => () => { calls.push(name); return Promise.resolve() }
  const logs: string[] = []
  let onLaunch: (() => void) | undefined
  const dependencies: Record<string, any> = {
    'vue': { defineComponent: (options: any) => options, watch: () => {} },
    '@dcloudio/uni-app': {
      onLaunch: (callback: () => void) => { onLaunch = callback }, onShow: () => {}, onHide: () => {},
    },
    './stores/appSession': { useAppSession: () => ({ bootstrapSession: action('session') }) },
    './stores/miniReview': { useMiniReviewStatus: () => ({ ensureMiniReviewReady: action('review') }) },
    './stores/runtimeConfig': { useRuntimeConfig: () => ({ ensureRuntimeConfig: action('config') }) },
    './utils/acquisitionSource': { captureAcquisitionSource: action('acquisition') },
    './utils/miniProgramUpdate': { setupMiniProgramUpdatePrompt: action('update') },
    './utils/devBuildInfo': infoExports,
  }
  const source = readFileSync(new URL('../src/App.vue', import.meta.url), 'utf8')
    .replace(/\/\/ #ifndef MP-WEIXIN[\s\S]*?\/\/ #endif/g, '')
    .replaceAll('import.meta.env.DEV', 'true')
  const { descriptor } = parse(source)
  const compiled = compileScript(descriptor, { id: 'app-launch-diagnostics' })
  const exports: Record<string, any> = {}
  vm.runInNewContext(compile(compiled.content), {
    exports, ...globals,
    require: (id: string) => {
      if (!(id in dependencies)) throw new Error(`Unexpected import ${id}`)
      return dependencies[id]
    },
    console: { log: () => {}, info: (_label: string, value: string) => logs.push(value) },
  })
  exports.default.setup({}, { expose: () => {} })
  onLaunch!()
  return { calls, logs }
}

test('App startup continues when a running dev compiler lacks the build-info define', () => {
  const result = launch()
  expect(result.calls).toEqual(['acquisition', 'update', 'session', 'review', 'config'])
  expect(JSON.parse(result.logs[0]).revision).toBe('unavailable')
})

test('fresh compiler metadata is preserved without changing startup behavior', () => {
  const buildInfo = { revision: 'abc123', builtAt: '2026-10-19T15:00:00.000Z' }
  const result = launch(buildInfo)
  expect(result.calls).toEqual(['acquisition', 'update', 'session', 'review', 'config'])
  expect(JSON.parse(result.logs[0])).toEqual(buildInfo)
})

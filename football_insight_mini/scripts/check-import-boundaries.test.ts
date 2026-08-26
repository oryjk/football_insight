import { describe, expect, test } from 'bun:test'
import { spawnSync } from 'node:child_process'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const here = path.dirname(fileURLToPath(import.meta.url))
const script = path.join(here, 'check-import-boundaries.mjs')
const fixtureRoot = path.join(here, 'fixtures', 'import-boundaries')

const CROSS_PAGE_MESSAGE = '页面间禁止互引（共享逻辑放 src/utils、共享组件进 src/components）'

function runCheckerOnFixture(): { status: number | null; violations: string[] } {
  const result = spawnSync(process.execPath, [script], {
    env: { ...process.env, BOUNDARY_CHECK_ROOT: fixtureRoot },
    encoding: 'utf8',
  })
  const violations = result.stderr
    .split('\n')
    .filter((line) => line.startsWith('  - '))
    .map((line) => line.slice(4).trim())
    .sort()
  return { status: result.status, violations }
}

describe('check-import-boundaries fixture', () => {
  test('reports every planted violation and nothing else', () => {
    const { status, violations } = runCheckerOnFixture()

    expect(status).toBe(1)
    expect(violations).toEqual([
      'src/api/badDeps.ts: api 层只能依赖 request/config/types/utils → ../utils/format',
      'src/api/badRequest.ts: 只有 utils/request.ts 可以直接调用 uni.request',
      'src/components/BadApiWidget.vue: 组件禁止依赖 api → ../api/insight',
      'src/components/BadAuthWidget.vue: 组件禁止依赖认证存储 → ../utils/authStorage',
      'src/components/BadDynamicImport.vue: 组件禁止依赖 api → ../api/insight',
      'src/components/BadPageDepWidget.vue: 全局组件禁止依赖页面层 → ../pages/home/helpers',
      'src/components/BadSideEffectImport.vue: 组件禁止依赖 api → ../api/insight',
      'src/components/BadSpacedRequest.vue: 只有 utils/request.ts 可以直接调用 uni.request',
      'src/pages/home/components/BadPanel.vue: 组件禁止依赖 api → ../../../api/insight',
      `src/pages/home/components/BadPanel.vue: ${CROSS_PAGE_MESSAGE}→ ../../insights/helpers`,
      `src/pages/rankings/index.vue: ${CROSS_PAGE_MESSAGE}→ ../home/helpers`,
      'src/pages/orphan/nested/DeepWidget.vue: 无法判定分层身份（登记进 pages.json、移入 components/ 或页面目录直属）',
    ].sort())
  })

  test('allows sanctioned shapes and keeps the allowlist scoped to its target', () => {
    const { violations } = runCheckerOnFixture()
    const reported = violations.join('\n')

    // 允许案例：注册页面、页面容器、嵌套注册页面可以依赖 api；页面局部组件可引本页 helpers。
    expect(violations.some((line) => line.startsWith('src/pages/home/index.vue:'))).toBe(false)
    expect(violations.some((line) => line.startsWith('src/pages/matches/MatchesContent.vue:'))).toBe(false)
    expect(violations.some((line) => line.startsWith('src/pages/user/settings/index.vue:'))).toBe(false)
    expect(violations.some((line) => line.startsWith('src/pages/home/components/HomePanel.vue:'))).toBe(false)
    expect(violations.some((line) => line.startsWith('src/components/FiWidget.vue:'))).toBe(false)
    expect(violations.some((line) => line.startsWith('src/api/insight.ts:'))).toBe(false)

    // 误报防护：注释、字符串字面量、模板文本里的 import 示例代码不是真实导入。
    expect(violations.some((line) => line.startsWith('src/components/FakeImportInComment.vue:'))).toBe(false)
    expect(violations.some((line) => line.startsWith('src/pages/home/components/FakeImportInTemplate.vue:'))).toBe(false)

    // allowlist 只放行 MatchesContent 这一个目标：同一文件里其它跨页导入仍然报错。
    expect(reported).not.toContain('MatchesContent')
    expect(violations.filter((line) => line.startsWith('src/pages/rankings/index.vue:'))).toEqual([
      `src/pages/rankings/index.vue: ${CROSS_PAGE_MESSAGE}→ ../home/helpers`,
    ])
  })
})

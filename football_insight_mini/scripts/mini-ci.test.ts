import { readFileSync } from 'node:fs'

import { describe, expect, test } from 'bun:test'

const miniCiSource = readFileSync(new URL('./mini-ci.mjs', import.meta.url), 'utf8')
const syncVersionSource = readFileSync(new URL('./sync-manifest-version.mjs', import.meta.url), 'utf8')
const packageJson = JSON.parse(readFileSync(new URL('../package.json', import.meta.url), 'utf8'))

describe('football mini ci release flow (aligned with registration_system_mini)', () => {
  test('uploads via miniprogram-ci directly instead of a shared cli wrapper', () => {
    expect(miniCiSource.includes('import ci from "miniprogram-ci";')).toBe(true)
    expect(miniCiSource.includes('spawn')).toBe(false)
    expect(miniCiSource.includes('mini-program-ci-cli')).toBe(false)
  })

  test('resolves the upload private key from the project root by appid', () => {
    expect(miniCiSource.includes('process.env.MINI_CI_PRIVATE_KEY_PATH || path.join(projectRoot, `private.${appid}.key`)')).toBe(true)
    expect(miniCiSource.includes('const robot = Number(argValue("--robot")) || 1;')).toBe(true)
  })

  test('allocates review versions from this backend registry with football identity', () => {
    expect(syncVersionSource.includes('const DEFAULT_MINI_REVIEW_API_URL = "https://match.oryjk.cn/api/v1";')).toBe(true)
    expect(syncVersionSource.includes('const DEFAULT_MINI_REVIEW_PROJECT_CODE = "football_insight_mini";')).toBe(true)
    expect(syncVersionSource.includes('/mini-review/allocate')).toBe(true)
    expect(syncVersionSource.includes('"X-Api-Key": config.apiKey')).toBe(true)
    expect(syncVersionSource.includes('MINI_REVIEW_SKIP')).toBe(true)
  })

  test('wires mp:release to a full build followed by ci upload', () => {
    expect(packageJson.scripts['mp:release']).toBe('bun run build:mp-weixin && node scripts/mini-ci.mjs upload')
    expect(packageJson.scripts['mp:preview']).toBe('node scripts/mini-ci.mjs preview')
    expect(packageJson.scripts['mp:upload']).toBeUndefined()
    expect(packageJson.scripts['build:mp-weixin']).toContain('verify-mp-component-registrations.mjs')
  })
})

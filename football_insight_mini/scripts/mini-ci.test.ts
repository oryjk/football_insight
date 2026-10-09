import { afterEach, describe, expect, test } from 'bun:test'
import { mkdirSync, mkdtempSync, readFileSync, rmSync, symlinkSync, writeFileSync } from 'node:fs'
import { createRequire } from 'node:module'
import { tmpdir } from 'node:os'
import path from 'node:path'

import { parseMiniCiArgs } from './mini-release-guards.mjs'
import {
  allocateReviewVersion,
  miniReviewConfig,
  parseVersionCode,
  syncManifestVersion,
} from './sync-manifest-version.mjs'

const productionApi = 'https://match.oryjk.cn/api/v1'
const localApi = 'http://127.0.0.1:8092/api/v1'
const roots: string[] = []

function tmpRoot(): string {
  const root = mkdtempSync(path.join(tmpdir(), 'football-mini-ci-test-'))
  roots.push(root)
  return root
}

afterEach(() => roots.splice(0).forEach((root) => rmSync(root, { recursive: true, force: true })))

describe('mini-ci upload arguments', () => {
  test('defaults to robot 1 and keeps the test-upload robot 2 line', () => {
    expect(parseMiniCiArgs([])).toEqual({ robot: 1, desc: undefined })
    expect(parseMiniCiArgs(['--', '--robot', '2', '--desc', '体验版说明'])).toEqual({ robot: 2, desc: '体验版说明' })
    expect(parseMiniCiArgs(['--desc', '修复说明'])).toEqual({ robot: 1, desc: '修复说明' })
  })

  test('rejects invalid robots, unknown flags and duplicates', () => {
    for (const args of [
      ['--robot', '0'],
      ['--robot', '3'],
      ['--robot', 'abc'],
      ['--robot'],
      ['--desc'],
      ['--mode', 'production'],
      ['--robot', '1', '--robot', '2'],
      ['--desc', 'a', '--desc', 'b'],
    ]) {
      expect(() => parseMiniCiArgs(args)).toThrow()
    }
  })
})

describe('mini review version allocation', () => {
  const config = {
    apiKey: 'test-key',
    baseUrl: 'https://match.oryjk.cn/api/v1',
    projectCode: 'football_insight_mini',
    skip: false,
  }

  test('posts to this project registry with the football identity and api key', async () => {
    const calls: Array<{ url: string; options: RequestInit }> = []
    const fetchImpl = async (url: string, options: RequestInit) => {
      calls.push({ url, options })
      return new Response(JSON.stringify({ version: '1.2.3' }), { status: 200 })
    }
    const version = await allocateReviewVersion({ config, explicitVersion: '', currentVersion: '1.2.2', fetchImpl: fetchImpl as typeof fetch })
    expect(version).toBe('1.2.3')
    expect(calls).toHaveLength(1)
    expect(calls[0].url).toBe('https://match.oryjk.cn/api/v1/mini-review/allocate')
    expect(calls[0].options.method).toBe('POST')
    expect((calls[0].options.headers as Record<string, string>)['X-Api-Key']).toBe('test-key')
    expect(JSON.parse(String(calls[0].options.body))).toEqual({
      project_code: 'football_insight_mini',
      current_version: '1.2.2',
    })
  })

  test('passes an explicit MINI_PROGRAM_VERSION through to the registry', async () => {
    const bodies: string[] = []
    const fetchImpl = async (_url: string, options: RequestInit) => {
      bodies.push(String(options.body))
      return new Response(JSON.stringify({ version: '9.9.9' }), { status: 200 })
    }
    await allocateReviewVersion({ config, explicitVersion: '9.9.9', currentVersion: '1.2.2', fetchImpl: fetchImpl as typeof fetch })
    expect(JSON.parse(bodies[0]).version).toBe('9.9.9')
  })

  test('fails on registry errors and invalid responses', async () => {
    const failing = async () => new Response('boom', { status: 500 })
    await expect(
      allocateReviewVersion({ config, fetchImpl: failing as unknown as typeof fetch }),
    ).rejects.toThrow('HTTP 500')

    const invalid = async () => new Response(JSON.stringify({ ok: true }), { status: 200 })
    await expect(
      allocateReviewVersion({ config, fetchImpl: invalid as unknown as typeof fetch }),
    ).rejects.toThrow('响应无效')
  })

  test('skip returns null without an api key; missing key without skip is an error', async () => {
    expect(await allocateReviewVersion({ config: { ...config, apiKey: '', skip: true } })).toBeNull()
    await expect(allocateReviewVersion({ config: { ...config, apiKey: '' } })).rejects.toThrow('MINI_REVIEW_API_KEY')
  })

  test('miniReviewConfig reads explicit env first and honors skip flags', () => {
    expect(miniReviewConfig({ env: { MINI_REVIEW_SKIP: '1' }, envFile: '/nonexistent' }).skip).toBe(true)
    expect(miniReviewConfig({ env: {}, envFile: '/nonexistent' }).projectCode).toBe('football_insight_mini')
    expect(miniReviewConfig({ env: { MINI_REVIEW_API_KEY: 'k' }, envFile: '/nonexistent' }).apiKey).toBe('k')
  })

  test('parseVersionCode only accepts x.y.z', () => {
    expect(parseVersionCode('1.0.64')).toBe('10064')
    expect(() => parseVersionCode('1.0')).toThrow('x.y.z')
    expect(() => parseVersionCode('v1.0.1')).toThrow('x.y.z')
  })
})

describe('syncManifestVersion with stubbed allocation', () => {
  function versionFixture(versionName = '1.0.64', versionCode = '10064') {
    const root = tmpRoot()
    mkdirSync(path.join(root, 'src/config'), { recursive: true })
    writeFileSync(
      path.join(root, 'src/manifest.json'),
      JSON.stringify({ versionName, versionCode, 'mp-weixin': { appid: 'wx-test' } }, null, 2),
    )
    return root
  }

  test('writes the allocated version into manifest and the generated constant', async () => {
    const root = versionFixture()
    const result = await syncManifestVersion({
      projectRoot: root,
      env: { MINI_REVIEW_SKIP: '0' },
      allocate: async () => '1.0.65',
    })
    expect(result).toEqual({ versionName: '1.0.65', versionCode: '10065', changed: true })
    const manifest = JSON.parse(readFileSync(path.join(root, 'src/manifest.json'), 'utf8'))
    expect(manifest.versionName).toBe('1.0.65')
    expect(manifest.versionCode).toBe('10065')
    const generated = readFileSync(path.join(root, 'src/config/generatedMiniProgramVersion.ts'), 'utf8')
    expect(generated).toContain('MINI_PROGRAM_VERSION = "1.0.65"')
    expect(generated).toContain('MINI_PROGRAM_VERSION_CODE = "10065"')
  })

  test('keeps the manifest untouched when the registry reuses the in-review version', async () => {
    const root = versionFixture()
    const before = readFileSync(path.join(root, 'src/manifest.json'), 'utf8')
    const result = await syncManifestVersion({
      projectRoot: root,
      env: { MINI_REVIEW_SKIP: '0' },
      allocate: async () => '1.0.64',
    })
    expect(result).toEqual({ versionName: '1.0.64', versionCode: '10064', changed: false })
    expect(readFileSync(path.join(root, 'src/manifest.json'), 'utf8')).toBe(before)
  })

  test('offline skip falls back to the manifest version without any allocation', async () => {
    const root = versionFixture()
    const result = await syncManifestVersion({
      projectRoot: root,
      env: { MINI_REVIEW_SKIP: '1' },
      allocate: allocateReviewVersion,
    })
    expect(result).toEqual({ versionName: '1.0.64', versionCode: '10064', changed: false })
  })

  test('explicit MINI_PROGRAM_VERSION wins in offline mode', async () => {
    const root = versionFixture()
    const result = await syncManifestVersion({
      projectRoot: root,
      env: { MINI_REVIEW_SKIP: '1', MINI_PROGRAM_VERSION: '2.0.0' },
      allocate: allocateReviewVersion,
    })
    expect(result).toEqual({ versionName: '2.0.0', versionCode: '20000', changed: true })
    const generated = readFileSync(path.join(root, 'src/config/generatedMiniProgramVersion.ts'), 'utf8')
    expect(generated).toContain('MINI_PROGRAM_VERSION = "2.0.0"')
  })

  test('propagates allocation failures without writing files', async () => {
    const root = versionFixture()
    await expect(
      syncManifestVersion({
        projectRoot: root,
        env: { MINI_REVIEW_SKIP: '0' },
        allocate: async () => {
          throw new Error('registry down')
        },
      }),
    ).rejects.toThrow('registry down')
    expect(readFileSync(path.join(root, 'src/manifest.json'), 'utf8')).toContain('"versionName": "1.0.64"')
  })
})

describe('direct mini-ci invocation (bypassing the entry)', () => {
  function cliFixture(api: string) {
    const root = tmpRoot()
    mkdirSync(path.join(root, 'scripts'), { recursive: true })
    mkdirSync(path.join(root, 'src'), { recursive: true })
    mkdirSync(path.join(root, 'node_modules'), { recursive: true })
    // Only JSON5 is available in this fixture: an accidental SDK load fails instead of contacting WeChat.
    const require = createRequire(import.meta.url)
    symlinkSync(path.dirname(require.resolve('json5/package.json')), path.join(root, 'node_modules/json5'), 'dir')
    for (const file of ['mini-ci.mjs', 'mini-release-guards.mjs', 'verify-mp-api-base.mjs']) {
      writeFileSync(path.join(root, 'scripts', file), readFileSync(new URL(file, import.meta.url)))
    }
    writeFileSync(path.join(root, 'src/manifest.json'), JSON.stringify({ 'mp-weixin': { appid: 'wx-test-only' }, versionName: '0.0.1' }))
    writeFileSync(path.join(root, '.env.production'), `VITE_API_BASE_URL=${productionApi}\n`)
    const dist = path.join(root, 'dist/build/mp-weixin')
    mkdirSync(path.join(dist, 'config'), { recursive: true })
    writeFileSync(path.join(dist, 'app.json'), '{}')
    writeFileSync(path.join(dist, 'config/apiBase.js'), `exports.API_BASE_URL = ${JSON.stringify(api)};`)
    const privateKeyPath = path.join(root, 'private.test.key')
    writeFileSync(privateKeyPath, 'not-a-real-key')
    return { root, privateKeyPath }
  }

  function runMiniCi(root: string, privateKeyPath: string, command: string) {
    return Bun.spawnSync([process.execPath, '--no-env-file', '--bun', 'scripts/mini-ci.mjs', command], {
      cwd: root,
      env: { ...process.env, MINI_CI_PRIVATE_KEY_PATH: privateKeyPath },
      stdout: 'pipe',
      stderr: 'pipe',
    })
  }

  test('rejects a local bundle before loading the WeChat SDK', () => {
    const { root, privateKeyPath } = cliFixture(localApi)
    const result = runMiniCi(root, privateKeyPath, 'upload')
    expect(result.exitCode).toBe(1)
    expect(result.stderr.toString()).toContain('小程序上传产物的 API 地址必须是')
    expect(result.stderr.toString()).not.toContain('Cannot find package')
    expect(result.stdout.toString()).not.toContain('上传完成')
  })

  test('upload also refuses a checkout outside main / clean / pushed state', () => {
    const { root, privateKeyPath } = cliFixture(productionApi)
    const result = runMiniCi(root, privateKeyPath, 'upload')
    expect(result.exitCode).toBe(1)
    expect(result.stderr.toString()).toContain('Git')
    expect(result.stderr.toString()).not.toContain('Cannot find package')
    expect(result.stdout.toString()).not.toContain('上传完成')
  })

  test('preview rejects a local bundle without touching the git gate', () => {
    const { root, privateKeyPath } = cliFixture(localApi)
    const result = runMiniCi(root, privateKeyPath, 'preview')
    expect(result.exitCode).toBe(1)
    expect(result.stderr.toString()).toContain('小程序上传产物的 API 地址必须是')
    expect(result.stderr.toString()).not.toContain('Cannot find package')
  })

  test('rejects invalid arguments before any check runs', () => {
    const { root, privateKeyPath } = cliFixture(productionApi)
    const result = Bun.spawnSync([process.execPath, '--no-env-file', '--bun', 'scripts/mini-ci.mjs', 'upload', '--robot', '9'], {
      cwd: root,
      env: { ...process.env, MINI_CI_PRIVATE_KEY_PATH: privateKeyPath },
      stdout: 'pipe',
      stderr: 'pipe',
    })
    expect(result.exitCode).toBe(1)
    expect(result.stderr.toString()).toContain('--robot')
  })
})

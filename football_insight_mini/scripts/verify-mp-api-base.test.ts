import { afterEach, describe, expect, test } from 'bun:test'
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import path from 'node:path'
import { verifyMiniProgramApiBase } from './verify-mp-api-base.mjs'

const productionApi = 'https://match.oryjk.cn/api/v1'
const localApi = 'http://127.0.0.1:8092/api/v1'
const roots: string[] = []

function artifact(source: string | undefined): string {
  const root = mkdtempSync(path.join(tmpdir(), 'football-mini-api-'))
  roots.push(root)
  const configDir = path.join(root, 'dist/build/mp-weixin/config')
  mkdirSync(configDir, { recursive: true })
  writeFileSync(path.join(root, 'dist/build/mp-weixin/app.json'), '{}')
  if (source !== undefined) {
    writeFileSync(path.join(configDir, 'apiBase.js'), source)
  }
  return root
}

function artifactWithExport(apiBaseUrl: string | undefined): string {
  return artifact(`exports.API_BASE_URL = ${JSON.stringify(apiBaseUrl)};`)
}

afterEach(() => roots.splice(0).forEach((root) => rmSync(root, { recursive: true, force: true })))

describe('mini-program upload API address', () => {
  test('accepts the compiled production HTTPS endpoint', () => {
    expect(verifyMiniProgramApiBase(artifactWithExport(productionApi))).toBe(productionApi)
  })

  test.each([
    localApi,
    'http://localhost:8092/api/v1',
    'http://match.oryjk.cn/api/v1',
    'https://example.com/api/v1',
    undefined,
  ])('rejects an unusable upload endpoint: %s', (apiBaseUrl) => {
    expect(() => verifyMiniProgramApiBase(artifactWithExport(apiBaseUrl))).toThrow('小程序上传产物的 API 地址必须是')
  })

  test('re-evaluates the same path: a rewritten local export is rejected after a production pass', () => {
    const root = artifactWithExport(productionApi)
    expect(verifyMiniProgramApiBase(root)).toBe(productionApi)
    writeFileSync(path.join(root, 'dist/build/mp-weixin/config/apiBase.js'), `exports.API_BASE_URL = ${JSON.stringify(localApi)};`)
    expect(() => verifyMiniProgramApiBase(root)).toThrow('小程序上传产物的 API 地址必须是')
  })

  test('rejects a wrong export even when the production URL appears elsewhere in the file', () => {
    const root = artifact(`const unusedProduction = ${JSON.stringify(productionApi)}; exports.API_BASE_URL = ${JSON.stringify(localApi)};`)
    expect(() => verifyMiniProgramApiBase(root)).toThrow('小程序上传产物的 API 地址必须是')
  })

  test('rejects a missing app.json or missing API module', () => {
    const root = artifactWithExport(productionApi)
    rmSync(path.join(root, 'dist/build/mp-weixin/app.json'))
    expect(() => verifyMiniProgramApiBase(root)).toThrow('app.json')

    const noModule = artifact(undefined)
    expect(() => verifyMiniProgramApiBase(noModule)).toThrow('config/apiBase.js')
  })

  test('rejects modules that throw, loop forever or rely on process/require', () => {
    expect(() => verifyMiniProgramApiBase(artifact('throw new Error("boom");'))).toThrow('无法读取编译包')
    expect(() => verifyMiniProgramApiBase(artifact('while (true) {}'))).toThrow('无法读取编译包')
    expect(() => verifyMiniProgramApiBase(artifact('exports.API_BASE_URL = process.env.VITE_API_BASE_URL;'))).toThrow('无法读取编译包')
    expect(() => verifyMiniProgramApiBase(artifact('exports.API_BASE_URL = require("./other.js");'))).toThrow('无法读取编译包')
  }, 15000)
})

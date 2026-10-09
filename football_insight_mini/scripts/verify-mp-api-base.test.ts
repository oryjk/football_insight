import { afterEach, describe, expect, test } from 'bun:test'
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import path from 'node:path'
import { verifyMiniProgramApiBase } from './verify-mp-api-base.mjs'

const roots: string[] = []

function artifact(apiBaseUrl: string | undefined): string {
  const root = mkdtempSync(path.join(tmpdir(), 'football-mini-api-'))
  roots.push(root)
  const configDir = path.join(root, 'dist/build/mp-weixin/config')
  mkdirSync(configDir, { recursive: true })
  writeFileSync(path.join(configDir, 'apiBase.js'), `exports.API_BASE_URL = ${JSON.stringify(apiBaseUrl)};`)
  return root
}

afterEach(() => roots.splice(0).forEach((root) => rmSync(root, { recursive: true, force: true })))

describe('mini-program upload API address', () => {
  test('accepts the compiled production HTTPS endpoint', () => {
    expect(verifyMiniProgramApiBase(artifact('https://match.oryjk.cn/api/v1'))).toBe('https://match.oryjk.cn/api/v1')
  })

  test.each([
    'http://127.0.0.1:8092/api/v1',
    'http://localhost:8092/api/v1',
    'http://match.oryjk.cn/api/v1',
    'https://example.com/api/v1',
    undefined,
  ])('rejects an unusable upload endpoint: %s', (apiBaseUrl) => {
    expect(() => verifyMiniProgramApiBase(artifact(apiBaseUrl))).toThrow('小程序上传产物的 API 地址必须是')
  })
})

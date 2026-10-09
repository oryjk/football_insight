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

  test('sandbox escape via constructor chains cannot reach the host realm', () => {
    // 制品即使导出正确地址，也不得经宿主对象原型链（exports.constructor.constructor = 宿主 Function）
    // 触达宿主 process/Bun 并产生宿主可观察的副作用（这里以改写宿主 process.env 为探针）。
    const root = artifact(`
      try {
        const hostProcess = exports.constructor.constructor('return process')();
        hostProcess.env.__FI_SANDBOX_PWNED__ = 'yes';
      } catch (e) {}
      try {
        const hostBun = exports.constructor.constructor('return typeof Bun !== "undefined" && Bun')();
        if (hostBun) hostProcess.env.__FI_SANDBOX_PWNED_BUN__ = 'yes';
      } catch (e) {}
      exports.API_BASE_URL = ${JSON.stringify(productionApi)};
    `)
    delete process.env.__FI_SANDBOX_PWNED__
    delete process.env.__FI_SANDBOX_PWNED_BUN__
    try {
      expect(verifyMiniProgramApiBase(root)).toBe(productionApi)
      expect(process.env.__FI_SANDBOX_PWNED__).toBeUndefined()
      expect(process.env.__FI_SANDBOX_PWNED_BUN__).toBeUndefined()
    } finally {
      delete process.env.__FI_SANDBOX_PWNED__
      delete process.env.__FI_SANDBOX_PWNED_BUN__
    }
  })

  test('sandbox evaluation provides no network/process/require capabilities', () => {
    // 校验拒绝时的报错会带上实际导出值，借此观察沙箱内各宿主能力的可见性。
    const root = artifact(
      `exports.API_BASE_URL = ['process:'+typeof process, 'Bun:'+typeof Bun, 'require:'+typeof require, 'fetch:'+typeof fetch, 'XMLHttpRequest:'+typeof XMLHttpRequest].join(' ');`,
    )
    expect(() => verifyMiniProgramApiBase(root)).toThrow('process:undefined Bun:undefined require:undefined fetch:undefined XMLHttpRequest:undefined')
  })

  test('rejects a non-string export without trusting sandbox objects', () => {
    // 即使对象 toString 能拼出生产地址，也只有真实字符串导出才被接受（沙箱对象不当宿主可信值用）。
    const root = artifact(`exports.API_BASE_URL = { toString() { return ${JSON.stringify(productionApi)}; } };`)
    expect(() => verifyMiniProgramApiBase(root)).toThrow('小程序上传产物的 API 地址必须是')
  })
})

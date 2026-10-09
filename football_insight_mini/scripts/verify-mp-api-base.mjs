#!/usr/bin/env bun
// 校验小程序构建产物真实导出的 API 地址。
// 每次调用都在独立 vm 上下文重新求值 exports.API_BASE_URL：不走 require 模块缓存，
// 同路径重写后再次校验一定读到新值；求值环境不提供网络、process 或 require，并限定执行时间。
import { existsSync, readFileSync } from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'
import { runInNewContext } from 'node:vm'

export const PRODUCTION_API_BASE_URL = 'https://match.oryjk.cn/api/v1'

export function verifyMiniProgramApiBase(projectRoot) {
  const distPath = path.join(projectRoot, 'dist', 'build', 'mp-weixin')
  if (!existsSync(path.join(distPath, 'app.json'))) {
    throw new Error('缺少小程序产物 app.json，请先运行 bun run build:mp-weixin。')
  }
  const apiFile = path.join(distPath, 'config', 'apiBase.js')
  if (!existsSync(apiFile)) {
    throw new Error('缺少编译后的 config/apiBase.js，已阻止上传。')
  }

  // 检查真实导出值，而不是搜索 URL 字符串：文件里出现生产地址但导出错误同样拒绝。
  let actual
  try {
    actual = runInNewContext(`${readFileSync(apiFile, 'utf8')}\n;exports.API_BASE_URL;`, { exports: {} }, { timeout: 1000, filename: apiFile })
  } catch (error) {
    const detail = error instanceof Error ? error.message : String(error)
    throw new Error(`无法读取编译包的 exports.API_BASE_URL，已阻止上传（${detail}）；请检查 config/apiBase.js。`)
  }
  if (actual !== PRODUCTION_API_BASE_URL) {
    throw new Error(`小程序上传产物的 API 地址必须是 ${PRODUCTION_API_BASE_URL}，实际为 ${actual}；请重新执行生产构建`)
  }
  return actual
}

if (import.meta.main) {
  try {
    const projectRoot = fileURLToPath(new URL('..', import.meta.url))
    console.log(`[verify-mp-api-base] ${verifyMiniProgramApiBase(projectRoot)}`)
  } catch (error) {
    console.error(`[verify-mp-api-base] ${error instanceof Error ? error.message : error}`)
    process.exit(1)
  }
}

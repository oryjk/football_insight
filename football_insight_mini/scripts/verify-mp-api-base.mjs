#!/usr/bin/env bun
import { createRequire } from 'node:module'
import path from 'node:path'
import { fileURLToPath, pathToFileURL } from 'node:url'

const require = createRequire(import.meta.url)
const productionApiBaseUrl = 'https://match.oryjk.cn/api/v1'

export function verifyMiniProgramApiBase(projectRoot) {
  const configPath = path.join(projectRoot, 'dist/build/mp-weixin/config/apiBase.js')
  const { API_BASE_URL } = require(configPath)
  if (API_BASE_URL !== productionApiBaseUrl) {
    throw new Error(`小程序上传产物的 API 地址必须是 ${productionApiBaseUrl}，实际为 ${API_BASE_URL}；请重新执行生产构建`)
  }
  return API_BASE_URL
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  try {
    const projectRoot = fileURLToPath(new URL('..', import.meta.url))
    console.log(`[verify-mp-api-base] ${verifyMiniProgramApiBase(projectRoot)}`)
  } catch (error) {
    console.error(`[verify-mp-api-base] ${error instanceof Error ? error.message : error}`)
    process.exit(1)
  }
}

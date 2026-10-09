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
  // 沙箱不注入任何宿主对象（此前传入宿主创建的 exports，制品可经
  // exports.constructor.constructor 拿到宿主 Function 逃逸到 process/Bun）；
  // 载体 exports 在被求值代码内部建立，原型链只能到达沙箱自己的 realm。
  // Bun 在 vm 上下文中注入的 console 是沙箱 realm 本地对象，经原型链验证不构成宿主桥。
  const source = `'use strict';\nconst exports = {};\n${readFileSync(apiFile, 'utf8')}\n;exports.API_BASE_URL;`
  let actual
  try {
    actual = runInNewContext(source, undefined, { timeout: 1000, filename: apiFile })
  } catch (error) {
    // 沙箱抛出的对象是另一 realm 的值，只读取字符串 message，不调用其任何方法。
    const detail = error && typeof error === 'object' && typeof error.message === 'string' ? error.message : 'unknown'
    throw new Error(`无法读取编译包的 exports.API_BASE_URL，已阻止上传（${detail}）；请检查 config/apiBase.js。`)
  }
  // 只接受真实的字符串导出：沙箱对象（哪怕 toString 能拼出生产地址）不当宿主可信值用。
  if (typeof actual !== 'string' || actual !== PRODUCTION_API_BASE_URL) {
    const shown = typeof actual === 'string' ? actual : `非字符串导出（${typeof actual}）`
    throw new Error(`小程序上传产物的 API 地址必须是 ${PRODUCTION_API_BASE_URL}，实际为 ${shown}；请重新执行生产构建`)
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

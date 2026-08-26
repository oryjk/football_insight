#!/usr/bin/env node
// 依赖边界检查（pragmatic layering 的机器约束，接入 prebuild:mp-weixin）：
//  1. 组件层（src/components、src/pages/*/components）禁止 import api/** 与认证存储；
//  2. 页面 A 禁止 import 页面 B 的 helpers（跨页面共享逻辑放 src/utils）；
//  3. api/** 只能依赖 request、config、types 与 utils；
//  4. 只有 utils/request.ts 可以直接调用 uni.request（api/ai.ts 流式为例外）。
// 例外必须显式登记在 ALLOWLIST，并注明原因与清理计划。
import { readFileSync, readdirSync, statSync } from 'node:fs'
import path from 'node:path'

const ROOT = path.resolve(new URL('..', import.meta.url).pathname)
const SRC = path.join(ROOT, 'src')

const ALLOWLIST = {
  // FiAiChatSheet 是 AI 聊天 feature 模块（视图+流式+存储一体，845 行），
  // 待按“变化原因”拆分后移出例外（见 AGENTS.md backlog）。
  'src/components/FiAiChatSheet.vue': ['component-imports-api'],
  // AI 流式对话需要 enableChunked 的 uni.request，无法走统一 request 封装。
  'src/api/ai.ts': ['uni-request', 'api-extra-deps'],
}

const violations = []

function walk(dir, files = []) {
  for (const name of readdirSync(dir)) {
    const full = path.join(dir, name)
    if (statSync(full).isDirectory()) {
      walk(full, files)
    } else if (/\.(vue|ts)$/.test(name) && !name.endsWith('.test.ts')) {
      files.push(full)
    }
  }
  return files
}

const rel = (file) => path.relative(ROOT, file).replaceAll('\\', '/')

function importsOf(file) {
  const source = readFileSync(file, 'utf8')
  const imports = []
  const pattern = /(?:import|export)\s+(?:[\s\S]*?)from\s+['"]([^'"]+)['"]/g
  let match
  while ((match = pattern.exec(source))) {
    imports.push(match[1])
  }
  return imports
}

const allowed = (file, rule) => {
  const rules = ALLOWLIST[rel(file)]
  return Boolean(rules?.includes(rule))
}

const allFiles = walk(SRC)

for (const file of allFiles) {
  const relative = rel(file)
  const imports = importsOf(file)
  const isComponent = /^src\/(components|pages\/[^/]+\/components)\//.test(relative)

  if (isComponent) {
    for (const spec of imports) {
      if (/(^|\/)api\//.test(spec) || /authStorage/.test(spec)) {
        if (!allowed(file, 'component-imports-api')) {
          violations.push(`${relative}: 组件禁止依赖 api/authStorage → ${spec}`)
        }
      }
    }
  }

  const pageHelpers = /^src\/pages\/([^/]+)\/helpers/.exec(relative)
  if (pageHelpers) {
    for (const spec of imports) {
      const other = /^(\.\.\/|\.\/)([A-Za-z0-9-]+)\/helpers/.exec(spec)
      if (other && other[2] !== pageHelpers[1]) {
        violations.push(`${relative}: 页面间禁止互引 helpers（共享逻辑放 src/utils）→ ${spec}`)
      }
    }
  }

  if (/^src\/api\//.test(relative)) {
    for (const spec of imports) {
      // api 内部互相引用（./xxx 同层模块）合法；其余只允许 utils 白名单、config、types。
      const ok = spec.startsWith('./')
        || /^(\.\.\/)?(utils\/(request|apiError|authStorage|wechatCloud)|config|types)/.test(spec)
        || /^\.\.\/types/.test(spec) || /^\.\.\/config/.test(spec)
        || /^\.\.\/utils\/(request|apiError|authStorage|wechatCloud)/.test(spec)
      if (!ok && !allowed(file, 'api-extra-deps')) {
        violations.push(`${relative}: api 层只能依赖 request/config/types/utils → ${spec}`)
      }
    }
  }

  const source = readFileSync(file, 'utf8')
  if (/uni\.request\(/.test(source)) {
    const permitted = relative === 'src/utils/request.ts' || allowed(file, 'uni-request')
    if (!permitted) {
      violations.push(`${relative}: 只有 utils/request.ts 可以直接调用 uni.request`)
    }
  }
}

if (violations.length) {
  console.error('❌ 依赖边界检查未通过：')
  for (const item of violations) {
    console.error(`  - ${item}`)
  }
  process.exit(1)
}

console.log('✅ 依赖边界检查通过（组件↛api、页面不互引 helpers、api 依赖受限、uni.request 收口）')

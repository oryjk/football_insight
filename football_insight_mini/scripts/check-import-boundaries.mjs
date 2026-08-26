#!/usr/bin/env node
// 依赖边界检查（pragmatic layering 的机器约束，接入 prebuild:mp-weixin 与 GitHub Actions）。
// 文件身份先按目录与 pages.json 判定，import 一律解析成绝对路径后再匹配，避免 '../api'、别名等写法绕过：
//  - 组件层：src/components/**、src/pages/**/components/**
//  - 页面层：pages.json 注册的页面入口（含 pages/user/settings/index 这类嵌套页面）、
//            页面目录直属的 *.vue 页面容器（index.vue、*Content.vue）
//  - 页面模块：页面目录直属的 *.ts（helpers、poster、loginHelpers 等）
// 规则：
//  1. 组件层禁止 import api/** 与认证存储；src/components 还禁止依赖页面层；
//  2. 页面 A 的任何文件禁止 import 页面 B 的任何模块（helpers / 页面容器 / 局部组件，
//     跨页面共享：逻辑放 src/utils，组件进 src/components）；
//  3. api/** 只能依赖 request、config、types 与 utils 白名单；
//  4. 只有 utils/request.ts 可以直接调用 uni.request（api/ai.ts 流式为例外）。
// import 与 uni.request 用 TypeScript AST 提取（Vue 只取 <script> 块）：注释、字符串、模板文本里的
// 示例代码不会误报，`uni.request ({ ... })` 这类带空白的调用表达式也不会漏检。
// 已知边界：模板内联事件表达式不在检查范围。
// 例外必须显式登记在 ALLOWLIST（可精确到目标路径），并注明原因与清理计划。
// 允许/禁止案例的回归见 scripts/check-import-boundaries.test.ts 的 fixture。
import { readFileSync, readdirSync, statSync } from 'node:fs'
import path from 'node:path'
import ts from 'typescript'

// 测试可用 BOUNDARY_CHECK_ROOT 指向 fixture 树，默认检查本仓库。
const ROOT = process.env.BOUNDARY_CHECK_ROOT
  ? path.resolve(process.env.BOUNDARY_CHECK_ROOT)
  : path.resolve(new URL('..', import.meta.url).pathname)
const SRC = path.join(ROOT, 'src')

// 规则值为 null：整个文件在该规则下放行；值为路径数组：只放行列出的目标（精确到文件）。
const ALLOWLIST = {
  // FiAiChatSheet 是 AI 聊天 feature 模块（视图+流式+存储一体，845 行），
  // 待按“变化原因”拆分后移出例外（见 AGENTS.md backlog）。
  'src/components/FiAiChatSheet.vue': { 'component-imports-api': null },
  // AI 流式对话需要 enableChunked 的 uni.request，无法走统一 request 封装。
  'src/api/ai.ts': { 'uni-request': null, 'api-extra-deps': null },
  // 榜单页 tab 内嵌整个赛程视图；只放行 MatchesContent 这一个目标。
  // 出现第二处跨页需求时先抽 composable / 共享组件，不要扩大此例外。
  'src/pages/rankings/index.vue': {
    'cross-page-import': ['src/pages/matches/MatchesContent.vue'],
  },
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

const under = (target, dir) => target === dir || target.startsWith(`${dir}/`)

/** Vue SFC 只拼接 <script> 块（普通 + setup）；.ts 原文返回。模板与注释不参与检查。 */
function scriptSourceOf(file, raw) {
  if (!file.endsWith('.vue')) {
    return raw
  }
  let code = ''
  const pattern = /<script\b[^>]*>([\s\S]*?)<\/script>/g
  let match
  while ((match = pattern.exec(raw))) {
    code += `${match[1]}\n`
  }
  return code
}

/**
 * AST 提取依赖说明符与 uni.request 调用，覆盖：
 * import x from 'x' / export ... from 'x' / 副作用 import 'x' / 动态 import('x') /
 * import x = require('x') / require('x')。
 */
function analyzeScript(source) {
  const imports = new Set()
  let usesUniRequest = false
  const sourceFile = ts.createSourceFile('check.ts', source, ts.ScriptTarget.Latest, true, ts.ScriptKind.TS)

  function visit(node) {
    if (ts.isImportDeclaration(node) && ts.isStringLiteral(node.moduleSpecifier)) {
      imports.add(node.moduleSpecifier.text)
    } else if (ts.isExportDeclaration(node) && node.moduleSpecifier && ts.isStringLiteral(node.moduleSpecifier)) {
      imports.add(node.moduleSpecifier.text)
    } else if (ts.isImportEqualsDeclaration(node)) {
      const reference = node.moduleReference
      if (ts.isExternalModuleReference(reference) && ts.isStringLiteral(reference.expression)) {
        imports.add(reference.expression.text)
      }
    } else if (ts.isCallExpression(node)) {
      const argument = node.arguments[0]
      const isDynamicImport = node.expression.kind === ts.SyntaxKind.ImportKeyword
      const isRequire = ts.isIdentifier(node.expression) && node.expression.text === 'require'
      if ((isDynamicImport || isRequire) && argument && ts.isStringLiteral(argument)) {
        imports.add(argument.text)
      }
      if (
        ts.isPropertyAccessExpression(node.expression)
        && ts.isIdentifier(node.expression.expression)
        && node.expression.expression.text === 'uni'
        && node.expression.name.text === 'request'
      ) {
        usesUniRequest = true
      }
    }
    ts.forEachChild(node, visit)
  }

  ts.forEachChild(sourceFile, visit)
  return { imports: [...imports], usesUniRequest }
}

/** 解析静态 import 说明符为仓库相对路径；bare module（vue 等）与解析不了的返回 null。 */
function resolveSpec(fromFile, spec) {
  let target = null
  if (spec.startsWith('./') || spec.startsWith('../')) {
    target = path.resolve(path.dirname(fromFile), spec)
  } else if (spec.startsWith('@/')) {
    target = path.join(SRC, spec.slice(2))
  } else {
    return null
  }
  return rel(target)
}

// pages.json 是页面入口的唯一权威来源。
const REGISTERED_PAGES = new Set(
  JSON.parse(readFileSync(path.join(SRC, 'pages.json'), 'utf8')).pages.map((page) => `src/${page.path}.vue`),
)

const PAGE_DIR_RE = /^src\/pages\/([^/]+)\//

function classify(file) {
  const relative = rel(file)
  if (relative.startsWith('src/components/')) {
    return { layer: 'component', domain: null }
  }

  const pageMatch = PAGE_DIR_RE.exec(relative)
  if (!pageMatch) {
    return relative.startsWith('src/pages/') && relative.endsWith('.vue')
      ? { layer: 'unclassified', domain: null }
      : { layer: 'other', domain: null }
  }

  const domain = pageMatch[1]
  const segments = relative.slice(pageMatch[0].length).split('/')
  if (segments.slice(0, -1).includes('components')) {
    return { layer: 'component', domain }
  }
  if (REGISTERED_PAGES.has(relative)) {
    return { layer: 'page', domain }
  }
  if (segments.length === 1) {
    return relative.endsWith('.vue')
      ? { layer: 'page-container', domain }
      : { layer: 'page-module', domain }
  }
  // 深层 .vue：既不是注册页面也不在 components/ 下，身份不明，逼着登记而不是默默放过。
  return relative.endsWith('.vue')
    ? { layer: 'unclassified', domain }
    : { layer: 'page-module', domain }
}

const allowed = (file, rule, target) => {
  const entry = ALLOWLIST[rel(file)]?.[rule]
  if (entry === undefined) {
    return false
  }
  if (entry === null) {
    return true
  }
  const normalize = (value) => value.replace(/\.(ts|vue)$/, '')
  return entry.some((allowedTarget) => normalize(allowedTarget) === normalize(target))
}

const allFiles = walk(SRC)

for (const file of allFiles) {
  const relative = rel(file)
  const raw = readFileSync(file, 'utf8')
  const { imports, usesUniRequest } = analyzeScript(scriptSourceOf(file, raw))
  const { layer, domain } = classify(file)

  if (layer === 'unclassified') {
    violations.push(`${relative}: 无法判定分层身份（登记进 pages.json、移入 components/ 或页面目录直属）`)
    continue
  }

  const resolvedImports = imports
    .map((spec) => ({ spec, target: resolveSpec(file, spec) }))
    .filter((item) => item.target)

  if (layer === 'component') {
    for (const { spec, target } of resolvedImports) {
      const normalized = target.replace(/\.(ts|vue)$/, '')
      if (under(normalized, 'src/api') && !allowed(file, 'component-imports-api', target)) {
        violations.push(`${relative}: 组件禁止依赖 api → ${spec}`)
      }
      if (normalized === 'src/utils/authStorage' && !allowed(file, 'component-imports-api', target)) {
        violations.push(`${relative}: 组件禁止依赖认证存储 → ${spec}`)
      }
      if (!relative.startsWith('src/pages/') && under(normalized, 'src/pages')) {
        violations.push(`${relative}: 全局组件禁止依赖页面层 → ${spec}`)
      }
    }
  }

  if (domain) {
    for (const { spec, target } of resolvedImports) {
      const other = PAGE_DIR_RE.exec(target)
      if (other && other[1] !== domain && !allowed(file, 'cross-page-import', target)) {
        violations.push(`${relative}: 页面间禁止互引（共享逻辑放 src/utils、共享组件进 src/components）→ ${spec}`)
      }
    }
  }

  if (relative.startsWith('src/api/')) {
    for (const { spec, target } of resolvedImports) {
      // api 内部互相引用（同层模块）合法；其余只允许 utils 白名单、config、types。
      const normalized = target.replace(/\.(ts|vue)$/, '')
      const ok = under(normalized, 'src/api')
        || ['src/utils/request', 'src/utils/apiError', 'src/utils/authStorage', 'src/utils/wechatCloud'].includes(normalized)
        || under(normalized, 'src/config')
        || under(normalized, 'src/types')
      if (!ok && !allowed(file, 'api-extra-deps', target)) {
        violations.push(`${relative}: api 层只能依赖 request/config/types/utils → ${spec}`)
      }
    }
  }

  if (usesUniRequest) {
    const permitted = relative === 'src/utils/request.ts' || allowed(file, 'uni-request', null)
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

console.log('✅ 依赖边界检查通过（组件↛api/页面、页面不互引、api 依赖受限、uni.request 收口）')

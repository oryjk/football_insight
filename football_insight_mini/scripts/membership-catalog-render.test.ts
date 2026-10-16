import { expect, test } from 'bun:test'
import { readFileSync } from 'node:fs'
import { createRequire } from 'node:module'
import { parse, compileScript } from '@vue/compiler-sfc'
import { computed, createSSRApp, ref } from 'vue'
import { buildMembershipPurchaseAgreement, formatMembershipDuration, formatMembershipTierLabel } from '../src/pages/membership-purchase/helpers'
import { renderToString } from '@vue/server-renderer'
import ts from 'typescript'

// Compile the real uni-app display SFC; only view/text tags are treated as native elements.
const filename = new URL('../src/pages/membership-purchase/components/MembershipTierList.vue', import.meta.url)
const descriptor = parse(readFileSync(filename, 'utf8'), { filename: filename.pathname }).descriptor
const compiled = compileScript(descriptor, { id: 'membership-tier-list', inlineTemplate: true, templateOptions: { compilerOptions: { isCustomElement: tag => tag === 'view' || tag === 'text' } } })
const js = ts.transpileModule(compiled.content, { compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2020 } }).outputText
const exports: any = {}
new Function('require', 'exports', js)(createRequire(filename), exports)
const Component = exports.default
const annual = { target_tier: 'V6', price_cents: 3900, title: 'V6 回流速览会员', subtitle: '查看 30 分钟内', description: '年度权益' }
const pass = { target_tier: 'V4', price_cents: 900, title: 'V4 场次体验卡', subtitle: '次日 23:59 前有效', description: '场次权益', membership_duration_months: 1, expiry_rule: 'end_of_next_day', duration_label: '次日 23:59 前有效', group_key: 'pass', is_pass: true }

test('mixed catalog renders named groups and accurate pass expiry while preserving tier-key selection', async () => {
  const props = { options: [pass, { ...annual, group_key: 'annual' }], selectedTier: 'V4', summaries: {} }
  const html = await renderToString(createSSRApp(Component, props))
  expect(html).toContain('场次体验卡')
  expect(html).toContain('年度会员')
  expect(html).toContain('次日 23:59 前有效')
  expect(html).not.toContain('1 个月')
  expect(html).toContain('tier-row--active')
  const events: any[] = []
  const render = Component.setup(props, { expose() {}, emit: (...event: any[]) => { events.push(event) } })
  const root = render(props, [])
  function tapSelected(node: any): boolean {
    if (!node || typeof node !== 'object') return false
    if (node.props?.onTap && String(node.props.class).includes('tier-row--active')) { node.props.onTap(); return true }
    return Array.isArray(node.children) && node.children.some(tapSelected)
  }
  expect(tapSelected(root)).toBe(true)
  expect(events).toEqual([['select', 'V4']])
})

test('old backend response keeps the existing annual row without new headings', async () => {
  const html = await renderToString(createSSRApp(Component, { options: [annual], selectedTier: 'V6', summaries: {} }))
  expect(html).toContain('一年')
  expect(html).toContain('查看 30 分钟内')
  expect(html).toContain('>39</text>')
  expect(html).not.toContain('年度会员')
  expect(html).not.toContain('场次体验卡')
  expect(html).not.toContain('tier-group__title')
})


test('purchase page wires selected catalog metadata into the caption and agreement', () => {
  const source = readFileSync(new URL('../src/pages/membership-purchase/index.vue', import.meta.url), 'utf8').match(/<script setup lang="ts">([\s\S]*?)<\/script>/)![1]
  const parsed = ts.createSourceFile('purchase.ts', source, ts.ScriptTarget.Latest, true)
  const declarations = parsed.statements.filter(statement => ts.isVariableStatement(statement) && statement.declarationList.declarations.some(declaration => ['agreementPoints', 'purchaseCaption'].includes(declaration.name.getText(parsed))))
  const js = ts.transpileModule(declarations.map(statement => statement.getText(parsed)).join('\n'), { compilerOptions: { target: ts.ScriptTarget.ES2020 } }).outputText
  function copy(option: any) {
    const scope = { selectedProduct: ref(option), selectedUpgradeNote: ref(''), computed, buildMembershipPurchaseAgreement, formatMembershipDuration, formatMembershipTierLabel }
    return new Function(...Object.keys(scope), `${js}; return { caption: purchaseCaption.value, agreement: agreementPoints.value }`)(...Object.values(scope))
  }
  const day = copy(pass)
  expect(day.caption).toBe('V4 · 次日 23:59 前有效')
  expect(day.agreement[0]).toContain('有效期至购买次日 23:59（北京时间）')
  expect(day.agreement.join('')).not.toContain('一年')
  const legacy = copy(annual)
  expect(legacy.caption).toBe('V6 · 一年')
  expect(legacy.agreement[0]).toBe('会员服务自支付成功并完成系统确认之时起生效；在服务器正常运营、相关服务未因不可抗力或监管要求中止的情况下，有效期为一年。')
})

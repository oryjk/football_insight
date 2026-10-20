import { expect, test } from 'bun:test'
import { readFileSync } from 'node:fs'
import { join } from 'node:path'

// 长列表里的球员/作者头像必须延迟加载：提名弹层一次渲染约 500 名候选人，
// 不加 lazy-load 时所有头像同时开始下载，弹层打开很慢（头像服务器单张首字节约 1–2 秒）。
// 球场圆点、共识卡片这类固定 11 人的地方不在此列。
const SRC = join(import.meta.dir, '../src')
const LONG_LISTS: Array<[string, string]> = [
  ['pages/transfer/components/TransferNominateSheet.vue', 'nominate__avatar'],
  ['pages/lineup/components/LineupPlayerSheet.vue', 'row__avatar'],
  ['components/FiTransferCard.vue', 'transfer-card__avatar'],
  ['components/FiLineupCommunityCard.vue', 'avatar'],
]

test('avatars in long lists load lazily', () => {
  const missing = LONG_LISTS.filter(([file, className]) => {
    const images = [...readFileSync(join(SRC, file), 'utf8').matchAll(/<image\b[^>]*>/g)].map(m => m[0])
    const avatar = images.find(tag => new RegExp(`class="[^"]*\\b${className}\\b`).test(tag))
    return !avatar || !/\blazy-load\b/.test(avatar)
  }).map(([file]) => file)
  expect(missing).toEqual([])
})

import { expect, test } from 'bun:test'
import { readFileSync } from 'node:fs'
import { join } from 'node:path'

// 长列表里的球员头像必须延迟加载：提名弹层一次渲染约 500 名候选人，
// 不加 lazy 时所有头像同时开始下载，弹层打开很慢（头像服务器单张首字节约 1–2 秒）。
// 球员头像统一走 FiAvatar 组件，这里检查每个 <FiAvatar> 标签都带 lazy。
// FiLineupCommunityCard 是作者的微信头像，不在球员目录，不换 FiAvatar，仍用 <image lazy-load>，
// 因此不再列入本清单。
const SRC = join(import.meta.dir, '../src')
const LONG_LISTS: Array<[string, string]> = [
  ['pages/transfer/components/TransferNominateSheet.vue', 'nominate__avatar'],
  ['pages/lineup/components/LineupPlayerSheet.vue', 'row__avatar'],
  ['components/FiTransferCard.vue', 'transfer-card__avatar'],
]

test('avatars in long lists load lazily', () => {
  const missing = LONG_LISTS.filter(([file]) => {
    const tags = [...readFileSync(join(SRC, file), 'utf8').matchAll(/<FiAvatar\b[^>]*>/g)].map(m => m[0])
    return tags.length === 0 || tags.some(tag => !/\blazy\b/.test(tag))
  }).map(([file]) => file)
  expect(missing).toEqual([])
})

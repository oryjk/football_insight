import { expect, test } from 'bun:test'
import { existsSync, readFileSync } from 'node:fs'
import { join } from 'node:path'

const SRC = join(import.meta.dir, '../src')
const read = (path: string) => readFileSync(join(SRC, path), 'utf8')

test('the reaction capsule is a shared, generically named component', () => {
  expect(existsSync(join(SRC, 'components/FiReactionBar.vue'))).toBe(true)
  expect(existsSync(join(SRC, 'components/FiLineupReactionBar.vue'))).toBe(false)
  for (const file of ['components/FiLineupCommunityCard.vue', 'pages/lineup/view.vue']) {
    expect(read(file).includes('FiLineupReactionBar')).toBe(false)
    expect(read(file).includes('FiReactionBar')).toBe(true)
  }
})

test('dislike count is opt-in and rendered next to the dislike icon', () => {
  const source = read('components/FiReactionBar.vue')
  expect(/dislikeCount\?: number/.test(source)).toBe(true)
  expect(/v-if="dislikeCount !== undefined"[^>]*>\{\{ dislikeCount \}\}/.test(source)).toBe(true)
  // 战术板不传踩数，外观不变
  expect(read('components/FiLineupCommunityCard.vue').includes('dislike-count')).toBe(false)
})

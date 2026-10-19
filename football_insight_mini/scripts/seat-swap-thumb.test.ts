import { describe, expect, test } from 'bun:test'
import { existsSync, readFileSync } from 'node:fs'
import { buildSeatSwapThumbSvg } from '../src/pages/seat-swap/helpers'
import { SEAT_SWAP_THUMB_PATH } from './generate-seat-swap-thumb'

describe('seat swap thumbnail asset', () => {
  // 分区布局变化后需重新运行 bun scripts/generate-seat-swap-thumb.ts。
  test('committed stadium-thumb.svg matches the region layout', () => {
    expect(existsSync(SEAT_SWAP_THUMB_PATH)).toBe(true)
    expect(readFileSync(SEAT_SWAP_THUMB_PATH, 'utf8')).toBe(buildSeatSwapThumbSvg())
  })
})

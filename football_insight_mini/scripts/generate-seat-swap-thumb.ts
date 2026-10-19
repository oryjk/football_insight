// 生成换座卡缩略图底图：bun scripts/generate-seat-swap-thumb.ts
import { writeFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { buildSeatSwapThumbSvg } from '../src/pages/seat-swap/helpers'

export const SEAT_SWAP_THUMB_PATH = fileURLToPath(new URL('../src/static/seat-swap/stadium-thumb.svg', import.meta.url))

if (import.meta.main) {
  writeFileSync(SEAT_SWAP_THUMB_PATH, buildSeatSwapThumbSvg())
  console.log(`wrote ${SEAT_SWAP_THUMB_PATH}`)
}

import { describe, expect, test } from 'bun:test'
import { nextTick, reactive } from 'vue'
import {
  DOT_BOX_EXTRA_HEIGHT_RPX, DOT_BOX_PADDING_RPX, DOT_DIAMETER_RPX,
  boxCentreRpx, boxPositionPx, useDragRebind,
} from './dragGeometry'

const WINDOW = 375 // px，1px = 2rpx
const FIELD = { width: 351, height: 468 } // 702rpx × 936rpx，3:4
const boxW = ((DOT_DIAMETER_RPX + DOT_BOX_PADDING_RPX) * WINDOW) / 750
const boxH = ((DOT_DIAMETER_RPX + DOT_BOX_EXTRA_HEIGHT_RPX) * WINDOW) / 750
const dot = (DOT_DIAMETER_RPX * WINDOW) / 750

describe('boxPositionPx', () => {
  test('puts the dot centre on the slot point and the box left edge half a box to the left', () => {
    const pos = boxPositionPx({ x: 0.5, y: 0.5 }, FIELD, WINDOW)
    expect(Math.abs(pos.x - (FIELD.width / 2 - boxW / 2)) < 1e-9).toBe(true)
    expect(Math.abs(pos.y - (FIELD.height / 2 - dot / 2)) < 1e-9).toBe(true)
  })

  test('wing-backs at x 0.06 / 0.94 are clamped inside the movable area, not placed at a negative offset', () => {
    const left = boxPositionPx({ x: 0.06, y: 0.72 }, FIELD, WINDOW)
    const right = boxPositionPx({ x: 0.94, y: 0.72 }, FIELD, WINDOW)
    expect(left.x).toBe(0)
    expect(Math.abs(right.x - (FIELD.width - boxW)) < 1e-9).toBe(true)
  })

  test('clamps vertically too', () => {
    expect(boxPositionPx({ x: 0.5, y: 0.0 }, FIELD, WINDOW).y).toBe(0)
    expect(Math.abs(boxPositionPx({ x: 0.5, y: 1.0 }, FIELD, WINDOW).y - (FIELD.height - boxH)) < 1e-9).toBe(true)
  })
})

describe('boxCentreRpx', () => {
  test('converts a box position back to the dot centre in rpx', () => {
    const centre = boxCentreRpx({ x: 10, y: 20 }, WINDOW)
    expect(Math.abs(centre.x - (10 + boxW / 2) * 2) < 1e-9).toBe(true)
    expect(Math.abs(centre.y - (20 + dot / 2) * 2) < 1e-9).toBe(true)
  })

  test('a clamped wing-back is measured from where it is really drawn, so a still tap stays a tap', () => {
    const drawn = boxPositionPx({ x: 0.06, y: 0.72 }, FIELD, WINDOW)
    const from = boxCentreRpx(drawn, WINDOW)
    const to = boxCentreRpx({ x: drawn.x + 2, y: drawn.y + 1 }, WINDOW) // 手指轻微抖动
    expect(Math.hypot(to.x - from.x, to.y - from.y) < 10).toBe(true)
  })
})

describe('useDragRebind', () => {
  test('shows the committed positions until a drag is released', () => {
    const target = reactive<Record<number, { x: number; y: number }>>({ 1: { x: 10, y: 10 } })
    const { bound } = useDragRebind(() => target)
    expect(bound.value[1]).toEqual({ x: 10, y: 10 })
  })

  test('after a release the binding is first forced to the dragged position, then returns to the committed one', async () => {
    const target = reactive<Record<number, { x: number; y: number }>>({ 1: { x: 10, y: 10 } })
    const { bound, release } = useDragRebind(() => target)
    const pending = release(1, { x: 80, y: 90 })
    // 先绑定到手指放开的位置：与上一次绑定值不同，小程序才会真的下发 setData
    expect(bound.value[1]).toEqual({ x: 80, y: 90 })
    await pending
    await nextTick()
    // 再回到已提交位置：互换时被拖的点位回到自己原来的坐标，而不是停在松手处
    expect(bound.value[1]).toEqual({ x: 10, y: 10 })
  })

  test('a release for one slot never affects the others', async () => {
    const target = reactive<Record<number, { x: number; y: number }>>({ 1: { x: 1, y: 1 }, 2: { x: 2, y: 2 } })
    const { bound, release } = useDragRebind(() => target)
    const pending = release(1, { x: 50, y: 50 })
    expect(bound.value[2]).toEqual({ x: 2, y: 2 })
    await pending
  })

  test('follows target changes after the release finished', async () => {
    const target = reactive<Record<number, { x: number; y: number }>>({ 1: { x: 10, y: 10 } })
    const { bound, release } = useDragRebind(() => target)
    await release(1, { x: 80, y: 90 })
    target[1] = { x: 33, y: 44 }
    expect(bound.value[1]).toEqual({ x: 33, y: 44 })
  })
})

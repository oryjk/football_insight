import { computed, nextTick, reactive } from 'vue'

/**
 * 球场拖动的几何：圆点尺寸与 --fi-component-lineup-dot-size 一致；movable-view 容器 = 圆点 + 下方姓名行。
 * movable-view 的 x/y 与 createSelectorQuery 只接受 px，所以这里的位置用 px，对外判定用 rpx。
 */
export const DOT_DIAMETER_RPX = 84
export const DOT_BOX_PADDING_RPX = 32 // 容器比圆点宽出的部分（容纳姓名），与 --fi-space-32 一致
export const DOT_BOX_EXTRA_HEIGHT_RPX = 40 // 容器比圆点高出的部分（姓名行），与 --fi-space-40 一致

export interface Px {
  x: number
  y: number
}

const toPx = (rpx: number, windowWidthPx: number) => (rpx * windowWidthPx) / 750
const toRpx = (px: number, windowWidthPx: number) => (px * 750) / windowWidthPx

/**
 * 某个位置在球场里应绑定的 movable-view 左上角（px）：圆点中心落在位置点上。
 * 必须钳制在 movable-area 内，否则贴边的位置（如 5-3-2 的翼卫）会算出负数，原生又把它拉回 0，
 * 按下时记录的起点与实际画出的位置不一致，轻点就被判成拖动。
 */
export function boxPositionPx(
  entry: { x: number; y: number },
  field: { width: number; height: number },
  windowWidthPx: number,
): Px {
  const boxW = toPx(DOT_DIAMETER_RPX + DOT_BOX_PADDING_RPX, windowWidthPx)
  const boxH = toPx(DOT_DIAMETER_RPX + DOT_BOX_EXTRA_HEIGHT_RPX, windowWidthPx)
  const dot = toPx(DOT_DIAMETER_RPX, windowWidthPx)
  const x = entry.x * field.width - boxW / 2
  const y = entry.y * field.height - dot / 2
  return {
    x: Math.min(Math.max(0, x), Math.max(0, field.width - boxW)),
    y: Math.min(Math.max(0, y), Math.max(0, field.height - boxH)),
  }
}

/** movable-view 左上角（px）对应的圆点中心（rpx，相对球场左上角）。 */
export function boxCentreRpx(position: Px, windowWidthPx: number): Px {
  const boxW = toPx(DOT_DIAMETER_RPX + DOT_BOX_PADDING_RPX, windowWidthPx)
  const dot = toPx(DOT_DIAMETER_RPX, windowWidthPx)
  return {
    x: toRpx(position.x + boxW / 2, windowWidthPx),
    y: toRpx(position.y + dot / 2, windowWidthPx),
  }
}

/**
 * 松手后强制重新绑定：互换时被拖的点位提交后的坐标与拖动前相同，绑定值没变，
 * 小程序运行时不会下发 setData，movable-view 就停在松手处。先绑定到松手位置（与上一次绑定值不同），
 * 下一个 tick 再回到已提交位置，原生层才会真的把它移回去（带过渡动画）。
 */
export function useDragRebind(target: () => Record<number, Px>) {
  const override = reactive<Record<number, Px>>({})
  const bound = computed<Record<number, Px>>(() => ({ ...target(), ...override }))

  async function release(slot: number, livePx: Px): Promise<void> {
    override[slot] = livePx
    await nextTick()
    delete override[slot]
  }

  return { bound, release }
}

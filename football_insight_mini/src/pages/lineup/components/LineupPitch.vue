<template>
  <view class="pitch" :class="{ 'pitch--fit': fitContainer }">
    <view id="lineup-pitch-field" class="pitch__field">
      <view
        v-for="stripe in 8"
        :key="stripe"
        class="pitch__stripe"
        :class="{ 'pitch__stripe--alt': stripe % 2 === 0 }"
      />
      <view class="pitch__line pitch__line--outline" />
      <view class="pitch__line pitch__line--half" />
      <view class="pitch__circle" />
      <view class="pitch__box pitch__box--top" />
      <view class="pitch__box pitch__box--bottom" />

      <movable-area v-if="measured" class="pitch__area" :style="areaStyle">
        <movable-view
          v-for="entry in visiblePlayers"
          :key="entry.slot"
          class="pitch__mover"
          direction="all"
          :x="bound[entry.slot]?.x ?? 0"
          :y="bound[entry.slot]?.y ?? 0"
          :inertia="false"
          :out-of-bounds="false"
          :damping="40"
          :disabled="readonly"
          @change="(event: MovableChangeEvent) => onChange(entry.slot, event)"
          @touchstart="() => onStart(entry.slot)"
          @touchend="() => onEnd(entry.slot)"
          @tap="() => onTap(entry.slot)"
        >
          <LineupPlayerDot :player="entry" :selected="selectedSlot === entry.slot" />
        </movable-view>
      </movable-area>
    </view>
  </view>
</template>

<script setup lang="ts">
import { computed, getCurrentInstance, nextTick, onBeforeUnmount, onMounted, reactive, ref, watch } from 'vue'
import { boxCentreRpx, boxPositionPx, DOT_DIAMETER_RPX, useDragRebind, type Px } from '../dragGeometry'
import { resolveDragRelease, type DragOutcome, type LineupSlots } from '../helpers'
import type { PitchPlayer } from '../lineupViewModel'
import { schedulePitchLayoutDiagnostic } from '../layoutDiagnostics'
import LineupPlayerDot from './LineupPlayerDot.vue'

interface MovableChangeEvent {
  detail: { x: number; y: number; source?: string }
}

const props = withDefaults(defineProps<{
  players: PitchPlayer[]
  readonly?: boolean
  fitContainer?: boolean
  selectedSlot?: number | null
}>(), {
  readonly: false,
  fitContainer: false,
  selectedSlot: null,
})

const emit = defineEmits<{
  (e: 'tap', slot: number): void
  (e: 'drag', slot: number, outcome: DragOutcome): void
}>()

const instance = getCurrentInstance()
const measured = ref(false)
const areaReady = ref(false)
const fieldPx = reactive({ width: 0, height: 0 })
const windowWidthPx = ref(375)
const MEASURE_RETRIES = 6
const MEASURE_RETRY_MS = 120
let measureEpoch = 0
let measureTimer: ReturnType<typeof setTimeout> | null = null
let disposed = false
let cancelDiagnostic = () => {}

// 原生区域先以明确的 px 尺寸建立，量到真实布局后才放入球员。
// 与 movable-view 同批创建时，微信可能先按尚未布局的区域把所有 x/y 钳制为 0。
const areaStyle = computed(() => ({ width: `${fieldPx.width}px`, height: `${fieldPx.height}px` }))
const visiblePlayers = computed(() => areaReady.value ? props.players : [])

// 每个位置当前应显示的左上角（px），已钳制在 movable-area 内（见 dragGeometry.boxPositionPx）。
const position = computed<Record<number, Px>>(() => {
  const result: Record<number, Px> = {}
  if (!measured.value) return result
  for (const entry of props.players) {
    result[entry.slot] = boxPositionPx(entry, fieldPx, windowWidthPx.value)
  }
  return result
})

// 松手后先绑定到手指位置、下个 tick 再回到已提交位置，保证互换时被拖的点位真的回到自己的坐标。
const { bound, release } = useDragRebind(() => position.value)

watch([areaReady, position], () => {
  cancelDiagnostic()
  if (!instance || !areaReady.value) return
  cancelDiagnostic = schedulePitchLayoutDiagnostic(instance, () => ({
    expected: props.players.map(({ slot }) => ({ slot, ...bound.value[slot] })),
    field: { ...fieldPx }, windowWidth: windowWidthPx.value,
  }))
})

// 拖动中的实时位置：非响应式，避免把原生拖动的 change 回写成 x/y 而互相打架。
const live: Record<number, Px> = {}
const pressedFrom: Record<number, Px> = {}
let suppressTap = false

function readWindowWidth(): number {
  // 接口存在但缺少 windowWidth 时不会抛错；不能让 undefined 进入 rpx/px 换算。
  // upx2px 与原生 rpx 尺寸使用同一比例，旧接口仅作为最后的 API 兜底。
  const readers = [
    () => uni.getWindowInfo?.().windowWidth,
    () => uni.upx2px(750),
    () => uni.getSystemInfoSync().windowWidth,
  ]
  for (const read of readers) {
    try {
      const width = read()
      if (typeof width === 'number' && Number.isFinite(width) && width > 0) return width
    } catch { /* Continue with the next source. */ }
  }
  return windowWidthPx.value
}

function currentMeasure(epoch: number): boolean {
  return !disposed && epoch === measureEpoch
}

function retryMeasure(epoch: number, action: () => void): void {
  if (!currentMeasure(epoch)) return
  if (measureTimer) clearTimeout(measureTimer)
  measureTimer = setTimeout(() => {
    measureTimer = null
    if (currentMeasure(epoch)) action()
  }, MEASURE_RETRY_MS)
}

function measureArea(epoch: number, attempt = 0): void {
  if (!instance || !currentMeasure(epoch)) return
  uni.createSelectorQuery().in(instance).select('.pitch__area').boundingClientRect((rect) => {
    if (!currentMeasure(epoch)) return
    const box = Array.isArray(rect) ? rect[0] : rect
    if (box && box.width && box.height
      && Math.abs(box.width - fieldPx.width) < 1 && Math.abs(box.height - fieldPx.height) < 1) {
      areaReady.value = true
      return
    }
    if (attempt < MEASURE_RETRIES) retryMeasure(epoch, () => measureArea(epoch, attempt + 1))
  }).exec()
}

// 外层有尺寸不代表原生 movable-area 已完成布局，分两阶段测量。
function measure(attempt = 0, epoch = ++measureEpoch): void {
  if (!currentMeasure(epoch)) return
  windowWidthPx.value = readWindowWidth()
  if (!instance) return
  uni.createSelectorQuery()
    .in(instance)
    .select('#lineup-pitch-field')
    .boundingClientRect((rect) => {
      if (!currentMeasure(epoch)) return
      const box = Array.isArray(rect) ? rect[0] : rect
      if (box && box.width && box.height) {
        if (areaReady.value && fieldPx.width === box.width && fieldPx.height === box.height) return
        areaReady.value = false
        fieldPx.width = box.width
        fieldPx.height = box.height
        measured.value = true
        void nextTick(() => measureArea(epoch))
        return
      }
      if (attempt < MEASURE_RETRIES) retryMeasure(epoch, () => measure(attempt + 1, epoch))
    })
    .exec()
}

const onResize = () => measure()

onMounted(() => {
  void nextTick(() => measure())
  uni.onWindowResize?.(onResize)
})
onBeforeUnmount(() => {
  disposed = true
  cancelDiagnostic()
  measureEpoch += 1
  if (measureTimer) clearTimeout(measureTimer)
  uni.offWindowResize?.(onResize)
})
watch(() => props.players.length, () => void nextTick(() => measure()))

function onStart(slot: number): void {
  const origin = position.value[slot]
  if (!origin) return
  pressedFrom[slot] = { ...origin }
  live[slot] = { ...origin }
}

function onChange(slot: number, event: MovableChangeEvent): void {
  // 只记录用户手指拖动产生的位置；程序设置 x/y 引起的变化忽略。
  if (event.detail.source === 'touch' || event.detail.source === 'touch-out-of-bounds') {
    live[slot] = { x: event.detail.x, y: event.detail.y }
  }
}

function onEnd(slot: number): void {
  if (props.readonly) return
  const from = pressedFrom[slot]
  const to = live[slot]
  delete pressedFrom[slot]
  delete live[slot]
  if (!from || !to) return

  const slots: LineupSlots = props.players.map((p) => ({
    slot: p.slot,
    player_id: p.player ? p.player.player_id : null,
    x: p.x,
    y: p.y,
  }))
  const rpxPerPx = 750 / windowWidthPx.value
  const a = boxCentreRpx(from, windowWidthPx.value)
  const b = boxCentreRpx(to, windowWidthPx.value)
  const outcome = resolveDragRelease({
    slot,
    fromX: a.x,
    fromY: a.y,
    toX: b.x,
    toY: b.y,
    areaWidthRpx: fieldPx.width * rpxPerPx,
    areaHeightRpx: fieldPx.height * rpxPerPx,
    dotDiameterRpx: DOT_DIAMETER_RPX,
    slots,
  })
  // 无论结果如何都强制重新绑定：轻点（位移 < 阈值）时手指的微小偏移要被拉回原位，互换时被拖的点位要回到自己的坐标。
  void release(slot, to)
  if (outcome.kind === 'tap') return
  // movable-view 在拖动后仍可能触发 tap，用标记吞掉紧随其后的那一次，避免松手后又弹出选人弹层。
  suppressTap = true
  setTimeout(() => { suppressTap = false }, 0)
  emit('drag', slot, outcome)
}

function onTap(slot: number): void {
  if (props.readonly || suppressTap) return
  emit('tap', slot)
}
</script>

<style scoped>
/* 球场宽高比 3:4：用 padding-top 百分比撑出高度，内容层绝对定位。 */
.pitch {
  width: 100%;
}

.pitch__field {
  position: relative;
  width: 100%;
  height: 0;
  padding-top: var(--fi-component-lineup-pitch-ratio);
  overflow: hidden;
  border-radius: var(--fi-radius-md);
  background: var(--fi-color-lineup-grass);
}

/* The editor owns the available viewport slot; public details keep the normal ratio. */
.pitch--fit {
  position: absolute;
  top: 0;
  left: 0;
  height: 100%;
}

.pitch--fit .pitch__field {
  height: 100%;
  padding-top: 0;
}

.pitch__stripe {
  position: absolute;
  left: 0;
  right: 0;
  height: 12.5%;
  background: var(--fi-color-lineup-grass);
}

.pitch__stripe--alt {
  background: var(--fi-color-lineup-grass-alt);
}

.pitch__stripe:nth-child(1) { top: 0; }
.pitch__stripe:nth-child(2) { top: 12.5%; }
.pitch__stripe:nth-child(3) { top: 25%; }
.pitch__stripe:nth-child(4) { top: 37.5%; }
.pitch__stripe:nth-child(5) { top: 50%; }
.pitch__stripe:nth-child(6) { top: 62.5%; }
.pitch__stripe:nth-child(7) { top: 75%; }
.pitch__stripe:nth-child(8) { top: 87.5%; }

.pitch__line {
  position: absolute;
  border: var(--fi-border-width) solid var(--fi-color-lineup-markings);
}

.pitch__line--outline {
  top: 2%;
  right: 3%;
  bottom: 2%;
  left: 3%;
}

.pitch__line--half {
  top: 50%;
  right: 3%;
  left: 3%;
  height: 0;
  border-width: var(--fi-border-width) 0 0;
}

.pitch__circle {
  position: absolute;
  top: 50%;
  left: 50%;
  width: 24%;
  height: 18%;
  border: var(--fi-border-width) solid var(--fi-color-lineup-markings);
  border-radius: var(--fi-radius-round);
  transform: translate(-50%, -50%);
}

.pitch__box {
  position: absolute;
  left: 25%;
  width: 50%;
  height: 14%;
  border: var(--fi-border-width) solid var(--fi-color-lineup-markings);
}

.pitch__box--top {
  top: 2%;
  border-top: 0;
}

.pitch__box--bottom {
  bottom: 2%;
  border-bottom: 0;
}

/* movable-area 必须显式写宽高：微信默认 10px，只靠 top/right/bottom/left 拉伸不会生效，
   可移动区域缩成 10×10 时所有圆点都会被限制在左上角叠在一起。 */
.pitch__area {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
}

.pitch__mover {
  display: flex;
  align-items: flex-start;
  justify-content: center;
  width: calc(var(--fi-component-lineup-dot-size) + var(--fi-space-32));
  height: calc(var(--fi-component-lineup-dot-size) + var(--fi-space-40));
}
</style>

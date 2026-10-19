<template>
  <view class="pitch">
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

      <movable-area v-if="measured" class="pitch__area">
        <movable-view
          v-for="entry in players"
          :key="entry.slot"
          class="pitch__mover"
          direction="all"
          :x="position[entry.slot]?.x ?? 0"
          :y="position[entry.slot]?.y ?? 0"
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
import { computed, getCurrentInstance, nextTick, onMounted, reactive, ref, watch } from 'vue'
import { resolveDragRelease, type DragOutcome, type LineupSlots } from '../helpers'
import type { PitchPlayer } from '../lineupViewModel'
import LineupPlayerDot from './LineupPlayerDot.vue'

interface MovableChangeEvent {
  detail: { x: number; y: number; source?: string }
}

const props = withDefaults(defineProps<{
  players: PitchPlayer[]
  readonly?: boolean
  selectedSlot?: number | null
}>(), {
  readonly: false,
  selectedSlot: null,
})

const emit = defineEmits<{
  (e: 'tap', slot: number): void
  (e: 'drag', slot: number, outcome: DragOutcome): void
}>()

// 圆点直径（rpx），与 --fi-component-lineup-dot-size 保持一致；movable-view 的 x/y 与
// createSelectorQuery 只接受 px，所以内部用 px 计算，对外的判定（resolveDragRelease）统一用 rpx。
const DOT_DIAMETER_RPX = 84
const DOT_BOX_PADDING_RPX = 32 // 圆点下方姓名所需的额外宽度，与 .dot 的宽度一致

const instance = getCurrentInstance()
const measured = ref(false)
const fieldPx = reactive({ width: 0, height: 0 })
let windowWidthPx = 375

const rpxPerPx = computed(() => 750 / windowWidthPx)
const dotPx = computed(() => (DOT_DIAMETER_RPX * windowWidthPx) / 750)
const boxPx = computed(() => ((DOT_DIAMETER_RPX + DOT_BOX_PADDING_RPX) * windowWidthPx) / 750)

// 每个位置当前应显示的左上角（px）：圆点中心减去半个容器。
const position = computed<Record<number, { x: number; y: number }>>(() => {
  const result: Record<number, { x: number; y: number }> = {}
  for (const entry of props.players) {
    result[entry.slot] = {
      x: entry.x * fieldPx.width - boxPx.value / 2,
      // 容器高度 = 圆点 + 姓名；纵向以圆点中心对齐位置点
      y: entry.y * fieldPx.height - dotPx.value / 2,
    }
  }
  return result
})

// 拖动中的实时位置：非响应式，避免把原生拖动的 change 回写成 x/y 而互相打架。
const live: Record<number, { x: number; y: number }> = {}
const pressedFrom: Record<number, { x: number; y: number }> = {}
let suppressTap = false

function windowWidth(): number {
  try {
    return uni.getWindowInfo ? uni.getWindowInfo().windowWidth : uni.getSystemInfoSync().windowWidth
  } catch {
    return 375
  }
}

function measure(): void {
  windowWidthPx = windowWidth()
  if (!instance) return
  uni.createSelectorQuery()
    .in(instance)
    .select('#lineup-pitch-field')
    .boundingClientRect((rect) => {
      const box = Array.isArray(rect) ? rect[0] : rect
      if (!box || !box.width || !box.height) return
      fieldPx.width = box.width
      fieldPx.height = box.height
      measured.value = true
    })
    .exec()
}

onMounted(() => {
  void nextTick(measure)
})
watch(() => props.players.length, () => void nextTick(measure))

function centreRpx(slot: number, point: { x: number; y: number }): { x: number; y: number } {
  return {
    x: (point.x + boxPx.value / 2) * rpxPerPx.value,
    y: (point.y + dotPx.value / 2) * rpxPerPx.value,
  }
}

function onStart(slot: number): void {
  const origin = position.value[slot]
  if (!origin) return
  pressedFrom[slot] = { ...origin }
  live[slot] = { ...origin }
}

function onChange(slot: number, event: MovableChangeEvent): void {
  // 只记录用户手指拖动产生的位置；程序设置 x/y 引起的变化（source 为空或 'touch-out-of-bounds' 之外）忽略。
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
  const a = centreRpx(slot, from)
  const b = centreRpx(slot, to)
  const outcome = resolveDragRelease({
    slot,
    fromX: a.x,
    fromY: a.y,
    toX: b.x,
    toY: b.y,
    areaWidthRpx: fieldPx.width * rpxPerPx.value,
    areaHeightRpx: fieldPx.height * rpxPerPx.value,
    dotDiameterRpx: DOT_DIAMETER_RPX,
    slots,
  })
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

.pitch__area {
  position: absolute;
  top: 0;
  right: 0;
  bottom: 0;
  left: 0;
}

.pitch__mover {
  display: flex;
  align-items: flex-start;
  justify-content: center;
  width: calc(var(--fi-component-lineup-dot-size) + var(--fi-space-32));
  height: calc(var(--fi-component-lineup-dot-size) + var(--fi-space-40));
}
</style>

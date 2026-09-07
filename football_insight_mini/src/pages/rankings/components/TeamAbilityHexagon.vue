<template>
  <view class="ability-hex" :style="sizeStyle" :aria-label="ariaLabel">
    <view class="ability-hex__grid ability-hex__grid--outer"></view>
    <view class="ability-hex__grid ability-hex__grid--mid"></view>
    <view class="ability-hex__shape" :style="{ clipPath: dataClipPath }"></view>
    <view class="ability-hex__core"></view>
    <template v-if="showLabels">
      <text
        v-for="(label, index) in AXIS_LABELS"
        :key="label"
        class="ability-hex__axis-label"
        :class="`ability-hex__axis-label--${index + 1}`"
      >{{ label }}</text>
    </template>
  </view>
</template>

<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(
  defineProps<{
    // 6 个维度的归一化数值（0~1），顺序：进攻、射门、组织、传球、防守、对抗
    values: number[]
    // 整体边长，单位 rpx
    size?: number
    // 是否在六个顶点外侧展示维度名
    showLabels?: boolean
  }>(),
  {
    size: 104,
    showLabels: false,
  },
)

const AXIS_LABELS = ['进攻', '射门', '组织', '传球', '防守', '对抗']
// 从正上方开始顺时针六个顶点的角度
const AXIS_ANGLES = [-90, -30, 30, 90, 150, 210]

const sizeStyle = computed(() => ({
  width: `${props.size}rpx`,
  height: `${props.size}rpx`,
}))

const dataClipPath = computed(() => {
  const points = AXIS_ANGLES.map((deg, index) => {
    // 留一个下限，避免某项为 0 时多边形塌缩到中心不可见；
    // 满分（1.0）恰好顶到外框内边，与细描边重合。
    const value = Math.min(1, Math.max(0.12, props.values[index] ?? 0))
    const rad = (deg * Math.PI) / 180
    const x = 50 + 46 * value * Math.cos(rad)
    const y = 50 + 46 * value * Math.sin(rad)
    return `${x.toFixed(1)}% ${y.toFixed(1)}%`
  })

  return `polygon(${points.join(', ')})`
})

const ariaLabel = computed(() =>
  AXIS_LABELS.map((label, index) => `${label} ${Math.round((props.values[index] ?? 0) * 100)}`).join('，'),
)
</script>

<style scoped lang="css">
.ability-hex {
  position: relative;
  width: 104rpx;
  height: 104rpx;
  flex-shrink: 0;
}

.ability-hex__grid,
.ability-hex__shape {
  position: absolute;
  inset: 0;
}

/* 外圈六边形边框：细描边 */
.ability-hex__grid--outer {
  clip-path: polygon(50% 2%, 91.5% 26%, 91.5% 74%, 50% 98%, 8.5% 74%, 8.5% 26%);
  background: rgba(21, 22, 27, 0.45);
}

/* 用略小的面板色六边形盖住外圈中心，形成细描边效果 */
.ability-hex__grid--outer::after {
  content: '';
  position: absolute;
  inset: 0;
  clip-path: polygon(50% 4.5%, 89.2% 27.1%, 89.2% 72.9%, 50% 95.5%, 10.8% 72.9%, 10.8% 27.1%);
  background: rgba(255, 255, 255, 0.96);
}

/* 半圈中位线 */
.ability-hex__grid--mid {
  clip-path: polygon(50% 27.5%, 69.3% 38.7%, 69.3% 61.3%, 50% 72.5%, 30.7% 61.3%, 30.7% 38.7%);
  background: rgba(21, 22, 27, 0.1);
}

.ability-hex__shape {
  background: linear-gradient(160deg, rgba(var(--fi-primitive-red-rgb, 214, 48, 49), 0.62) 0%, rgba(var(--fi-primitive-red-rgb, 214, 48, 49), 0.34) 100%);
}

.ability-hex__core {
  position: absolute;
  left: 50%;
  top: 50%;
  width: 6rpx;
  height: 6rpx;
  margin: -3rpx 0 0 -3rpx;
  border-radius: 50%;
  background: rgba(21, 22, 27, 0.6);
}

/* 顶点维度名标注：与 AXIS_ANGLES 顺序一致（上、右上、右下、下、左下、左上） */
.ability-hex__axis-label {
  position: absolute;
  color: #5c616b;
  font-size: 20rpx;
  font-weight: 600;
  line-height: 1;
  white-space: nowrap;
}

.ability-hex__axis-label--1 {
  left: 50%;
  top: 0;
  transform: translate(-50%, -110%);
}

.ability-hex__axis-label--2 {
  right: 0;
  top: 26%;
  transform: translate(110%, -50%);
}

.ability-hex__axis-label--3 {
  right: 0;
  top: 74%;
  transform: translate(110%, -50%);
}

.ability-hex__axis-label--4 {
  left: 50%;
  bottom: 0;
  transform: translate(-50%, 110%);
}

.ability-hex__axis-label--5 {
  left: 0;
  top: 74%;
  transform: translate(-110%, -50%);
}

.ability-hex__axis-label--6 {
  left: 0;
  top: 26%;
  transform: translate(-110%, -50%);
}
</style>

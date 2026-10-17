<template>
  <view class="fi-segmented" :class="{ 'fi-segmented--compact': compact }" role="group" :aria-label="ariaLabel">
    <view class="fi-segmented__thumb" :style="thumbStyle" />
    <view
      v-for="item in items"
      :key="item.key"
      class="fi-segmented__button"
      :class="{ 'fi-segmented__button--active': item.key === modelValue }"
      role="button"
      :aria-label="item.label"
      :aria-pressed="item.key === modelValue"
      hover-class="fi-segmented__button--pressed"
      :hover-stay-time="80"
      @tap="select(item.key)"
    >
      <image
        v-if="item.icon"
        :src="item.key === modelValue ? (item.iconActive ?? item.icon) : item.icon"
        class="fi-segmented__icon"
        mode="aspectFit"
      />
      <text class="fi-segmented__text">{{ item.label }}</text>
    </view>
  </view>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { FiSegmentedItem } from '../types/segmented'
import { segmentedThumbStyle } from '../utils/segmented'

/**
 * 通用分段切换：带描边的圆角轨道 + 随选中项滑动的实心墨色滑块，任意段数、可选图标。
 * 榜赛页（球队榜/球员榜）与洞察页（进球/助攻/失球）共用，样式只在这里维护一份。
 */
const props = defineProps<{
  items: FiSegmentedItem[]
  modelValue: string
  ariaLabel?: string
  /** 行内小尺寸：用于放在文字行旁边，不撑高所在行。 */
  compact?: boolean
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', value: string): void
}>()

const thumbStyle = computed(() =>
  segmentedThumbStyle(
    props.items.length,
    props.items.findIndex(item => item.key === props.modelValue),
  ),
)

function select(key: string): void {
  if (key !== props.modelValue) emit('update:modelValue', key)
}
</script>

<style scoped lang="css">
.fi-segmented {
  position: relative;
  display: flex;
  width: 100%;
  gap: var(--fi-space-6);
  padding: var(--fi-space-6);
  border: var(--fi-border-width) solid var(--fi-color-border-chip);
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-segmented-track);
  box-sizing: border-box;
}

.fi-segmented__thumb {
  position: absolute;
  top: var(--fi-space-6);
  bottom: var(--fi-space-6);
  left: var(--fi-space-6);
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-text-primary);
  box-shadow: var(--fi-shadow-segmented-thumb);
  transition: transform 300ms cubic-bezier(0.3, 0.9, 0.3, 1);
  pointer-events: none;
}

.fi-segmented__button {
  position: relative;
  z-index: 1;
  flex: 1;
  min-width: 0;
  min-height: var(--fi-space-64);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: var(--fi-space-10);
  padding: 0 var(--fi-space-20);
  border-radius: var(--fi-radius-round);
  color: var(--fi-color-segmented-idle);
  font-size: var(--fi-font-26);
  font-weight: var(--fi-weight-bold);
  line-height: var(--fi-leading-none);
  white-space: nowrap;
  box-sizing: border-box;
  transition: color 260ms ease;
}

.fi-segmented__button--active {
  color: var(--fi-color-surface);
  font-weight: var(--fi-weight-black);
}

.fi-segmented--compact { padding: var(--fi-space-4); gap: var(--fi-space-4); }
.fi-segmented--compact .fi-segmented__thumb { top: var(--fi-space-4); bottom: var(--fi-space-4); left: var(--fi-space-4); }
.fi-segmented--compact .fi-segmented__button { min-height: var(--fi-space-44); padding: 0 var(--fi-space-8); font-size: var(--fi-font-22); }

.fi-segmented__button--pressed {
  opacity: var(--fi-opacity-pressed);
}

.fi-segmented__icon {
  width: var(--fi-icon-sm);
  height: var(--fi-icon-sm);
  flex-shrink: 0;
}

.fi-segmented__text {
  display: block;
  text-align: center;
}

@media (prefers-reduced-motion: reduce) {
  .fi-segmented__thumb,
  .fi-segmented__button {
    transition: none;
  }
}
</style>

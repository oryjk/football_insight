<template>
  <view class="bar">
    <scroll-view scroll-x class="bar__scroll" :scroll-left="scrollLeft" scroll-with-animation>
      <view class="bar__list">
        <button
          v-for="code in FORMATION_CODES"
          :id="`lineup-formation-${code}`"
          :key="code"
          class="chip"
          :class="{ 'chip--active': modelValue === code }"
          hover-class="chip--pressed"
          @tap="emit('update:modelValue', code)"
        >
          {{ code }}
        </button>
      </view>
    </scroll-view>
  </view>
</template>

<script setup lang="ts">
import { getCurrentInstance, nextTick, ref, watch } from 'vue'
import { FORMATION_CODES } from '../formations'

const props = defineProps<{ modelValue: string }>()
const emit = defineEmits<{ (e: 'update:modelValue', value: string): void }>()

const instance = getCurrentInstance()
const scrollLeft = ref(0)

function isRect(value: unknown): value is { left: number; width: number } {
  return !!value && typeof value === 'object'
    && typeof (value as { left?: unknown }).left === 'number'
    && typeof (value as { width?: unknown }).width === 'number'
}

function isOffset(value: unknown): value is { scrollLeft: number } {
  return !!value && typeof value === 'object' && typeof (value as { scrollLeft?: unknown }).scrollLeft === 'number'
}

async function centreSelected(): Promise<void> {
  if (!instance) return
  await nextTick()
  const query = uni.createSelectorQuery().in(instance)
  query.select('.bar__scroll').boundingClientRect()
  query.select('.bar__scroll').scrollOffset(() => {})
  query.select(`#lineup-formation-${props.modelValue}`).boundingClientRect()
  query.exec((result) => {
    const [scrollRect, offset, chipRect] = (result ?? []) as unknown[]
    if (!isRect(scrollRect) || !isOffset(offset) || !isRect(chipRect)) return
    const delta = chipRect.left + chipRect.width / 2 - (scrollRect.left + scrollRect.width / 2)
    const next = Math.max(0, Math.round(offset.scrollLeft + delta))
    if (next !== scrollLeft.value) scrollLeft.value = next
  })
}

watch(() => props.modelValue, () => void centreSelected(), { immediate: true })
</script>

<style scoped>
.bar {
  min-width: 0;
  padding: var(--fi-space-6);
  border: var(--fi-border-width) solid var(--fi-color-border-chip);
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-segmented-track);
  overflow: hidden;
  box-sizing: border-box;
}

.bar__scroll {
  width: 100%;
  min-width: 0;
  white-space: nowrap;
}

.bar__list {
  display: inline-flex;
  gap: var(--fi-space-6);
}

.chip {
  display: inline-flex;
  flex-shrink: 0;
  align-items: center;
  min-height: var(--fi-space-64);
  margin: 0;
  padding: 0 var(--fi-space-28);
  border-radius: var(--fi-radius-round);
  background: transparent;
  color: var(--fi-color-segmented-idle);
  font-size: var(--fi-font-26);
  line-height: var(--fi-leading-none);
  white-space: nowrap;
  transition:
    background-color var(--fi-motion-duration-base) var(--fi-motion-ease-out),
    color var(--fi-motion-duration-base) var(--fi-motion-ease-out);
}

.chip::after {
  border: 0;
}

.chip--active {
  background: var(--fi-color-text-primary);
  box-shadow: var(--fi-shadow-segmented-thumb);
  color: var(--fi-color-surface);
  font-weight: var(--fi-weight-extrabold);
}

.chip--pressed {
  opacity: var(--fi-primitive-opacity-pressed);
}

@media (prefers-reduced-motion: reduce) {
  .chip {
    transition: none;
  }
}
</style>

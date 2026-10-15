<template>
  <view v-if="items.length" class="result-dots" :class="[`result-dots--${size}`, `result-dots--${align}`]">
    <view class="result-dots__row" :class="{ 'result-dots__row--grid': perRow }" :style="perRow ? { '--result-columns': perRow } : undefined" :role="interactive ? undefined : 'img'" :aria-label="interactive ? undefined : summary">
      <template v-for="(item, index) in items" :key="item.id">
        <view
          v-if="interactive"
          class="result-dots__cell"
          role="button"
          :aria-label="item.ariaLabel || RESULT_DOT_LABELS[item.tone]"
          hover-class="result-dots__cell--pressed"
          hover-stay-time="80"
          @tap="emit('select', item)"
        >
          <view class="result-dots__dot" :class="[`result-dots__dot--${item.tone}`, { 'result-dots__dot--latest': highlightLatest && index === items.length - 1 }]" />
        </view>
        <view v-else class="result-dots__cell" aria-hidden="true">
          <view class="result-dots__dot" :class="[`result-dots__dot--${item.tone}`, { 'result-dots__dot--latest': highlightLatest && index === items.length - 1 }]" />
        </view>
      </template>
    </view>
    <text v-if="showLatestLabel" class="result-dots__latest" aria-hidden="true">{{ latestLabel }}</text>
  </view>
  <text v-else-if="emptyText" class="result-dots__empty">{{ emptyText }}</text>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { RESULT_DOT_LABELS, type ResultDotItem } from '../utils/resultDots'

const props = withDefaults(
  defineProps<{
    items: ResultDotItem[]
    /** md: 22rpx dots with tappable cells; sm: compact dots for list rows. */
    size?: 'md' | 'sm'
    /** end keeps short records on the right, with the same spacing as a full record. */
    align?: 'start' | 'end'
    /** Wrap into rows of this many dots (oldest to newest, left to right, top to bottom); omit for a single shrinking row. */
    perRow?: number
    interactive?: boolean
    highlightLatest?: boolean
    showLatestLabel?: boolean
    emptyText?: string
  }>(),
  { size: 'md', align: 'start', perRow: undefined, interactive: false, highlightLatest: false, showLatestLabel: false, emptyText: '' },
)

const emit = defineEmits<{ (event: 'select', item: ResultDotItem): void }>()

const latestLabel = computed(() => RESULT_DOT_LABELS[props.items[props.items.length - 1]!.tone])
const summary = computed(
  () => `共${props.items.length}场，从旧到新：${props.items.map(item => RESULT_DOT_LABELS[item.tone]).join('、')}`,
)
</script>

<style scoped>
.result-dots {
  display: flex;
  align-items: center;
  gap: var(--fi-space-8);
  width: 100%;
  min-width: 0;
  --result-dot-size: var(--fi-space-22);
  --result-cell-height: var(--fi-space-44);
  --result-cell-max: 52rpx;
}
.result-dots--sm {
  --result-dot-size: 14rpx;
  --result-cell-height: var(--fi-space-44);
  --result-cell-max: 28rpx;
}
.result-dots__row {
  display: flex;
  flex: 1;
  flex-wrap: nowrap;
  min-width: 0;
}
.result-dots--end .result-dots__row {
  justify-content: flex-end;
}
.result-dots__row--grid {
  display: grid;
  grid-template-columns: repeat(var(--result-columns), minmax(0, var(--result-cell-max)));
}
.result-dots--sm .result-dots__row--grid {
  --result-cell-height: var(--fi-space-36);
}
.result-dots__cell {
  display: flex;
  align-items: center;
  justify-content: center;
  flex: 1 1 0;
  min-width: 0;
  max-width: var(--result-cell-max);
  height: var(--result-cell-height);
  border-radius: var(--fi-radius-round);
}
.result-dots__cell--pressed {
  background: var(--fi-color-divider-soft);
}
.result-dots__dot {
  --result-dot-color: var(--fi-color-team-form-line);
  width: var(--result-dot-size);
  height: var(--result-dot-size);
  border-radius: var(--fi-radius-round);
  background: var(--result-dot-color);
}
.result-dots__dot--win {
  --result-dot-color: var(--fi-color-primary);
}
.result-dots__dot--loss {
  --result-dot-color: var(--fi-color-success-accent);
}
.result-dots__dot--latest {
  box-shadow: 0 0 0 4rpx var(--fi-color-divider-soft);
}
.result-dots__latest {
  min-width: var(--fi-space-20);
  text-align: right;
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-20);
}
.result-dots__empty {
  display: block;
  padding: var(--fi-space-8) 0;
  font-size: var(--fi-font-20);
  color: var(--fi-color-text-caption);
}
</style>

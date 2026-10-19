<template>
  <view class="picker">
    <scroll-view scroll-x class="picker__scroll" :scroll-left="scrollLeft" scroll-with-animation>
      <view class="picker__list">
        <button
          v-for="team in teams"
          :id="`lineup-team-${team.team_id}`"
          :key="team.team_id"
          class="chip"
          :class="{ 'chip--active': modelValue === team.team_id }"
          hover-class="chip--pressed"
          @tap="emit('update:modelValue', team.team_id)"
        >
          <image v-if="team.avatar_url" :src="team.avatar_url" mode="aspectFit" class="chip__avatar" />
          <view v-else class="chip__avatar chip__avatar--fallback">{{ team.team_name.charAt(0) }}</view>
          <text>{{ team.team_name }}</text>
        </button>
      </view>
    </scroll-view>
  </view>
</template>

<script setup lang="ts">
import { getCurrentInstance, nextTick, ref, watch } from 'vue'

const props = defineProps<{
  teams: Array<{ team_id: number; team_name: string; avatar_url: string | null }>
  modelValue: number
}>()

const emit = defineEmits<{ (e: 'update:modelValue', value: number): void }>()

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

// 选中的球队滚动到胶囊行中间（createSelectorQuery 返回 px，只用于换算滚动位置）。
async function centreSelected(): Promise<void> {
  if (!instance) return
  await nextTick()
  const query = uni.createSelectorQuery().in(instance)
  query.select('.picker__scroll').boundingClientRect()
  query.select('.picker__scroll').scrollOffset(() => {})
  query.select(`#lineup-team-${props.modelValue}`).boundingClientRect()
  query.exec((result) => {
    const [scrollRect, offset, chipRect] = (result ?? []) as unknown[]
    if (!isRect(scrollRect) || !isOffset(offset) || !isRect(chipRect)) return
    const delta = chipRect.left + chipRect.width / 2 - (scrollRect.left + scrollRect.width / 2)
    const next = Math.max(0, Math.round(offset.scrollLeft + delta))
    if (next !== scrollLeft.value) scrollLeft.value = next
  })
}

watch(() => [props.modelValue, props.teams.length], () => void centreSelected(), { immediate: true })
</script>

<style scoped>
.picker {
  min-width: 0;
  padding: var(--fi-space-6);
  border: var(--fi-border-width) solid var(--fi-color-border-chip);
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-segmented-track);
  overflow: hidden;
  box-sizing: border-box;
}

.picker__scroll {
  width: 100%;
  min-width: 0;
  white-space: nowrap;
}

.picker__list {
  display: inline-flex;
  gap: var(--fi-space-6);
}

.chip {
  display: inline-flex;
  flex-shrink: 0;
  align-items: center;
  gap: var(--fi-space-12);
  min-height: var(--fi-space-64);
  margin: 0;
  padding: 0 var(--fi-space-22);
  border-radius: var(--fi-radius-round);
  background: transparent;
  color: var(--fi-color-segmented-idle);
  font-size: var(--fi-font-24);
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

.chip__avatar {
  flex-shrink: 0;
  width: var(--fi-space-36);
  height: var(--fi-space-36);
}

.chip__avatar--fallback {
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-primary);
  color: var(--fi-color-surface);
  font-size: var(--fi-font-22);
  font-weight: var(--fi-weight-bold);
  line-height: var(--fi-leading-none);
}

@media (prefers-reduced-motion: reduce) {
  .chip {
    transition: none;
  }
}
</style>

<template>
  <view class="team-selector">
    <scroll-view
      scroll-x
      class="team-selector__scroll"
      :scroll-left="scrollLeft"
      scroll-with-animation
    >
      <view class="team-selector__list">
        <button
          v-for="team in teams"
          :id="`fi-team-chip-${team.team_id}`"
          :key="team.team_id"
          class="team-chip"
          :class="{ 'team-chip--active': modelValue === team.team_id }"
          hover-class="team-chip--pressed"
          @tap="emit('update:modelValue', team.team_id)"
        >
          <image
            v-if="teamAvatar(team)"
            :src="teamAvatar(team)"
            mode="aspectFit"
            class="team-chip__avatar"
          />
          <view v-else class="team-chip__avatar team-chip__avatar--fallback">
            {{ team.team_name.charAt(0) }}
          </view>
          <text>{{ team.team_name }}</text>
        </button>
      </view>
    </scroll-view>
  </view>
</template>

<script setup lang="ts">
import { getCurrentInstance, nextTick, ref, watch } from 'vue'

export interface FiTeamOption {
  team_id: number
  team_name: string
  avatar_storage_url?: string | null
  avatar_url?: string | null
}

const props = defineProps<{
  teams: FiTeamOption[]
  modelValue: number | null
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', value: number): void
}>()

const instance = getCurrentInstance()
const scrollLeft = ref(0)

function teamAvatar(team: FiTeamOption): string {
  return team.avatar_storage_url || team.avatar_url || ''
}

function hasRectShape(value: unknown): value is { left: number; width: number } {
  return !!value && typeof value === 'object'
    && typeof (value as { left?: unknown }).left === 'number'
    && typeof (value as { width?: unknown }).width === 'number'
}

function hasScrollLeft(value: unknown): value is { scrollLeft: number } {
  return !!value && typeof value === 'object'
    && typeof (value as { scrollLeft?: unknown }).scrollLeft === 'number'
}

async function centerSelectedChip(): Promise<void> {
  if (!instance || props.modelValue === null) {
    return
  }

  await nextTick()

  const query = uni.createSelectorQuery().in(instance)
  query.select('.team-selector__scroll').boundingClientRect()
  query.select('.team-selector__scroll').scrollOffset(() => {})
  query.select(`#fi-team-chip-${props.modelValue}`).boundingClientRect()
  query.exec((result) => {
    const [rawScrollRect, rawScrollOffset, rawChipRect] = (result ?? []) as unknown[]

    if (!hasRectShape(rawScrollRect) || !hasScrollLeft(rawScrollOffset) || !hasRectShape(rawChipRect)) {
      return
    }

    const delta = (rawChipRect.left + rawChipRect.width / 2)
      - (rawScrollRect.left + rawScrollRect.width / 2)
    const nextScrollLeft = Math.max(0, Math.round(rawScrollOffset.scrollLeft + delta))

    if (nextScrollLeft !== scrollLeft.value) {
      scrollLeft.value = nextScrollLeft
    }
  })
}

watch(
  () => [props.modelValue, props.teams.length],
  () => {
    void centerSelectedChip()
  },
  { immediate: true },
)
</script>

<style scoped lang="css">
.team-selector {
  min-width: 0;
  padding: var(--fi-space-6);
  border: var(--fi-border-width) solid var(--fi-color-border-chip);
  border-radius: var(--fi-radius-xs);
  background: var(--fi-color-segmented-track);
  overflow: hidden;
  box-sizing: border-box;
}

.team-selector__scroll {
  width: 100%;
  min-width: 0;
  white-space: nowrap;
}

.team-selector__list {
  display: inline-flex;
  gap: var(--fi-space-6);
}

.team-chip {
  display: inline-flex;
  align-items: center;
  gap: var(--fi-space-12);
  min-height: var(--fi-space-64);
  padding: 0 var(--fi-space-20);
  border-radius: var(--fi-radius-2xs);
  background: transparent;
  color: var(--fi-color-segmented-idle);
  font-size: var(--fi-font-24);
  line-height: var(--fi-leading-none);
  flex-shrink: 0;
  white-space: nowrap;
  margin: 0;
  transition: background-color var(--fi-motion-duration-base) var(--fi-motion-ease-out),
    color var(--fi-motion-duration-base) var(--fi-motion-ease-out);
}

.team-chip::after {
  border: 0;
}

.team-chip--active {
  background: var(--fi-color-text-primary);
  box-shadow: var(--fi-shadow-segmented-thumb);
  color: var(--fi-color-surface);
  font-weight: var(--fi-weight-extrabold);
}

.team-chip--pressed {
  opacity: var(--fi-opacity-pressed);
}

.team-chip__avatar {
  width: var(--fi-space-36);
  height: var(--fi-space-36);
  flex-shrink: 0;
}

.team-chip__avatar--fallback {
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
  .team-chip {
    transition: none;
  }
}
</style>

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
          :id="`team-chip-${team.team_id}`"
          :key="team.team_id"
          class="team-chip"
          :class="{ active: selectedTeamId === team.team_id }"
          hover-class="team-chip--pressed"
          @click="emit('update:selectedTeamId', team.team_id)"
        >
          <image
            v-if="team.avatar_storage_url"
            :src="team.avatar_storage_url"
            mode="aspectFit"
            class="team-chip__avatar"
          />
          <view v-else class="team-chip__avatar team-chip__avatar--fallback">
            {{ team.team_name.charAt(0) }}
          </view>
          <text>#{{ team.rank_no }} {{ team.team_name }}</text>
        </button>
      </view>
    </scroll-view>
  </view>
</template>

<script setup lang="ts">
import { getCurrentInstance, nextTick, ref, watch } from 'vue'
import type { TeamInsightTeam } from '../../../types/insight'

const props = defineProps<{
  teams: TeamInsightTeam[]
  selectedTeamId: number | null
}>()

const emit = defineEmits<{
  (e: 'update:selectedTeamId', value: number): void
}>()

const instance = getCurrentInstance()
const scrollLeft = ref(0)

watch(
  () => props.selectedTeamId,
  () => {
    void centerSelectedChip()
  },
  { immediate: true },
)

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
  if (!instance || props.selectedTeamId === null) {
    return
  }

  await nextTick()

  const query = uni.createSelectorQuery().in(instance)
  query.select('.team-selector__scroll').boundingClientRect()
  query.select('.team-selector__scroll').scrollOffset(() => {})
  query.select(`#team-chip-${props.selectedTeamId}`).boundingClientRect()
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
</script>

<style scoped lang="css">
/* 外层轨道与 FiSegmented（进球/助攻/失球、球队榜/球员榜）使用同一组 token，外观保持一致。 */
.team-selector {
  min-width: 0;
  padding: var(--fi-space-6);
  border: var(--fi-border-width) solid var(--fi-color-border-chip);
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-segmented-track);
  overflow: hidden;
  box-sizing: border-box;
}
.team-selector__scroll { width: 100%; min-width: 0; white-space: nowrap; }
.team-selector__list {
  display: inline-flex;
  gap: var(--fi-space-6);
}
.team-chip {
  display: inline-flex;
  align-items: center;
  gap: var(--fi-space-12);
  min-height: var(--fi-space-64);
  padding: 0 var(--fi-space-22);
  border-radius: var(--fi-radius-round);
  background: transparent;
  color: var(--fi-color-segmented-idle);
  font-size: var(--fi-font-24);
  line-height: var(--fi-leading-none);
  flex-shrink: 0;
  white-space: nowrap;
  margin: 0;
  transition: background-color 220ms ease, color 220ms ease;
}
.team-chip::after { border: 0; }
.team-chip.active {
  background: var(--fi-color-text-primary);
  box-shadow: var(--fi-shadow-segmented-thumb);
  color: var(--fi-color-surface);
  font-weight: var(--fi-weight-extrabold);
}
.team-chip--pressed { opacity: var(--fi-opacity-insights-pressed); }
.team-chip__avatar { width: var(--fi-space-36); height: var(--fi-space-36); flex-shrink: 0; }
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
  .team-chip { transition: none; }
}
</style>

<template>
  <view class="panel">
    <view class="section-heading">
      <view>
        <text class="section-kicker">球队列表</text>
        <text class="section-title">先选一个球队</text>
      </view>
      <text class="meta-note">{{ teams.length }} 支球队</text>
    </view>

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
.panel {
  position: relative;
  background: rgba(255, 255, 255, 0.94);
  border-radius: var(--fi-radius-xl);
  border: var(--fi-border-card);
  box-shadow: var(--fi-shadow-card);
  padding: var(--fi-space-20);
}

.section-heading {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--fi-space-12);
}

.section-kicker {
  margin: 0;
  color: var(--fi-component-kicker-text);
  font-size: var(--fi-component-kicker-size);
  font-weight: var(--fi-weight-bold);
  letter-spacing: 3rpx;
}

.section-title {
  display: block;
  margin-top: var(--fi-space-10);
  color: var(--fi-color-text-strong);
  font-size: var(--fi-component-title-size);
  line-height: 1.08;
  font-weight: var(--fi-weight-extrabold);
}

.meta-note {
  display: inline-flex;
  align-items: center;
  justify-content: flex-end;
  flex: 0 0 auto;
  flex-shrink: 0;
  max-width: 100%;
  white-space: nowrap;
  line-height: var(--fi-leading-none);
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-24);
  font-weight: var(--fi-weight-bold);
  letter-spacing: 1rpx;
  padding: var(--fi-space-8) 0 0;
}

.team-selector__scroll {
  margin-top: var(--fi-space-18);
  white-space: nowrap;
}

.team-selector__list {
  display: inline-flex;
  gap: var(--fi-space-12);
  padding-bottom: var(--fi-space-6);
}

.team-chip {
  display: inline-flex;
  align-items: center;
  gap: var(--fi-space-12);
  padding: var(--fi-space-16) var(--fi-space-22);
  border-radius: var(--fi-radius-round);
  background: #f1f2f6;
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-24);
  line-height: var(--fi-leading-none);
  transition: transform 220ms ease, box-shadow 220ms ease, background-color 220ms ease, color 220ms ease;
}

.team-chip.active {
  background: var(--fi-primitive-ink);
  color: var(--fi-primitive-white);
  transform: translateY(-2rpx);
  box-shadow: 0 10rpx 24rpx rgba(21, 22, 27, 0.14);
}

.team-chip__avatar {
  width: 38rpx;
  height: 38rpx;
}

.team-chip__avatar--fallback {
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
  background: linear-gradient(135deg, var(--fi-color-primary), var(--fi-color-primary-deep));
  color: var(--fi-primitive-white);
  font-size: var(--fi-font-22);
  font-weight: var(--fi-weight-bold);
  line-height: var(--fi-leading-none);
}
</style>

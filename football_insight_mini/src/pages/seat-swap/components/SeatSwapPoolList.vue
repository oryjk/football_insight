<template>
  <FiPanel>
    <FiSectionHeading v-if="filterKey" kicker="目标座位" :title="filterName || filterKey" align="center">
      <template #meta>
        <view class="filter-meta">
          <text class="filter-meta__count">{{ filteredCandidates.length }} 条</text>
          <button class="filter-meta__clear" hover-class="filter-meta__clear--pressed" @tap="emit('clear-filter')">✕ 清除</button>
        </view>
      </template>
    </FiSectionHeading>
    <FiSectionHeading v-else kicker="换座池" title="按分区浏览" />

    <template v-if="!filterKey">
      <view v-if="!groups.length" class="empty-row">
        <text>当前还没有换座意向。</text>
      </view>

      <template v-for="group in groups" :key="group.region_key">
        <view
          :id="buildSeatSwapRegionAnchorId(group.region_key)"
          class="group-row"
          :class="{ 'group-row--open': expandedKeys.includes(group.region_key) }"
          @tap="emit('toggle-group', group.region_key)"
        >
          <view class="group-row__main">
            <text class="group-row__name">{{ group.region_name }}</text>
            <text class="group-row__count">{{ group.requests.length }} 条</text>
            <text v-if="myDesiredKeys.includes(group.region_key)" class="group-row__hit">命中目标座位</text>
          </view>
          <image
            class="group-row__caret"
            :class="{ 'group-row__caret--open': expandedKeys.includes(group.region_key) }"
            src="/static/seat-swap/chevron-down.svg"
            mode="aspectFit"
            :aria-label="expandedKeys.includes(group.region_key) ? '收起' : '展开'"
          />
        </view>
        <view v-if="expandedKeys.includes(group.region_key)" :key="`group-${group.region_key}`" class="candidate-list">
          <view
            v-for="(candidate, index) in group.requests"
            :key="candidate.request_id"
            class="candidate-list__item"
            :style="{ animationDelay: `${resolveSeatSwapStaggerDelayMs(index)}ms` }"
          >
            <SeatSwapCandidateCard
              :candidate="candidate"
              :action="candidateAction(candidate)"
              @confirm="(id: string) => emit('confirm', id)"
              @cancel-confirmation="(id: string) => emit('cancel-confirmation', id)"
              @matched-cancel="(id: string) => emit('matched-cancel', id)"
            />
          </view>
        </view>
      </template>
    </template>

    <template v-else>
      <view v-if="!filteredCandidates.length" class="empty-row">
        <text>该分区暂无换座意向。</text>
      </view>
      <view v-if="filteredCandidates.length" :key="`filter-${filterKey}`" class="candidate-list">
        <view
          v-for="(candidate, index) in filteredCandidates"
          :key="candidate.request_id"
          class="candidate-list__item"
          :style="{ animationDelay: `${resolveSeatSwapStaggerDelayMs(index)}ms` }"
        >
          <SeatSwapCandidateCard
            :candidate="candidate"
            :action="candidateAction(candidate)"
            @confirm="(id: string) => emit('confirm', id)"
            @cancel-confirmation="(id: string) => emit('cancel-confirmation', id)"
            @matched-cancel="(id: string) => emit('matched-cancel', id)"
          />
        </view>
      </view>
    </template>
  </FiPanel>
</template>

<script setup lang="ts">
import FiPanel from '../../../components/FiPanel.vue'
import FiSectionHeading from '../../../components/FiSectionHeading.vue'
import SeatSwapCandidateCard from './SeatSwapCandidateCard.vue'
import type { SeatSwapCandidate } from '../../../types/seatSwap'
import {
  buildSeatSwapRegionAnchorId,
  resolveSeatSwapCandidateAction,
  resolveSeatSwapStaggerDelayMs,
} from '../helpers'

const props = defineProps<{
  groups: Array<{ region_key: string; region_name: string; requests: SeatSwapCandidate[] }>
  expandedKeys: string[]
  filterKey: string
  filterName: string
  filteredCandidates: SeatSwapCandidate[]
  totalCount: number
  myDesiredKeys: string[]
  isLoggedIn: boolean
  myRequestId: string | null
}>()

const emit = defineEmits<{
  (e: 'toggle-group', key: string): void
  (e: 'clear-filter'): void
  (e: 'confirm', requestId: string): void
  (e: 'cancel-confirmation', requestId: string): void
  (e: 'matched-cancel', requestId: string): void
}>()

function candidateAction(candidate: SeatSwapCandidate): ReturnType<typeof resolveSeatSwapCandidateAction> {
  return resolveSeatSwapCandidateAction({
    candidateStatus: candidate.status,
    candidateRequestId: candidate.request_id,
    myRequestId: props.myRequestId,
    isLoggedIn: props.isLoggedIn,
  })
}
</script>

<style scoped>
.candidate-list {
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-card-gap);
  margin: var(--fi-space-12) 0 var(--fi-space-card-gap);
}

.candidate-list__item {
  animation: fi-rise-in var(--fi-motion-duration-base) var(--fi-motion-ease-out) both;
}

.filter-meta {
  display: flex;
  align-items: center;
  gap: var(--fi-space-12);
}

.filter-meta__count {
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-22);
}

.filter-meta__clear {
  margin: 0;
  padding: var(--fi-space-8) var(--fi-space-18);
  border-radius: var(--fi-radius-round);
  border: var(--fi-border-width) solid var(--fi-color-border-chip);
  background: var(--fi-color-page);
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-22);
  font-weight: var(--fi-weight-bold);
  line-height: var(--fi-leading-snug);
}

.filter-meta__clear::after {
  border: 0;
}

.filter-meta__clear--pressed {
  opacity: var(--fi-primitive-opacity-pressed);
}

.group-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--fi-space-18);
  margin-top: var(--fi-space-12);
  padding: var(--fi-space-22) var(--fi-space-4);
  border-top: var(--fi-border-width) solid var(--fi-color-divider-soft);
}

.group-row__main {
  flex: 1;
  display: flex;
  align-items: center;
  gap: var(--fi-space-12);
  flex-wrap: wrap;
}

.group-row__name {
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-28);
  font-weight: var(--fi-weight-bold);
}

.group-row__count {
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-22);
}

.group-row__hit {
  padding: var(--fi-space-4) var(--fi-space-14);
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-seat-desired-bg);
  color: var(--fi-color-seat-desired);
  font-size: var(--fi-font-20);
  font-weight: var(--fi-weight-bold);
}

.group-row__caret {
  flex-shrink: 0;
  width: var(--fi-space-32);
  height: var(--fi-space-32);
  transition: transform var(--fi-motion-duration-base) var(--fi-motion-ease-out);
}

.group-row__caret--open {
  transform: rotate(180deg);
}

.empty-row {
  padding: var(--fi-space-32) var(--fi-space-12);
  text-align: center;
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-24);
}

@media (prefers-reduced-motion: reduce) {
  .candidate-list__item {
    animation-name: fi-overlay-fade-in;
    animation-duration: 150ms;
    animation-delay: 0ms !important;
  }

  .group-row__caret {
    transition: none;
  }
}
</style>

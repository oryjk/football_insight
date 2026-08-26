<template>
  <view>
    <view v-if="filterKey" class="filter-row">
      <view class="filter-row__main">
        <text class="filter-row__label">目标座位 {{ filterName || filterKey }}</text>
        <text class="filter-row__count">{{ filteredCandidates.length }} 条</text>
      </view>
      <button class="filter-row__clear" @tap="emit('clear-filter')">✕ 清除</button>
    </view>

    <template v-if="!filterKey">
      <view class="section-row">
        <text class="section-row__title">换座池</text>
        <text class="section-row__sub">共 {{ totalCount }} 条</text>
      </view>

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
          <text class="group-row__caret">{{ expandedKeys.includes(group.region_key) ? '收起' : '展开' }}</text>
        </view>
        <template v-if="expandedKeys.includes(group.region_key)">
          <SeatSwapCandidateCard
            v-for="candidate in group.requests"
            :key="candidate.request_id"
            :candidate="candidate"
            :action="candidateAction(candidate)"
            @confirm="(id: string) => emit('confirm', id)"
            @cancel-confirmation="(id: string) => emit('cancel-confirmation', id)"
            @matched-cancel="(id: string) => emit('matched-cancel', id)"
          />
        </template>
      </template>
    </template>

    <template v-else>
      <view v-if="!filteredCandidates.length" class="empty-row">
        <text>该分区暂无换座意向。</text>
      </view>
      <SeatSwapCandidateCard
        v-for="candidate in filteredCandidates"
        :key="candidate.request_id"
        :candidate="candidate"
        :action="candidateAction(candidate)"
        @confirm="(id: string) => emit('confirm', id)"
        @cancel-confirmation="(id: string) => emit('cancel-confirmation', id)"
        @matched-cancel="(id: string) => emit('matched-cancel', id)"
      />
    </template>
  </view>
</template>

<script setup lang="ts">
import SeatSwapCandidateCard from '../../../components/SeatSwapCandidateCard.vue'
import type { SeatSwapCandidate } from '../../../types/seatSwap'
import { buildSeatSwapRegionAnchorId, resolveSeatSwapCandidateAction } from '../helpers'

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
.filter-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--fi-space-18) var(--fi-space-8);
  margin-bottom: var(--fi-space-8);
  border-bottom: 1rpx solid var(--fi-color-border-chip);
}

.filter-row__main {
  display: flex;
  align-items: center;
  gap: var(--fi-space-14);
}

.filter-row__label {
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-28);
  font-weight: 400;
}

.filter-row__count {
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-22);
}

.filter-row__clear {
  flex-shrink: 0;
  background: var(--fi-color-page);
  border: 1rpx solid var(--fi-color-border-chip);
  color: var(--fi-color-text-secondary);
  padding: var(--fi-space-8) var(--fi-space-18);
  border-radius: var(--fi-radius-round);
  font-size: var(--fi-font-22);
  font-weight: 400;
}

.filter-row__clear::after {
  border: 0;
}

.section-row {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  padding: var(--fi-space-20) var(--fi-space-8) var(--fi-space-12);
}

.section-row__title {
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-30);
  font-weight: 400;
}

.section-row__sub {
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-22);
}

.group-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--fi-space-18);
  padding: var(--fi-space-22) 12rpx;
  border-top: 1rpx solid var(--fi-color-border-chip);
}

.group-row:first-of-type {
  border-top: 0;
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
  font-weight: 400;
}

.group-row__count {
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-22);
}

.group-row__hit {
  font-size: var(--fi-font-20);
  padding: 4rpx var(--fi-space-14);
  border-radius: var(--fi-radius-round);
  background: #eef8f2;
  color: #167348;
  border: 1rpx solid rgba(29, 138, 85, 0.3);
}

.group-row__caret {
  flex-shrink: 0;
  font-size: var(--fi-font-22);
  color: #9aa0aa;
  padding: var(--fi-space-6) 4rpx;
  line-height: var(--fi-leading-snug);
}

.empty-row {
  padding: var(--fi-space-32) 12rpx;
  text-align: center;
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-24);
}
</style>

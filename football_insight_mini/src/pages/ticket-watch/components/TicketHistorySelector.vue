<template>
  <view class="panel">
    <view class="section-heading section-heading--compact">
      <view>
        <text class="section-kicker">History Matches</text>
        <text class="section-title">选择一场历史比赛</text>
      </view>
      <text class="meta-pill">{{ matches.length }} 场</text>
    </view>

    <scroll-view
      scroll-x
      class="history-chip-scroll"
      :scroll-left="historyChipScrollLeft"
      scroll-with-animation
      show-scrollbar="false"
    >
      <view class="history-chip-list">
        <button
          v-for="match in matches"
          :id="`history-chip-${match.match_id}`"
          :key="match.match_id"
          class="history-chip"
          :class="{
            'history-chip--active': selectedMatchId === match.match_id,
          }"
          @click="emit('select', match.match_id)"
        >
          <text class="history-chip__date">{{ match.match_date }}</text>
          <text class="history-chip__teams"
            >{{ match.home_team_name }} VS {{ match.away_team_name }}</text
          >
        </button>
      </view>
    </scroll-view>
  </view>
</template>

<script setup lang="ts">
import type { TicketWatchMatchSummary } from '../../../types/ticketWatch'

import { getCurrentInstance, nextTick, ref, watch, onMounted } from 'vue'
const props = defineProps<{
  matches: TicketWatchMatchSummary[]
  selectedMatchId: number | null
}>()
const emit = defineEmits<{ (e: 'select', matchId: number): void }>()
const historyChipScrollLeft = ref(0)
const instance = getCurrentInstance()
async function centerSelectedChip(): Promise<void> {
  if (!instance || props.selectedMatchId === null) return
  await nextTick()
  const query = uni.createSelectorQuery().in(instance)
  query.select('.history-chip-scroll').boundingClientRect()
  query.select('.history-chip-scroll').scrollOffset(() => {})
  query.select(`#history-chip-${props.selectedMatchId}`).boundingClientRect()
  query.exec((result) => {
    const [scroll, offset, chip] = (result ?? []) as Array<{
      left?: number
      width?: number
      scrollLeft?: number
    } | null>
    if (
      typeof scroll?.left !== 'number' ||
      typeof offset?.scrollLeft !== 'number' ||
      typeof chip?.left !== 'number' ||
      typeof chip?.width !== 'number' ||
      typeof scroll?.width !== 'number'
    )
      return
    historyChipScrollLeft.value = Math.max(
      0,
      Math.round(
        offset.scrollLeft +
          chip.left +
          chip.width / 2 -
          scroll.left -
          scroll.width / 2,
      ),
    )
  })
}
watch(
  () => props.selectedMatchId,
  () => {
    void centerSelectedChip()
  },
)
onMounted(() => {
  void centerSelectedChip()
})
</script>

<style scoped lang="css">
@import '../ticket-watch-shared.css';

.history-chip-scroll {
  width: 100%;
  margin-top: var(--fi-space-14);
}

.history-chip-list {
  display: inline-flex;
  gap: var(--fi-space-10);
  padding-bottom: var(--fi-space-6);
}

.history-chip {
  min-width: 240rpx;
  border-radius: var(--fi-radius-ticket-20);
  padding: var(--fi-space-16) var(--fi-space-18);
  background: var(--fi-color-ticket-history-chip-background);
  text-align: left;
}

.history-chip--active {
  background: var(--fi-color-text-primary);
}

.history-chip__date {
  display: block;
  color: var(--fi-color-ticket-history-chip-date-color);
  font-size: var(--fi-font-18);
}

.history-chip__teams {
  display: block;
  margin-top: var(--fi-space-8);
  color: var(--fi-color-ticket-history-chip-teams-color);
  font-size: var(--fi-font-20);
  font-weight: var(--fi-weight-normal);
  line-height: var(--fi-leading-ticket-1-45);
}

.history-chip--active .history-chip__date,
.history-chip--active .history-chip__teams {
  color: var(--fi-color-bg);
}
</style>

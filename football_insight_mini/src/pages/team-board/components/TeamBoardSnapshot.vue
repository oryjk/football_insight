<template>
  <view class="team-board-snapshot" :class="{ 'team-board-snapshot--composer': composer }">
    <view class="team-board-snapshot__summary">
      <view>
        <text class="team-board-snapshot__label">{{ snapshot.insight_label }}</text>
        <text class="team-board-snapshot__headline">{{ snapshot.summary_label }}</text>
      </view>
      <text class="team-board-snapshot__summary-value">{{ snapshot.summary_value }}</text>
    </view>

    <view class="team-board-snapshot__sections">
      <view
        v-for="section in snapshot.sections"
        :key="section.title"
        class="team-board-snapshot__section"
      >
        <view class="team-board-snapshot__section-head">
          <text>{{ section.title }}</text>
          <text>{{ section.metric_label }}</text>
        </view>

        <view class="team-board-snapshot__rows">
          <view
            v-for="item in section.items.slice(0, 3)"
            :key="`${section.title}-${item.item_id ?? item.name}`"
            class="team-board-snapshot__row"
          >
            <view class="team-board-snapshot__identity">
              <image :src="item.avatar_storage_url || ''" mode="aspectFit" class="team-board-snapshot__avatar" />
              <view>
                <text class="team-board-snapshot__name">{{ item.name }}</text>
                <text class="team-board-snapshot__metric">{{ metricSuffix(section.metric_label, item.value) }}</text>
              </view>
            </view>
            <text class="team-board-snapshot__share">{{ formatShare(item.share) }}</text>
          </view>
        </view>
      </view>
    </view>
  </view>
</template>

<script setup lang="ts">
import type { TeamBoardSnapshot } from '../../../types/teamBoard'
import { formatShare, metricSuffix } from '../helpers'

defineProps<{ snapshot: TeamBoardSnapshot; composer?: boolean }>()
</script>

<style scoped lang="css">
@import '../team-board-shared.css';

.team-board-snapshot {
  margin-top: var(--fi-space-18);
  padding: var(--fi-space-22);
  border-radius: var(--fi-radius-lg);
  background: #f7f8fb;
}
.team-board-snapshot__summary-value {
  color: #121212;
  font-size: var(--fi-font-48);
  font-weight: 800;
}
.team-board-snapshot__section {
  display: grid;
  gap: var(--fi-space-10);
}
.team-board-snapshot__rows {
  display: grid;
  gap: var(--fi-space-10);
}
.team-board-snapshot__row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--fi-space-16);
}
.team-board-snapshot__identity {
  display: flex;
  align-items: center;
  gap: var(--fi-space-12);
}
</style>

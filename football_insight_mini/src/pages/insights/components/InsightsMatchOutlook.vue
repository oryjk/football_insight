<template>
  <FiPanel>
    <FiSectionHeading kicker="主胜 / 平 / 客胜" :title="outlookCopy.title" />
    <view class="outlook-match-picker">
      <scroll-view
        v-if="matches.length"
        :key="matches.map(match => match.match_id).join(',')"
        scroll-x
        scroll-with-animation
        :scroll-into-view="selectedMatchId === null ? '' : `outlook-match-${selectedMatchId}`"
        class="outlook-fixtures"
      >
        <view class="outlook-fixtures__list"
          ><button
            v-for="match in matches"
            :id="`outlook-match-${match.match_id}`"
            :key="match.match_id"
            class="outlook-fixture"
            :class="{
              'outlook-fixture--active': match.match_id === selectedMatchId,
            }"
            hover-class="outlook--pressed"
            @tap="emit('select-match', match.match_id)"
          >
            {{ match.match_date }} {{ match.home_team_name }} ·
            {{ match.away_team_name }}
          </button></view
        >
      </scroll-view>
      <text v-else class="outlook-note">该球队暂无待赛比赛</text>
    </view>
    <view class="outlook-forecast" :aria-busy="loading">
      <view
        :key="`${selectedMatchId}:${outlook?.generated_at ?? 'pending'}`"
        class="outlook-forecast__body"
      >
        <text v-if="loading" class="outlook-note">正在读取赛前资料</text>
        <text v-else-if="error" class="outlook-note">{{ error }}</text>
        <template v-else-if="outlook">
          <view
            class="outlook-bar"
            role="img"
            :aria-label="`主胜${parts[0]}%，平${parts[1]}%，客胜${parts[2]}%`"
            ><view
              v-for="(part, index) in parts"
              :key="index"
              class="outlook-part"
              :class="`outlook-part--${index}`"
              :style="{ width: `${part}%` }"
          /></view>
          <view class="outlook-labels"
            ><text>主胜 {{ parts[0] }}%</text><text>平 {{ parts[1] }}%</text
            ><text>客胜 {{ parts[2] }}%</text></view
          >
          <text v-if="outlook.schedule_changed" class="outlook-note"
            >赛程已调整，此处保留原赛前记录</text
          >
          <text v-if="outlook.outcome !== null" class="outlook-note"
            >实际结果：{{ ['主胜', '平', '客胜'][outlook.outcome] }}</text
          >
        </template>
        <text v-else class="outlook-note">{{ outlookCopy.empty }}</text>
      </view>
    </view>
    <view class="outlook-record"
      ><text class="outlook-record__title">{{ outlookCopy.record }}</text
      ><text class="outlook-note">{{
        record ? recordSummary(record) : '暂无公开计分资料'
      }}</text
      ><text
        v-if="record?.rps_model !== null && record?.rps_model !== undefined"
        class="outlook-note"
        >模型 RPS {{ record.rps_model.toFixed(4) }} · 基线
        {{ record.rps_baseline?.toFixed(4) }}（越低越好）</text
      ><text
        v-if="completedMatches !== null && completedMatches < 30"
        class="outlook-note"
        >{{ outlookCopy.early }}</text
      ></view
    >
    <text class="outlook-note">{{ outlookCopy.disclaimer }}</text>
  </FiPanel>
</template>
<script setup lang="ts">
import { computed } from 'vue'
import FiPanel from '../../../components/FiPanel.vue'
import FiSectionHeading from '../../../components/FiSectionHeading.vue'
import type { MatchCard } from '../../../types/insight'
import type { MatchOutlook, OutlookRecord } from '../../../types/matchOutlook'
import { percentages, outlookCopy, recordSummary } from '../matchOutlook'
const props = defineProps<{
  matches: MatchCard[]
  selectedMatchId: number | null
  outlook: MatchOutlook | null
  record: OutlookRecord | null
  completedMatches: number | null
  loading: boolean
  error: string
}>()
const emit = defineEmits<{
  (event: 'select-match', id: number): void
}>()
const parts = computed(() =>
  props.outlook
    ? percentages([
        props.outlook.home_win,
        props.outlook.draw,
        props.outlook.away_win,
      ])
    : [],
)
</script>
<style scoped>
.outlook-fixtures {
  width: 100%;
  height: var(--fi-space-72);
  white-space: nowrap;
}
.outlook-match-picker {
  height: var(--fi-space-72);
  display: flex;
  align-items: center;
  margin-top: var(--fi-space-16);
}
.outlook-fixtures__list {
  display: inline-flex;
  align-items: center;
  gap: var(--fi-space-8);
  height: 100%;
}
.outlook-forecast {
  min-height: calc(var(--fi-space-24) * 5);
}
.outlook-forecast__body {
  display: flex;
  flex-direction: column;
  animation: outlook-body-fade 260ms ease both;
}
@keyframes outlook-body-fade {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}
@media (prefers-reduced-motion: reduce) {
  .outlook-forecast__body {
    animation: none;
  }
}
.outlook-fixture {
  flex-shrink: 0;
  white-space: nowrap;
  margin: 0;
  padding: var(--fi-space-12) var(--fi-space-16);
  color: var(--fi-color-text-secondary);
  background: var(--fi-color-page);
  border-radius: var(--fi-radius-sm);
  font-size: var(--fi-font-24);
  line-height: var(--fi-leading-normal);
  text-align: left;
  border: var(--fi-border-width) solid var(--fi-color-divider);
}
.outlook-fixture::after {
  border: 0;
}
.outlook-fixture--active {
  border-color: var(--fi-color-text-primary);
  color: var(--fi-color-text-primary);
}
.outlook--pressed {
  opacity: var(--fi-opacity-pressed);
}
.outlook-bar {
  display: flex;
  height: var(--fi-space-16);
  overflow: hidden;
  border-radius: var(--fi-radius-round);
  margin-top: var(--fi-space-24);
}
.outlook-part--0 {
  background: var(--fi-color-primary);
}
.outlook-part--1 {
  background: var(--fi-color-divider);
}
.outlook-part--2 {
  background: var(--fi-color-text-secondary);
}
.outlook-labels {
  display: flex;
  justify-content: space-between;
  margin-top: var(--fi-space-12);
  font-size: var(--fi-font-26);
  color: var(--fi-color-text-primary);
}
.outlook-note {
  display: block;
  margin-top: var(--fi-space-12);
  font-size: var(--fi-font-24);
  line-height: var(--fi-leading-normal);
  color: var(--fi-color-text-secondary);
}
.outlook-record {
  margin-top: var(--fi-space-20);
  padding-top: var(--fi-space-16);
  border-top: var(--fi-border-width) solid var(--fi-color-divider);
}
.outlook-record__title {
  font-size: var(--fi-font-28);
  font-weight: var(--fi-weight-bold);
  color: var(--fi-color-text-primary);
}
</style>

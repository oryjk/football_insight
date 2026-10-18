<template>
  <FiPanel>
    <FiSectionHeading kicker="主胜 / 平 / 客胜" :title="outlookCopy.title" />
    <InsightsTeamSelector
      v-if="teams.length"
      :teams="teams"
      :selected-team-id="selectedTeamId"
      @update:selected-team-id="emit('select-team', $event)"
    />
    <view v-if="matches.length" class="outlook-fixtures"
      ><button
        v-for="match in matches"
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
      <text v-if="outlook.tau !== null" class="outlook-note">{{
        confidenceText(outlook.tau)
      }}</text>
      <text class="outlook-note"
        >{{ outlook.frozen ? '赛前记录已冻结' : '数据随赛前同步更新'
        }}{{
          outlook.schedule_changed ? ' · 赛程已调整，保留原记录' : ''
        }}</text
      >
      <text v-if="outlook.outcome !== null" class="outlook-note"
        >实际结果：{{ ['主胜', '平', '客胜'][outlook.outcome] }}</text
      >
    </template>
    <text v-else class="outlook-note">{{ outlookCopy.empty }}</text>
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
import InsightsTeamSelector from './InsightsTeamSelector.vue'
import type { MatchCard } from '../../../types/insight'
import type { SupportTeam } from '../../../types/support'
import type { MatchOutlook, OutlookRecord } from '../../../types/matchOutlook'
import {
  percentages,
  confidenceText,
  outlookCopy,
  recordSummary,
} from '../matchOutlook'
const props = defineProps<{
  teams: SupportTeam[]
  selectedTeamId: number | null
  matches: MatchCard[]
  selectedMatchId: number | null
  outlook: MatchOutlook | null
  record: OutlookRecord | null
  completedMatches: number | null
  loading: boolean
  error: string
}>()
const emit = defineEmits<{
  (event: 'select-team', id: number): void
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
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-8);
  margin-top: var(--fi-space-16);
}
.outlook-fixture {
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

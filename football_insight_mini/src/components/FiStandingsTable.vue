<template>
  <view>
    <view class="standings__caption">
      <text>本赛季赛果 · 每行 15 场，由旧到新</text>
      <text>积分</text>
    </view>
    <view class="standings__list">
      <view
        v-for="team in teams"
        :key="team.team_id"
        class="standings__row"
        :class="aclOf(team.rank_no) ? `standings__row--acl-${aclOf(team.rank_no)?.tier}` : ''"
        role="button"
        :aria-label="`${team.team_name}，${team.points}积分，${aclOf(team.rank_no)?.fullLabel ?? ''}查看完整战绩`"
        hover-class="standings__row--pressed"
        hover-stay-time="100"
        @click="emit('select', { key: team.team_id })"
      >
        <text class="standings__rank" :class="{ 'standings__rank--first': team.rank_no === 1 }">#{{ team.rank_no }}</text>
        <image class="standings__avatar" :src="team.avatar_storage_url || ''" mode="aspectFit" />
        <view class="standings__identity">
          <text class="standings__name">{{ team.team_name }}</text>
          <text v-if="aclOf(team.rank_no)" class="standings__acl" :class="`standings__acl--${aclOf(team.rank_no)?.tier}`">{{ aclOf(team.rank_no)?.label }}</text>
        </view>
        <view class="standings__form">
          <FiResultDots :items="buildOutcomeDots(team.recent_form)" size="sm" align="end" :per-row="15" highlight-latest empty-text="近期战绩待更新" />
        </view>
        <text class="standings__points">{{ team.points }}</text>
      </view>
    </view>
    <view class="standings__footer">
      <view class="standings__legend">
        <text class="standings__legend-item standings__legend-item--win">胜</text>
        <text class="standings__legend-item standings__legend-item--draw">平</text>
        <text class="standings__legend-item standings__legend-item--loss">负</text>
      </view>
      <text>点击球队查看完整战绩</text>
    </view>
  </view>
</template>

<script setup lang="ts">
// 走势线组件 FiTeamFormTrend 保留，需要折线形态时在此换回：<FiTeamFormTrend :outcomes="team.recent_form" />
import FiResultDots from './FiResultDots.vue'
import { buildOutcomeDots } from '../utils/resultDots'
import { resolveAclQualification } from '../utils/aclQualification'
import type { OverviewStanding } from '../types/insight'

const props = withDefaults(defineProps<{ teams: OverviewStanding[]; qualification?: boolean }>(), { qualification: false })
// 亚冠资格只在榜单页启用；首页等其他入口保持原样。
const aclOf = (rankNo: number) => (props.qualification ? resolveAclQualification(rankNo) : null)
const emit = defineEmits<{ (event: 'select', row: { key: number }): void }>()
</script>

<style scoped>
.standings__caption,
.standings__footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--fi-space-16);
  color: var(--fi-color-text-caption);
  font-size: var(--fi-font-22);
  line-height: var(--fi-leading-normal);
}
.standings__caption {
  margin-top: var(--fi-space-18);
  padding-bottom: var(--fi-space-12);
  border-bottom: var(--fi-border-width) solid var(--fi-color-divider-soft);
}
.standings__row {
  display: grid;
  grid-template-columns: var(--fi-space-32) var(--fi-space-44) minmax(0, calc(var(--fi-font-28) * 4.5)) minmax(0, 1fr) var(--fi-space-56);
  align-items: center;
  column-gap: var(--fi-space-8);
  padding: var(--fi-space-20) 0 var(--fi-space-16);
  border-bottom: var(--fi-border-width) solid var(--fi-color-divider-soft);
}
.standings__row--acl-elite { background: linear-gradient(90deg, rgba(212, 160, 23, 0.14), rgba(212, 160, 23, 0) 65%); box-shadow: inset 6rpx 0 0 #d4a017; }
.standings__row--acl-second { background: linear-gradient(90deg, rgba(84, 118, 168, 0.12), rgba(84, 118, 168, 0) 65%); box-shadow: inset 6rpx 0 0 #6b8cc0; }
.standings__row--pressed {
  background: var(--fi-color-team-form-pressed);
}
.standings__rank {
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-24);
  font-weight: var(--fi-weight-bold);
  font-variant-numeric: tabular-nums;
}
.standings__rank--first {
  color: var(--fi-color-primary);
}
.standings__avatar {
  width: var(--fi-space-44);
  height: var(--fi-space-44);
}
.standings__identity { min-width: 0; display: flex; flex-direction: column; align-items: flex-start; gap: var(--fi-space-6); }
.standings__acl { padding: var(--fi-space-4) var(--fi-space-10); border-radius: var(--fi-radius-round); font-size: var(--fi-font-20); line-height: var(--fi-leading-none); font-weight: var(--fi-weight-bold); white-space: nowrap; }
.standings__acl--elite { background: #fbeec2; color: #8a6200; }
.standings__acl--second { background: #dfe8f6; color: #3f5f8f; }
.standings__name {
  max-width: 100%;
  min-width: 0;
  font-size: var(--fi-font-28);
  font-weight: var(--fi-weight-bold);
  color: var(--fi-color-text-strong);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.standings__points {
  font-size: var(--fi-font-36);
  line-height: var(--fi-leading-tight);
  font-weight: var(--fi-weight-bold);
  color: var(--fi-color-text-strong);
  text-align: right;
  font-variant-numeric: tabular-nums;
}
.standings__form {
  min-width: 0;
}
.standings__footer {
  margin-top: var(--fi-space-16);
  flex-wrap: wrap;
  gap: var(--fi-space-8);
}
.standings__legend {
  display: flex;
  align-items: center;
  gap: var(--fi-space-16);
}
.standings__legend-item {
  display: inline-flex;
  align-items: center;
  gap: var(--fi-space-6);
}
.standings__legend-item::before {
  content: '';
  width: var(--fi-space-8);
  height: var(--fi-space-8);
  border-radius: var(--fi-radius-round);
}
.standings__legend-item--win::before {
  background: var(--fi-color-primary);
}
.standings__legend-item--draw::before {
  background: var(--fi-color-team-form-line);
}
.standings__legend-item--loss::before {
  background: var(--fi-color-success-accent);
}
</style>

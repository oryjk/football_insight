<template>
  <FiPanel>
    <view class="contribution-surface">
      <FiSegmented v-model="tab" :items="tabItems" aria-label="贡献类型" />

      <view v-if="tab === 'goals-for'" class="pill-row">
        <view
          v-for="item in INSIGHT_GOALS_FOR_DIMENSIONS"
          :key="item.key"
          class="pill-row__item"
          :class="{ active: dimension === item.key }"
          role="button"
          :aria-label="item.label"
          :aria-pressed="dimension === item.key"
          hover-class="contribution-control--pressed"
          @tap="dimension = item.key"
        >{{ item.label }}</view>
      </view>

      <text class="contribution-caption">{{ INSIGHT_CONTRIBUTION_CAPTIONS[viewKey] }}</text>

      <view :key="viewKey" class="contribution-body">
        <view v-if="visibleRows.length" class="contribution-list">
          <view v-for="row in visibleRows" :key="row.instanceKey" class="contribution-row">
            <view class="contribution-row__header">
              <view class="contribution-row__identity">
                <image
                  :src="row.avatar || ''"
                  :mode="row.avatarMode === 'fill' ? 'aspectFill' : 'aspectFit'"
                  class="contribution-row__avatar"
                  :class="{ 'contribution-row__avatar--player': row.avatarMode === 'fill' }"
                />
                <view class="contribution-row__copy">
                  <text class="contribution-row__name">{{ row.name }}</text>
                  <text class="contribution-row__note">{{ row.note }}</text>
                </view>
              </view>
              <text class="contribution-row__share">{{ formatInsightShare(row.share) }}</text>
            </view>
            <view class="contribution-row__bar">
              <view
                class="contribution-row__fill"
                :class="`contribution-row__fill--${row.variant}`"
                :style="{ width: insightBarWidth(row.share) }"
              />
            </view>
          </view>
        </view>
        <text v-else class="contribution-empty">暂无数据</text>
        <button
          v-if="shouldShowInsightToggle(rows[viewKey])"
          class="contribution-toggle"
          hover-class="contribution-control--pressed"
          @tap="emit('toggle', viewKey)"
        >{{ expanded[viewKey] ? '收起' : '更多' }}</button>
      </view>
    </view>
  </FiPanel>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import FiPanel from '../../../components/FiPanel.vue'
import FiSegmented from '../../../components/FiSegmented.vue'
import goalsIdle from '../../../static/insights/contribution-goals-idle.svg'
import goalsActive from '../../../static/insights/contribution-goals-active.svg'
import assistsIdle from '../../../static/insights/contribution-assists-idle.svg'
import assistsActive from '../../../static/insights/contribution-assists-active.svg'
import concededIdle from '../../../static/insights/contribution-conceded-idle.svg'
import concededActive from '../../../static/insights/contribution-conceded-active.svg'
import type { FiSegmentedItem } from '../../../types/segmented'
import {
  INSIGHT_CONTRIBUTION_TABS,
  INSIGHT_GOALS_FOR_DIMENSIONS,
  INSIGHT_CONTRIBUTION_CAPTIONS,
  resolveContributionViewKey,
  formatInsightShare,
  getVisibleInsightContributions,
  insightBarWidth,
  shouldShowInsightToggle,
  type InsightContributionTab,
  type InsightGoalsForDimension,
  type InsightContributionViewKey,
  type InsightContributionRow,
} from '../helpers'

const props = defineProps<{
  rows: Record<InsightContributionViewKey, InsightContributionRow[]>
  expanded: Record<InsightContributionViewKey, boolean>
}>()
const emit = defineEmits<{ (e: 'toggle', key: InsightContributionViewKey): void }>()

const TAB_ICONS: Record<InsightContributionTab, { icon: string; iconActive: string }> = {
  'goals-for': { icon: goalsIdle, iconActive: goalsActive },
  assists: { icon: assistsIdle, iconActive: assistsActive },
  'goals-against': { icon: concededIdle, iconActive: concededActive },
}
const tabItems: FiSegmentedItem[] = INSIGHT_CONTRIBUTION_TABS.map(item => ({
  key: item.key,
  label: item.label,
  ...TAB_ICONS[item.key],
}))

const tab = ref<InsightContributionTab>('goals-for')
const dimension = ref<InsightGoalsForDimension>('opponent')
const viewKey = computed(() => resolveContributionViewKey(tab.value, dimension.value))
const visibleRows = computed(() =>
  getVisibleInsightContributions(props.rows[viewKey.value], props.expanded[viewKey.value]),
)
</script>

<style scoped lang="css">
.contribution-surface { display: grid; gap: var(--fi-space-14); min-width: 0; }
.pill-row { display: flex; flex-wrap: wrap; gap: var(--fi-space-10); padding-bottom: var(--fi-space-4); }
.pill-row__item {
  padding: var(--fi-space-18) var(--fi-space-22);
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-page);
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-24);
  line-height: var(--fi-leading-none);
  white-space: nowrap;
}
.pill-row__item.active { background: var(--fi-color-text-primary); color: var(--fi-color-surface); font-weight: var(--fi-weight-extrabold); }
.contribution-control--pressed { opacity: var(--fi-opacity-insights-pressed); }
.contribution-caption { color: var(--fi-color-text-muted); font-size: var(--fi-font-24); line-height: var(--fi-leading-normal); }
.contribution-body { min-width: 0; animation: fi-rise-in 220ms ease both; }
.contribution-list { display: grid; gap: var(--fi-space-12); }
.contribution-row { display: grid; gap: var(--fi-space-10); }
.contribution-row__header { display: flex; align-items: center; justify-content: space-between; gap: var(--fi-space-12); }
.contribution-row__identity { display: flex; align-items: center; gap: var(--fi-space-12); min-width: 0; }
.contribution-row__avatar { width: var(--fi-space-48); height: var(--fi-space-48); flex-shrink: 0; }
.contribution-row__avatar--player { border-radius: var(--fi-radius-round); }
.contribution-row__copy { min-width: 0; }
.contribution-row__name {
  display: block;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-26);
  line-height: var(--fi-leading-normal);
  font-weight: var(--fi-weight-bold);
}
.contribution-row__note,
.contribution-row__share { color: var(--fi-color-text-caption); font-size: var(--fi-font-22); line-height: var(--fi-leading-normal); }
.contribution-row__share { flex-shrink: 0; }
.contribution-row__bar { height: var(--fi-space-14); border-radius: var(--fi-radius-round); background: var(--fi-color-insights-track); overflow: hidden; }
.contribution-row__fill { height: 100%; border-radius: inherit; background: var(--fi-color-text-primary); transition: width 320ms cubic-bezier(0.22, 1, 0.36, 1); }
.contribution-row__fill--red,
.contribution-row__fill--danger { background: var(--fi-color-primary); }
.contribution-row__fill--green { background: var(--fi-color-insights-assist); }
.contribution-empty { color: var(--fi-color-text-muted); font-size: var(--fi-font-24); line-height: var(--fi-leading-normal); }
.contribution-toggle {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  margin: var(--fi-space-18) 0 0;
  padding: var(--fi-space-12) var(--fi-space-22);
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-insights-toggle);
  color: var(--fi-color-primary);
  font-size: var(--fi-font-24);
  white-space: nowrap;
  line-height: var(--fi-leading-none);
}
.contribution-toggle::after { border: 0; }
@media (prefers-reduced-motion: reduce) {
  .contribution-row__fill { transition: none; }
  .contribution-body { animation: none; }
}
</style>
